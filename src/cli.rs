use clap::{Parser, ValueEnum};

use largo_core::{build, conf, dirs, Result};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Subcommand,
    /// Print the parsed cli options and exit
    #[cfg(debug_assertions)]
    #[arg(long)]
    debug: bool,
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
    #[command(flatten)]
    Create(CreateSubcommand),
    #[command(flatten)]
    Project(ProjectSubcommand),
    #[cfg(debug_assertions)]
    /// Print the Largo configuration
    DebugLargo,
}

#[derive(Debug, clap::Subcommand)]
enum CreateSubcommand {
    /// Initialize a largo project in the current directory
    Init(InitSubcommand),
    /// Create a largo project in a new directory
    New(InitSubcommand),
}

#[derive(Debug, clap::Subcommand)]
enum ProjectSubcommand {
    /// Build the current project
    Build(BuildSubcommand),
    /// Erase the build directory
    Clean {
        #[arg(long)]
        profile: Option<String>,
    },
    /// Generate a standalone TeX project
    Eject,
    #[cfg(debug_assertions)]
    /// Print the project configuration
    DebugProject,
    // This subcommand only exists in debug builds
    #[cfg(debug_assertions)]
    /// Print the build plan
    DebugBuild(BuildSubcommand),
}

#[derive(Debug, Clone, ValueEnum)]
enum TexFormat {
    Tex,
    Latex,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TexEngine {
    Tex,
    Pdftex,
    Xetex,
    Luatex,
}

#[derive(Debug, Parser)]
#[clap(group(
    clap::ArgGroup::new("type")
        .multiple(false)
        .args(&["package", "class"])
        .conflicts_with("doc")
))]
struct InitSubcommand {
    // TODO: should probably be a `PathBuf`
    name: String,
    /// Create a (La)TeX package.
    #[arg(long)]
    package: bool,
    /// Create a (La)TeX class.
    #[arg(long)]
    class: bool,
    /// Create a (La)TeX document.
    #[arg(
        long,
        default_value_t = true,
        default_value_if("package", "true", "false"),
        default_value_if("class", "true", "false")
    )]
    doc: bool,
    /// Create a Beamer project. If the `--package` flag is passed, create an
    /// empty Beamer template.
    #[clap(skip)]
    _beamer: bool,
    #[arg(long, value_enum)]
    /// Overrides the default TeX format if set
    system: Option<TexFormat>,
    #[arg(long, value_enum)]
    /// Overrides the default TeX engine if set
    engine: Option<TexEngine>,
}

#[derive(Debug, Parser)]
struct BuildSubcommand {
    #[arg(short = 'p', long)]
    /// Overrides the default build profile if set
    profile: Option<String>,
    /// Print output from TeX engine
    #[arg(short = 'v', long)]
    verbose: bool,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        // This option only exists in debug builds
        #[cfg(debug_assertions)]
        if self.debug {
            println!("{:#?}", self);
            return Ok(());
        }
        self.command.execute()
    }
}

impl InitSubcommand {
    fn project_kind(&self) -> dirs::ProjectKind {
        use dirs::ProjectKind::*;
        if self.doc {
            Document
        } else if self.package {
            Package
        } else if self.class {
            Class
        } else {
            unreachable!()
        }
    }

    fn execute(self, path: std::path::PathBuf) -> Result<()> {
        let new_project = dirs::NewProject {
            name: self.name.as_str(),
            kind: self.project_kind(),
        };
        new_project.init(path)
    }
}

impl CreateSubcommand {
    fn execute(self) -> Result<()> {
        match self {
            CreateSubcommand::Init(subcmd) => {
                let path = std::env::current_dir().unwrap();
                subcmd.execute(path)
            }
            CreateSubcommand::New(subcmd) => {
                std::fs::create_dir(&subcmd.name)?;
                // FIXME This unnecessary clone is an artifact of these commands
                // not being factored quite right
                let name = subcmd.name.clone();
                subcmd.execute(std::path::PathBuf::from(name))
            }
        }
    }
}

