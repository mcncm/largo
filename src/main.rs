use std::collections::BTreeMap;

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
    DebugLargo,
}

#[derive(clap::Subcommand)]
enum CreateSubcommand {
    Init(InitSubcommand),
    New(InitSubcommand),
}

#[derive(clap::Subcommand)]
enum ProjectSubcommand {
    Build(BuildSubcommand),
    Clean,
    Eject,
    #[cfg(debug_assertions)]
    DebugProject,
    #[cfg(debug_assertions)]
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
    #[arg(long, value_enum, default_value_t = TexFormat::Latex)]
    system: TexFormat,
    #[arg(long, value_enum, default_value_t = TexEngine::Pdftex)]
    engine: TexEngine,
}

#[derive(Parser)]
struct BuildSubcommand {
    #[arg(short = 'F', long)]
    profile: Option<String>,
}

impl InitSubcommand {
    fn project_toml(&self) -> project::ProjectConfig {
        project::ProjectConfig {
            project: project::ProjectConfigHead {
                name: self.name.clone(),
                system_settings: project::SystemSettings {
                    tex_format: None,
                    tex_engine: None,
                },
                project_settings: project::ProjectSettings::default(),
            },
            profiles: BTreeMap::new(),
            dependencies: BTreeMap::new(),
        }
    }
}

impl InitSubcommand {
    /// Only call in project directory
    fn execute(&self, path: std::path::PathBuf) -> Result<()> {
        let project_config = &self.project_toml();
        let new_project = dirs::proj::NewProject { project_config };
        dirs::proj::init(path, new_project)
    }
}

impl CreateSubcommand {
    fn execute(&self) -> Result<()> {
        match &self {
            CreateSubcommand::Init(subcmd) => {
                let path = std::env::current_dir().unwrap();
                subcmd.execute(path)
            }
            CreateSubcommand::New(subcmd) => {
                std::fs::create_dir(&subcmd.name)?;
                subcmd.execute(std::path::PathBuf::from(&subcmd.name))
            }
        }
    }
}

impl ProjectSubcommand {
    fn execute(&self, project: project::Project, conf: &LargoConfig) -> Result<()> {
        use ProjectSubcommand::*;
        match &self {
            Build(subcmd) => {
                let build_cmd = largo::building::BuildCmd::new(&subcmd.profile, project, conf)?;
                let mut shell_cmd: std::process::Command = build_cmd.into();
                shell_cmd.output()?;
                Ok(())
            }
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
            DebugBuild(subcmd) => {
                let build_cmd = largo::building::BuildCmd::new(&subcmd.profile, project, conf)?;
                let shell_cmd: std::process::Command = build_cmd.into();
                println!("{:#?}", shell_cmd);
                Ok(())
            }
        }
    }
}

impl Subcommand {
    fn execute(&self) -> Result<()> {
        match &self {
            Subcommand::Create(subcmd) => subcmd.execute(),
            Subcommand::Project(subcmd) => {
                let project = project::Project::find()?;
                let conf = conf::LargoConfig::new()?;
                subcmd.execute(project, &conf)
            }
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
    cli.command.execute()
}
