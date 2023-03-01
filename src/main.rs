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
    Clean {
        #[arg(long)]
        profile: Option<String>,
    },
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

#[derive(Parser)]
struct BuildSubcommand {
    #[arg(short = 'p', long)]
    /// Overrides the default build profile if set
    profile: Option<String>,
}

impl InitSubcommand {
    fn project_kind(&self) -> dirs::proj::ProjectKind {
        use dirs::proj::ProjectKind::*;
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
        let new_project = dirs::proj::NewProject {
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
    fn try_into_build(
        self,
        project: project::Project,
        conf: &LargoConfig,
    ) -> Result<largo::building::Build> {
        let profile = match self.profile {
            Some(p) => Some(p.try_into()?),
            None => None,
        };
        largo::building::BuildBuilder::new(conf, project)
            .with_profile_name(&profile)
            .try_finish()
    }
}

impl ProjectSubcommand {
    fn execute(self, project: project::Project, conf: &LargoConfig) -> Result<()> {
        use ProjectSubcommand::*;
        match self {
            Build(subcmd) => subcmd.try_into_build(project, conf)?.run(),
            // the `Project` is (reasonable) proof that it is a valid project:
            // the manifest file parses. It's *reasonably* safe to delete a
            // directory if `proj` is constructed.
            Clean { profile } => {
                let root = project.root;
                let build_dir = typedir::path!(root => dirs::proj::BuildDir);
                match profile {
                    Some(profile) => {
                        let profile: crate::project::ProfileName = profile.try_into()?;
                        use typedir::Extend;
                        let profile_dir: typedir::PathBuf<dirs::proj::ProfileBuildDir> =
                            build_dir.extend(&profile);
                        std::fs::remove_dir_all(&profile_dir)?;
                    }
                    None => {
                        std::fs::remove_dir_all(&build_dir)?;
                    }
                }
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
