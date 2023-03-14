use anyhow::{anyhow, Result};

use typedir::{Extend, PathBuf as P};

use crate::conf::LargoConfig;
use crate::conf::{Dependencies, ProfileName, Project, ProjectSettings, SystemSettings};
use crate::dirs;
use crate::engines;
use crate::vars::LargoVars;

impl<'a> crate::vars::LargoVars<'a> {
    fn from_build_settings(settings: &'a BuildBuilderUnpacked<'a>) -> Self {
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
    fn try_finish_unpack(self) -> Result<BuildBuilderUnpacked<'a>> {
        use merge::Merge;
        let conf = self.conf;
        let project = self.project;
        let profile_name = self.profile.unwrap_or(self.conf.default_profile);
        let project_name = project.config.project.name;
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
        Ok(BuildBuilderUnpacked {
            conf,
            root_dir,
            src_dir,
            build_dir,
            project_name,
            profile_name,
            system_settings: proj_conf.system_settings,
            project_settings,
            dependencies,
            verbosity: self.verbosity,
        })
    }

    pub fn try_finish(self) -> Result<BuildRunner<'a>> {
        let unpacked = self.try_finish_unpack()?;
        unpacked.into_runner()
    }
}

/// An intermediate state of unpackaging and treating all the data we've
/// received
#[derive(Debug)]
struct BuildBuilderUnpacked<'a> {
    conf: &'a LargoConfig<'a>,
    root_dir: P<dirs::RootDir>,
    src_dir: P<dirs::SrcDir>,
    build_dir: P<dirs::BuildDir>,
    profile_name: ProfileName<'a>,
    project_name: &'a str,
    system_settings: SystemSettings,
    project_settings: ProjectSettings,
    dependencies: Dependencies<'a>,
    verbosity: Verbosity,
}

impl<'a> BuildBuilderUnpacked<'a> {
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

    fn into_ctx(self) -> BuildCtx<'a> {
        BuildCtx {
            root_dir: self.root_dir,
            src_dir: self.src_dir,
            build_dir: self.build_dir,
            profile_name: self.profile_name,
            project_name: self.project_name,
            verbosity: self.verbosity,
        }
    }

    fn into_runner(self) -> Result<BuildRunner<'a>> {
        let engine = self.get_engine()?;
        let ctx = self.into_ctx();
        Ok(BuildRunner { ctx, engine })
    }
}

#[derive(Debug)]
pub struct BuildCtx<'a> {
    root_dir: P<dirs::RootDir>,
    #[allow(unused)]
    src_dir: P<dirs::SrcDir>,
    #[allow(unused)]
    build_dir: P<dirs::BuildDir>,
    profile_name: ProfileName<'a>,
    project_name: &'a str,
    #[allow(unused)]
    verbosity: Verbosity,
}

// FIXME: this will incur a lot of unnecessary clones. Figure out the lifetimes
// and fix it!
#[derive(Debug)]
pub enum LargoInfo<'c> {
    Compiling {
        project: &'c str,
        version: Option<&'c str>,
        root: &'c std::path::Path,
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
pub enum BuildInfo<'c> {
    LargoInfo(LargoInfo<'c>),
    EngineInfo(crate::engines::EngineInfo),
}

impl<'c> From<LargoInfo<'c>> for BuildInfo<'c> {
    fn from(info: LargoInfo<'c>) -> Self {
        Self::LargoInfo(info)
    }
}

impl<'c> From<crate::engines::EngineInfo> for BuildInfo<'c> {
    fn from(info: crate::engines::EngineInfo) -> Self {
        Self::EngineInfo(info)
    }
}

#[derive(Debug)]
pub struct BuildRunner<'c> {
    ctx: BuildCtx<'c>,
    engine: engines::Engine,
}

enum BuildState {
    Init,
    EngineRunning(crate::engines::EngineOutput),
    Finished,
    Exit,
}

pub struct BuildOutput<'b> {
    ctx: &'b BuildCtx<'b>,
    engine: &'b mut engines::Engine,
    state: BuildState,
    start: std::time::Instant,
}

impl<'b> smol::stream::Stream for BuildOutput<'b> {
    type Item = Result<BuildInfo<'b>>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::task::Poll;
        match self.state {
            BuildState::Init => {
                let info = LargoInfo::Compiling {
                    project: &self.ctx.project_name,
                    version: None,
                    root: &self.ctx.root_dir,
                }
                .into();
                match self.engine.run() {
                    Result::Ok(engine_output) => {
                        self.state = BuildState::EngineRunning(engine_output);
                        Poll::Ready(Some(Ok(info)))
                    }
                    Result::Err(err) => Poll::Ready(Some(Err(err.into()))),
                }
            }
            BuildState::EngineRunning(ref mut engine_output) => {
                match smol::stream::StreamExt::poll_next(engine_output, cx) {
                    Poll::Ready(Some(engine_info)) => Poll::Ready(Some(Ok(engine_info.into()))),
                    Poll::Ready(None) => {
                        self.state = BuildState::Finished;
                        self.poll_next(cx)
                    }
                    Poll::Pending => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            BuildState::Finished => {
                self.state = BuildState::Exit;
                let duration = std::time::Instant::now() - self.start;
                Poll::Ready(Some(Ok(BuildInfo::LargoInfo(LargoInfo::Finished {
                    profile_name: self.ctx.profile_name,
                    duration,
                }))))
            }
            BuildState::Exit => Poll::Ready(None),
        }
    }
}

impl<'c> BuildRunner<'c> {
    pub async fn run<'a>(&'a mut self) -> BuildOutput {
        BuildOutput {
            ctx: &self.ctx,
            engine: &mut self.engine,
            state: BuildState::Init,
            start: std::time::Instant::now(),
        }
    }
}
