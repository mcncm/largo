use std::collections::BTreeMap;
use std::ffi::OsStr;

use anyhow::{anyhow, Result};

use typedir::{Extend, PathBuf as P, PathRef as R};

use crate::conf::{self, LargoConfig};
use crate::dirs;
use crate::project::{self, Dependencies, Project, ProjectSettings, SystemSettings};

struct TexInput(String);

impl AsRef<OsStr> for TexInput {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

/// Variables available at TeX run time
// FIXME: this implementation is very, very suboptimal. It's particularly bad
// for documentation for the keys to be dynamic.
struct LargoVars<'a>(std::collections::BTreeMap<&'static str, &'a str>);

impl<'a> LargoVars<'a> {
    fn new(profile_name: &'a project::ProfileName, conf: &'a LargoConfig) -> Self {
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("Profile", profile_name.as_ref());
        if let Some(bib) = conf.default_bibliography.as_ref() {
            vars.insert("Biblio", bib);
        }
        Self(vars)
    }

    fn to_defs(self) -> String {
        use std::fmt::Write;
        let mut defs = String::new();
        for (k, v) in self.0.into_iter() {
            write!(&mut defs, r#"\def\Largo{k}{{{v}}}"#).unwrap();
        }
        defs
    }
}

// TODO Other TeX vars: `\X:OUTPUTDIR`
fn tex_input(profile_name: &project::ProfileName, conf: &LargoConfig) -> TexInput {
    let vars = LargoVars::new(profile_name, conf);
    let vars = vars.to_defs();
    let main_file = dirs::proj::MAIN_FILE;
    TexInput(format!(r#"{vars}\input{{{main_file}}}"#))
}

/// Environment variables for the build command
#[derive(Debug, Default)]
struct BuildVars(BTreeMap<&'static str, String>);

impl BuildVars {
    fn new() -> Self {
        Self(BTreeMap::new())
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
            self.0.insert("TEXINPUTS", tex_inputs);
        }
        self
    }
}

pub struct BuildBuilder<'a> {
    conf: &'a LargoConfig,
    project: Project,
    /// Which profile to build in
    profile_name: Option<&'a crate::project::ProfileName>,
}

impl<'a> BuildBuilder<'a> {
    pub fn new(conf: &'a LargoConfig, project: Project) -> Self {
        Self {
            conf,
            project,
            profile_name: None,
        }
    }

    pub fn with_profile_name(mut self, name: &'a Option<crate::project::ProfileName>) -> Self {
        self.profile_name = name.as_ref();
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
    pub conf: &'a LargoConfig,
    pub root_dir: P<dirs::proj::RootDir>,
    pub profile_name: &'a project::ProfileName,
    pub system_settings: SystemSettings,
    pub project_settings: ProjectSettings,
    pub dependencies: Dependencies,
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
        BuildVars::new().with_dependencies(&self.dependencies)
    }

    fn build_command(mut self) -> std::process::Command {
        let tex_input = tex_input(&self.profile_name, self.conf);
        let build_vars = self.build_vars();
        let mut cmd = std::process::Command::new(self.executable());
        {
            let src_dir: R<dirs::proj::SrcDir> = (&mut self.root_dir).extend(());
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
        use clam::Options;
        pdflatex_options.apply(&mut cmd);
        let build_dir: P<dirs::proj::ProfileBuildDir> =
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
        self.shell_cmd.output()?;
        Ok(())
    }
}
