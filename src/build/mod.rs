use std::collections::BTreeMap;
use std::ffi::OsStr;

use anyhow::{anyhow, Result};

use thiserror::__private::PathAsDisplay;
use typedir::{Extend, PathBuf as P, PathRef as R};

use crate::conf::{self, LargoConfig};
use crate::dirs;
use crate::project::{self, Dependencies, ProfileName, Project, ProjectSettings, SystemSettings};

struct TexInput(String);

impl AsRef<OsStr> for TexInput {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

/// Variables available at TeX run time
#[derive(Debug)]
struct LargoVars<'a> {
    profile: &'a ProfileName,
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
            profile: &settings.profile_name,
            bibliography: settings.conf.default_bibliography,
            output_directory: root_dir.extend(()).extend(settings.profile_name),
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
struct BuildVars(BTreeMap<&'static str, String>);

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
    project: Project,
    verbosity: Verbosity,
    /// Which profile to build in
    profile_name: Option<&'a crate::project::ProfileName>,
}

impl<'a> BuildBuilder<'a> {
    pub fn new(conf: &'a LargoConfig, project: Project) -> Self {
        Self {
            conf,
            project,
            verbosity: Verbosity::Silent,
            profile_name: None,
        }
    }

    pub fn with_profile_name(mut self, name: &'a Option<crate::project::ProfileName>) -> Self {
        self.profile_name = name.as_ref();
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
        let profile_name = self.profile_name.unwrap_or(&self.conf.default_profile);
        // FIXME This is a bug: there should *always* be a default profile to select
        let profiles = project.config.profiles;
        let profile = profiles
            .select_profile(profile_name)
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
    profile_name: &'a ProfileName,
    system_settings: SystemSettings,
    project_settings: ProjectSettings,
    dependencies: Dependencies,
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

    fn build_command(mut self) -> std::process::Command {
        use std::process;
        let largo_vars = LargoVars::from_build_settings(&self);
        let tex_input = tex_input(largo_vars, self.conf);
        let build_vars = self.build_vars();
        let mut cmd = std::process::Command::new(self.executable());
        if !matches!(self.verbosity, Verbosity::Noisy) {
            cmd.stdout(process::Stdio::null());
        }
        match &self.verbosity {
            Verbosity::Silent => {
                cmd.stdout(process::Stdio::null());
            }
            Verbosity::Info(_log_level) => {
                // What do we do here? Custom pipe?
                todo!();
            }
            Verbosity::Noisy => {
                // Don't have to do anything, inheriting stdout
            }
        }
        {
            let src_dir: R<dirs::SrcDir> = (&mut self.root_dir).extend(());
            cmd.current_dir(src_dir);
        }
        for (var, val) in build_vars.0 {
            cmd.env(var, val);
        }
        let mut pdflatex_options = crate::engines::pdflatex::CommandLineOptions::default();
        match self.project_settings.shell_escape {
            Some(true) => {
                pdflatex_options.shell_escape = true;
            }
            Some(false) => {
                pdflatex_options.no_shell_escape = true;
            }
            None => (),
        };
        // Always use nonstop mode for now.
        pdflatex_options.interaction = Some(crate::engines::pdflatex::InteractionMode::NonStopMode);
        use clam::Options;
        pdflatex_options.apply(&mut cmd);
        let build_dir: P<dirs::ProfileBuildDir> =
            self.root_dir.extend(()).extend(self.profile_name);
        std::fs::create_dir_all(&build_dir).expect("TODO: Sorry, this code needs to be refactored; it's a waste of time to handle this error.");
        match &self.project_settings.shell_escape {
            Some(true) => cmd.arg("-shell-escape"),
            Some(false) => cmd.arg("-no-shell-escape"),
            // Needed to make types match
            None => &mut cmd,
        }
        .args([
            "-output-directory",
            build_dir.to_str().expect("some kind of non-utf8 path"),
        ])
        .arg(&tex_input);
        cmd
    }

    fn to_build(self) -> Result<Build> {
        Ok(Build {
            shell_cmd: self.build_command(),
        })
    }
}

#[derive(Debug)]
pub struct Build {
    shell_cmd: std::process::Command,
}

impl Build {
    pub fn run(mut self) -> Result<()> {
        let mut child = self.shell_cmd.spawn()?;
        child.wait()?;
        Ok(())
    }
}
