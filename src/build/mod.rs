use anyhow::{anyhow, Result};
pub use smol::process::Command;
use std::collections::BTreeMap;

use typedir::{Extend, PathBuf as P};

use crate::conf::LargoConfig;
use crate::dirs;
use crate::project::{self, Dependencies, ProfileName, Project, ProjectSettings, SystemSettings};
use crate::vars::LargoVars;

mod engines;

impl<'a> crate::vars::LargoVars<'a> {
    fn from_build_settings(settings: &'a BuildSettings<'a>) -> Self {
        // NOTE: unfortunate clone
        let root_dir = settings.root_dir.clone();
        Self {
            profile: settings.profile_name,
            bibliography: settings.conf.default_bibliography,
            output_directory: root_dir.extend(()).extend(&settings.profile_name),
        }
    }
}

/// Environment variables for the build command
#[derive(Debug, Default)]
pub struct BuildVars(BTreeMap<&'static str, String>);

#[allow(dead_code)]
impl BuildVars {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    fn insert<V: std::fmt::Display>(&mut self, k: &'static str, v: V) {
        self.0.insert(k, format!("{}", v));
    }
}

#[allow(dead_code)]
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
        let _dependencies = project.config.dependencies;
        Ok(BuildSettings {
            conf,
            root_dir,
            profile_name,
            system_settings: proj_conf.system_settings,
            project_settings,
            _dependencies,
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
    _dependencies: Dependencies<'a>,
    verbosity: Verbosity,
}

impl<'a> BuildSettings<'a> {
    fn engine_builder(&self) -> engines::pdflatex::PdflatexBuilder {
        let tex_engine = &self.system_settings.tex_engine;
        let tex_format = &self.system_settings.tex_format;
        match (tex_engine, tex_format) {
            (crate::conf::TexEngine::Pdftex, crate::conf::TexFormat::Latex) => {
                engines::pdflatex::PdflatexBuilder::new(&self.conf)
            }
            (_, _) => {
                unimplemented!();
            }
        }
    }

    fn get_engine(&self) -> Result<engines::Engine> {
        use engines::EngineBuilder;
        let mut root_dir = self.root_dir.clone();
        let build_dir: P<dirs::ProfileBuildDir> =
            self.root_dir.clone().extend(()).extend(&self.profile_name);
        // FIXME this should happen *at build time*, right?
        std::fs::create_dir_all(&build_dir).expect("TODO: Sorry, this code needs to be refactored; it's a waste of time to handle this error.");
        let largo_vars = LargoVars::from_build_settings(&self);
        let eng = self
            .engine_builder()
            .with_src_dir((&mut root_dir).extend(()))
            .with_output_dir(build_dir)
            .with_verbosity(&self.verbosity)
            .with_largo_vars(&largo_vars)?
            .with_synctex(self.project_settings.synctex)?
            .with_shell_escape(self.project_settings.shell_escape)?
            .finish();
        Ok(eng)
    }

    fn to_build(self) -> Result<Build> {
        let engine = self.get_engine()?;
        Ok(Build { engine })
    }
}

#[derive(Debug)]
pub struct Build {
    // FIXME this absolutely should not be public
    pub engine: engines::Engine,
}

impl Build {
    pub async fn run(mut self) -> Result<()> {
        self.engine.run().await?;
        Ok(())
    }
}
