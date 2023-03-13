use anyhow::{anyhow, Result};

use typedir::{Extend, PathBuf as P};

use crate::conf::LargoConfig;
use crate::conf::{Dependencies, ProfileName, Project, ProjectSettings, SystemSettings};
use crate::dirs;
use crate::engines;
use crate::vars::LargoVars;

impl<'a> crate::vars::LargoVars<'a> {
    fn from_build_settings(settings: &'a BuildCtx<'a>) -> Self {
        // NOTE: unfortunate clone
        let root_dir = settings.root_dir.clone();
        Self {
            profile: settings.profile_name,
            bibliography: settings.conf.bib.bibliography,
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
    profile: Option<crate::conf::ProfileName<'a>>,
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

    pub fn with_profile(mut self, name: Option<crate::conf::ProfileName<'a>>) -> Self {
        self.profile = name.as_ref().copied();
        self
    }

    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Unpack the data we've been passed into a more convenient shape
    fn finish(self) -> Result<BuildCtx<'a>> {
        use merge::Merge;
        let conf = self.conf;
        let project = self.project;
        let profile_name = self.profile.unwrap_or(self.conf.default_profile);
        let root_dir = project.root;
        let src_dir = root_dir.clone().extend(());
        let build_dir = root_dir.clone().extend(()).extend(&profile_name).extend(());
        // FIXME This is a bug: there should *always* be a default profile to select
        let mut profiles = project.config.profiles.unwrap_or_default();
        profiles.merge_left(crate::conf::Profiles::standard());
        let profile = profiles
            .select_profile(&profile_name)
            .ok_or_else(|| anyhow!("profile `{}` not found", profile_name))?;
        let proj_conf = project.config.project;
        let mut project_settings = proj_conf.project_settings;
        project_settings.merge_right(profile.project_settings);
        let dependencies = project.config.dependencies;
        Ok(BuildCtx {
            conf,
            root_dir,
            src_dir,
            build_dir,
            profile_name,
            system_settings: proj_conf.system_settings,
            project_settings,
            dependencies,
            verbosity: self.verbosity,
        })
    }

    pub fn try_finish(self) -> Result<BuildRunner<'a>> {
        let build_settings = self.finish()?;
        build_settings.to_build()
    }
}

/// An intermediate state of unpackaging and treating all the data we've
/// received
struct BuildCtx<'a> {
    conf: &'a LargoConfig<'a>,
    root_dir: P<dirs::RootDir>,
    src_dir: P<dirs::SrcDir>,
    build_dir: P<dirs::BuildDir>,
    profile_name: ProfileName<'a>,
    system_settings: SystemSettings,
    project_settings: ProjectSettings,
    dependencies: Dependencies<'a>,
    verbosity: Verbosity,
}

impl<'a> BuildCtx<'a> {
    fn engine_builder(&self) -> engines::pdflatex::PdflatexBuilder {
        let tex_engine = &self.system_settings.tex_engine;
        let tex_format = &self.system_settings.tex_format;
        match (tex_engine, tex_format) {
            (crate::conf::TexEngine::Pdftex, crate::conf::TexFormat::Latex) => {
                engines::pdflatex::PdflatexBuilder::new(self.conf)
            }
            (_, _) => {
                unimplemented!();
            }
        }
    }

    fn get_engine(&self) -> Result<engines::Engine> {
        use engines::EngineBuilder;
        // FIXME this should happen *at build time*, right?
        std::fs::create_dir_all(&self.build_dir).expect("TODO: Sorry, this code needs to be refactored; it's a waste of time to handle this error.");
        let largo_vars = LargoVars::from_build_settings(self);
        let eng = self
            .engine_builder()
            // Yes, these are extraneous clones. I want to be sure first what
            // lifetime the `Engine` should really have.
            .with_src_dir(self.src_dir.clone())
            .with_output_dir(self.build_dir.clone())
            .with_verbosity(&self.verbosity)
            .with_largo_vars(&largo_vars)?
            .with_synctex(self.project_settings.synctex.unwrap_or_default())?
            .with_shell_escape(self.project_settings.shell_escape)?
            .with_dependencies(&crate::dependencies::get_dependency_paths(
                &self.dependencies,
            ))
            .finish();
        Ok(eng)
    }

    fn to_build(self) -> Result<BuildRunner<'a>> {
        let engine = self.get_engine()?;
        Ok(BuildRunner {
            profile_name: self.profile_name,
            verbosity: self.verbosity,
            engine,
        })
    }
}

// FIXME: this will incur a lot of unnecessary clones. Figure out the lifetimes
// and fix it!
#[derive(Debug)]
pub enum BuildInfo<'c> {
    Compiling {
        project: String,
        version: Option<String>,
        root: std::path::PathBuf,
    },
    Running {
        exec: crate::conf::Executable<'c>,
    },
    Finished {
        profile_name: ProfileName<'c>,
        duration: std::time::Duration,
    },
}

#[derive(Debug)]
pub struct BuildRunner<'c> {
    profile_name: ProfileName<'c>,
    verbosity: Verbosity,
    engine: engines::Engine,
}

impl<'c> BuildRunner<'c> {
    pub async fn run(mut self) -> impl smol::stream::Stream<Item = BuildInfo<'c>> {
        let (_, dur) = crate::util::timed_async(|| async { self.run_engine().await }).await;
        smol::stream::once(BuildInfo::Finished {
            profile_name: self.profile_name,
            duration: dur,
        })
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
                if line.starts_with('!') {
                    println!("{}", line);
                }
            }
        }
        Ok(())
    }
}
