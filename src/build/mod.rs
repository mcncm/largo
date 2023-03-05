use anyhow::{anyhow, Result};

use typedir::{Extend, PathBuf as P};

use crate::conf::LargoConfig;
use crate::dirs;
use crate::engines;
use crate::project::{Dependencies, ProfileName, Project, ProjectSettings, SystemSettings};
use crate::vars::LargoVars;

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

/// Level of severity of information to forward from TeX engine
#[derive(Debug, Default)]
pub enum LogLevel {
    #[default]
    Warning,
    Error,
}

#[derive(Debug, Default)]
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
    fn finish(self) -> Result<BuildSettings<'a>> {
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
        let dependencies = project.config.dependencies;
        Ok(BuildSettings {
            conf,
            root_dir,
            profile_name,
            system_settings: proj_conf.system_settings,
            project_settings,
            dependencies,
            verbosity: self.verbosity,
        })
    }

    pub fn try_finish(self) -> Result<Build> {
        let build_settings = self.finish()?;
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
            .with_dependencies(&crate::dependencies::get_dependency_paths(
                &self.dependencies,
            ))
            .finish();
        Ok(eng)
    }

    fn to_build(self) -> Result<Build> {
        let engine = self.get_engine()?;
        Ok(Build {
            verbosity: self.verbosity,
            engine,
        })
    }
}

#[derive(Debug)]
pub struct Build {
    verbosity: Verbosity,
    engine: engines::Engine,
}

impl Build {
    pub async fn run(mut self) -> Result<()> {
        let (_, dur) = crate::util::timed_async(|| async { self.run_engine().await }).await;
        println!("ran in {:.2}s.", dur.as_secs_f32());
        Ok(())
    }

    pub async fn run_engine(&mut self) -> Result<()> {
        use smol::prelude::*;
        let stdout = self.engine.run()?;
        let mut lines = stdout.lines();
        if matches!(self.verbosity, Verbosity::Noisy) {
            while let Some(line) = lines.next().await {
                println!("{}", line?);
            }
        } else {
            while let Some(line) = lines.next().await {
                let line = line?;
                if line.starts_with("!") {
                    println!("{}", line);
                }
            }
        }
        Ok(())
    }
}
