use std::collections::BTreeMap;
use std::ffi::OsStr;

use anyhow::{anyhow, Result};
pub use smol::process::Command;

use thiserror::__private::PathAsDisplay;
use typedir::{Extend, PathBuf as P};

use crate::conf::{self, LargoConfig};
use crate::project::{self, Dependencies, ProfileName, Project, ProjectSettings, SystemSettings};
use crate::{dirs, engines};

// Probably shouldn't really be public: currently is just for PdflatexBuilder's sake
pub struct TexInput(String);

impl AsRef<OsStr> for TexInput {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

/// Variables available at TeX run time
#[derive(Debug)]
struct LargoVars<'a> {
    profile: ProfileName<'a>,
    bibliography: Option<&'a str>,
    output_directory: P<dirs::ProfileBuildDir>,
}

// For use in `LargoVars::to_defs`
macro_rules! write_lv {
    ($defs:expr, $var:expr, $val:expr) => {
        write!($defs, r#"\def\Largo{}{{{}}}"#, $var, $val).expect("internal error");
    };
}

impl<'a> LargoVars<'a> {
    fn from_build_settings(settings: &'a BuildSettings<'a>) -> Self {
        // NOTE: unfortunate clone
        let root_dir = settings.root_dir.clone();
        Self {
            profile: settings.profile_name,
            bibliography: settings.conf.default_bibliography,
            output_directory: root_dir.extend(()).extend(&settings.profile_name),
        }
    }

    fn to_defs(self) -> String {
        use std::fmt::Write;
        let mut defs = String::new();
        {
            let defs = &mut defs;
            write_lv!(defs, "Profile", &self.profile);
            if let Some(bib) = self.bibliography {
                write_lv!(defs, "Bibliography", bib);
            }
            write_lv!(defs, "OutputDirectory", &self.output_directory.as_display());
        }
        defs
    }
}

fn tex_input(largo_vars: LargoVars, _conf: &LargoConfig) -> TexInput {
    let vars = largo_vars.to_defs();
    let main_file = dirs::MAIN_FILE;
    TexInput(format!(r#"{vars}\input{{{main_file}}}"#))
}

/// Environment variables for the build command
#[derive(Debug, Default)]
pub struct BuildVars(BTreeMap<&'static str, String>);

impl BuildVars {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    fn insert<V: std::fmt::Display>(&mut self, k: &'static str, v: V) {
        self.0.insert(k, format!("{}", v));
    }
}

impl BuildVars {
    fn with_dependencies(mut self, deps: &project::Dependencies) -> Self {
        let mut tex_inputs = String::new();
        for (_dep_name, dep_body) in deps {
            match &dep_body {
                project::Dependency::Path { path } => {
                    tex_inputs += &path;
                    tex_inputs.push(':');
                }
            }
        }
        if !tex_inputs.is_empty() {
            self.insert("TEXINPUTS", tex_inputs);
        }
        self
    }

    /// NOTE: there seems to be no way to *actually* turn off line wrapping from
    /// pdflatex, but we can fake it by wrapping at a very high column number.
    fn disable_line_wrapping(mut self) -> Self {
        self.insert("max_print_line", i32::MAX);
        self
    }

    pub fn apply(&self, cmd: &mut Command) {
        for (var, val) in &self.0 {
            cmd.env(var, val);
        }
    }
}

/// Level of severity of information to forward from TeX engine
pub enum LogLevel {
    Warning,
    Error,
}

#[derive(Default)]
pub enum Verbosity {
    /// Never emit anything, even on failure
    #[default]
    Silent,
    /// Only forward TeX engine warnings, errors
    Info(LogLevel),
    /// Forward all TeX engine output
    Noisy,
}

pub struct BuildBuilder<'a> {
    conf: &'a LargoConfig<'a>,
    project: Project<'a>,
    verbosity: Verbosity,
    /// Which profile to build in
    profile: Option<crate::project::ProfileName<'a>>,
}

impl<'a> BuildBuilder<'a> {
    pub fn new(conf: &'a LargoConfig, project: Project<'a>) -> Self {
        Self {
            conf,
            project,
            verbosity: Verbosity::Silent,
            profile: None,
        }
    }

    pub fn with_profile(mut self, name: Option<crate::project::ProfileName<'a>>) -> Self {
        self.profile = name.as_ref().copied();
        self
    }

    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Unpack the data we've been passed into a more convenient shape
    fn to_build_settings(self) -> Result<BuildSettings<'a>> {
        let conf = self.conf;
        let project = self.project;
        let root_dir = project.root;
        let profile_name = self.profile.unwrap_or(self.conf.default_profile);
        // FIXME This is a bug: there should *always* be a default profile to select
        let profiles = project.config.profiles;
        let profile = profiles
            .select_profile(&profile_name)
            .ok_or_else(|| anyhow!("profile `{}` not found", profile_name))?;
        let proj_conf = project.config.project;
        let project_settings = proj_conf.project_settings.merge(profile.project_settings);
        let system_settings = proj_conf.system_settings.merge(profile.system_settings);
        let dependencies = project.config.dependencies;
        Ok(BuildSettings {
            conf,
            root_dir,
            profile_name,
            system_settings,
            project_settings,
            dependencies,
            verbosity: self.verbosity,
        })
    }

    pub fn try_finish(self) -> Result<Build> {
        let build_settings = self.to_build_settings()?;
        build_settings.to_build()
    }
}

/// An intermediate state of unpackaging and treating all the data we've
/// received
struct BuildSettings<'a> {
    conf: &'a LargoConfig<'a>,
    root_dir: P<dirs::RootDir>,
    profile_name: ProfileName<'a>,
    system_settings: SystemSettings,
    project_settings: ProjectSettings,
    dependencies: Dependencies<'a>,
    verbosity: Verbosity,
}

impl<'a> BuildSettings<'a> {
    fn executable(&self) -> &conf::Executable {
        let engine = self
            .system_settings
            .tex_engine
            .unwrap_or(self.conf.default_tex_engine);
        let system = self
            .system_settings
            .tex_format
            .unwrap_or(self.conf.default_tex_format);
        self.conf.choose_program(engine, system)
    }

    fn build_vars(&self) -> BuildVars {
        BuildVars::new()
            .with_dependencies(&self.dependencies)
            .disable_line_wrapping()
    }

    fn to_build(mut self) -> Result<Build> {
        let largo_vars = LargoVars::from_build_settings(&self);
        let tex_input = tex_input(largo_vars, self.conf);
        let mut plb = engines::pdflatex::PdflatexBuilder::new(self.executable(), tex_input)
            .with_src_dir((&mut self.root_dir).extend(()))
            .with_build_vars(&self.build_vars())
            .with_verbosity(self.verbosity)
            .with_synctex(self.project_settings.synctex);
        let build_dir: P<dirs::ProfileBuildDir> =
            self.root_dir.extend(()).extend(&self.profile_name);
        // FIXME this should happen *at build time*, right?
        std::fs::create_dir_all(&build_dir).expect("TODO: Sorry, this code needs to be refactored; it's a waste of time to handle this error.");
        plb.cli_options.output_directory = Some(build_dir.into());
        match self.project_settings.shell_escape {
            Some(true) => {
                plb.cli_options.shell_escape = true;
            }
            Some(false) => {
                plb.cli_options.no_shell_escape = true;
            }
            None => (),
        };
        Ok(plb.finalize())
    }
}

#[derive(Debug)]
pub struct Build {
    // FIXME this absolutely should not be public
    pub cmd: Command,
}

impl Build {
    pub async fn run(mut self) -> Result<()> {
        self.cmd.spawn()?;
        // `async_process::Child` does not require a manual call to `.wait()`.
        Ok(())
    }
}