impl BuildSubcommand {
    fn try_to_build<'c>(
        &'c self,
        project: conf::Project<'c>,
        conf: &'c conf::LargoConfig,
    ) -> Result<build::BuildRunner<'c>> {
        let profile = match &self.profile {
            Some(p) => Some(p.as_str().try_into()?),
            None => None,
        };
        let verbosity = if self.verbose {
            build::Verbosity::Noisy
        } else {
            build::Verbosity::Silent
        };
        build::BuildBuilder::new(conf, project)
            .with_profile(profile)
            .with_verbosity(verbosity)
            .try_finish()
    }
}

// Wrapper structs for info from core
struct BuildInfo<'c>(largo_core::build::BuildInfo<'c>);
struct LargoInfo<'c>(&'c largo_core::build::LargoInfo<'c>);
struct EngineInfo<'c>(&'c largo_core::engines::EngineInfo);

impl<'c> std::fmt::Display for BuildInfo<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            build::BuildInfo::LargoInfo(info) => LargoInfo(info).fmt(f),
            build::BuildInfo::EngineInfo(info) => EngineInfo(info).fmt(f),
        }
    }
}

impl<'c> LargoInfo<'c> {
    fn info_name(&self) -> &str {
        use build::LargoInfo::*;
        match &self.0 {
            Compiling { .. } => "Compiling",
            Running { .. } => "Running",
            Finished { .. } => "Finished",
        }
    }
}

impl<'c> std::fmt::Display for LargoInfo<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use build::LargoInfo::*;
        let info = &self.0;
        write!(f, "{: >12} ", self.info_name())?;
        match info {
            Compiling {
                project,
                version: _,
                root,
            } => write!(f, "{} ({})", project, root.display()),
            Running { exec } => write!(f, "{}", exec,),
            Finished {
                profile_name,
                duration,
            } => write!(f, "`{}` in {:.2}s", profile_name, duration.as_secs_f32()),
        }
    }
}

impl<'c> std::fmt::Display for EngineInfo<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use largo_core::engines::EngineInfo;
        match &self.0 {
            EngineInfo::Error { line, msg } => write!(f, "error [{}]: {}", line, msg),
        }
    }
}

impl ProjectSubcommand {
    fn execute(&self, project: conf::Project, conf: &conf::LargoConfig) -> Result<()> {
        use ProjectSubcommand::*;
        match self {
            Build(subcmd) => {
                // Run this inside an async runtime
                smol::block_on(async {
                    use smol::stream::StreamExt;
                    let mut build_runner = subcmd.try_to_build(project, conf)?;
                    let mut build_info = build_runner.run().await;
                    while let Some(info) = build_info.next().await {
                        println!("{}", BuildInfo(info?));
                    }
                    Ok::<(), largo_core::Error>(())
                })
            }
            // the `Project` is (reasonable) proof that it is a valid project:
            // the manifest file parses. It's *reasonably* safe to delete a
            // directory if `proj` is constructed.
            Clean { profile } => {
                let root = project.root;
                let build_dir = typedir::path!(root => dirs::TargetDir);
                match &profile {
                    Some(profile) => {
                        let profile: largo_core::conf::ProfileName = profile.as_str().try_into()?;
                        use typedir::Extend;
                        let profile_dir: typedir::PathBuf<dirs::ProfileTargetDir> =
                            build_dir.extend(&profile);
                        dirs::remove_dir_all(&profile_dir)
                    }
                    None => dirs::remove_dir_all(&build_dir),
                }
            }
            Eject => todo!(),
            // This subcommand only exists in debug builds
            #[cfg(debug_assertions)]
            DebugProject => {
                println!("{:#?}", project);
                Ok(())
            }
            // This subcommand only exists in debug builds
            #[cfg(debug_assertions)]
            DebugBuild(subcmd) => {
                let build = subcmd.try_to_build(project, conf)?;
                println!("{:#?}", build);
                Ok(())
            }
        }
    }
}

impl Subcommand {
    fn execute(self) -> Result<()> {
        match self {
            Subcommand::Create(subcmd) => subcmd.execute(),
            Subcommand::Project(subcmd) => conf::with_config(|conf, proj| match proj {
                Some(proj) => subcmd.execute(proj, conf),
                None => Err(anyhow::anyhow!("no enclosing project found")),
            })?,
            // This subcommand only exists in debug builds
            #[cfg(debug_assertions)]
            Subcommand::DebugLargo => conf::with_config(|conf, _| {
                println!("{:#?}", conf);
            }),
        }
    }
}
