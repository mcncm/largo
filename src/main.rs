use anyhow::Result;
use clap::Parser;

use largo::{
    conf::{self, LargoConfig},
    dirs,
    options::*,
    project,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    #[command(flatten)]
    Create(CreateSubcommand),
    #[command(flatten)]
    Project(ProjectSubcommand),
    #[cfg(debug_assertions)]
    /// Print the Largo configuration
    DebugLargo,
}

#[derive(clap::Subcommand)]
enum CreateSubcommand {
    /// Initialize a largo project in the current directory
    Init(InitSubcommand),
    /// Create a largo project in a new directory
    New(InitSubcommand),
}

#[derive(clap::Subcommand)]
enum ProjectSubcommand {
    /// Build the current project
    Build(BuildSubcommand),
    /// Erase the build directory
    Clean,
    /// Generate a standalone TeX project
    Eject,
    #[cfg(debug_assertions)]
    /// Print the project configuration
    DebugProject,
    #[cfg(debug_assertions)]
    /// Print the build plan
    DebugBuild(BuildSubcommand),
}

#[derive(Parser)]
struct InitSubcommand {
    // TODO: should probably be a `PathBuf`
    name: String,
    /// Create a (La)TeX package if passing the `--package` flag.
    #[arg(long)]
    package: bool,
    /// Create a Beamer project. If the `--package` flag is passed, create an
    /// empty Beamer template.
    #[arg(long)]
    beamer: bool,
    #[arg(long, value_enum)]
    /// Overrides the default TeX format if set
    system: TexFormat,
    #[arg(long, value_enum)]
    /// Overrides the default TeX engine if set
    engine: TexEngine,
}

#[derive(Parser)]
struct BuildSubcommand {
    #[arg(short = 'p', long)]
    /// Overrides the default build profile if set
    profile: Option<String>,
}

impl InitSubcommand {
    fn project_toml(self) -> project::ProjectConfig {
        project::ProjectConfig {
            project: project::ProjectConfigHead {
                name: self.name.clone(),
                system_settings: project::SystemSettings::default(),
                project_settings: project::ProjectSettings::default(),
            },
            profiles: project::Profiles::new(),
            dependencies: project::Dependencies::new(),
        }
    }
}

impl InitSubcommand {
    /// Only call in project directory
    fn execute(self, path: std::path::PathBuf) -> Result<()> {
        let project_config = &self.project_toml();
        let new_project = dirs::proj::NewProject { project_config };
        dirs::proj::init(path, new_project)
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
    fn try_into_build(
        self,
        project: project::Project,
        conf: &LargoConfig,
    ) -> Result<largo::building::Build> {
        largo::building::BuildBuilder::new(conf, project)
            .with_profile_name(&self.profile)
            .try_finish()
    }
}

impl ProjectSubcommand {
    fn execute(self, project: project::Project, conf: &LargoConfig) -> Result<()> {
        use ProjectSubcommand::*;
        match self {
            Build(subcmd) => subcmd.try_into_build(project, conf)?.run(),
            // the `Project` is reasonable proof that it is a valid project:
            // the manifest file parses. It's *reasonably* safe to delete a
            // directory if `proj` is constructed.
            Clean => {
                let root = project.root;
                let build_dir = dirs::proj::BuildDir::from(root);
                std::fs::remove_dir_all(&build_dir.as_ref())?;
                std::fs::create_dir(&build_dir.as_ref())?;
                Ok(())
            }
            Eject => todo!(),
            DebugProject => {
                println!("{:#?}", project);
                Ok(())
            }
            #[cfg(debug_assertions)]
            DebugBuild(subcmd) => {
                let build = subcmd.try_into_build(project, conf)?;
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
            Subcommand::Project(subcmd) => {
                let project = project::Project::find()?;
                let conf = conf::LargoConfig::new()?;
                subcmd.execute(project, &conf)
            }
            #[cfg(debug_assertions)]
            Subcommand::DebugLargo => {
                let conf = conf::LargoConfig::new()?;
                println!("{:#?}", conf);
                Ok(())
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let res = cli.command.execute();
    if let Err(ref err) = res {
        println!("{:?}", err.backtrace());
    }
    res
}
