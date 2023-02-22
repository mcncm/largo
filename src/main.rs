use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;

use largo::{conf::LargoConfig, dirs, project, tex::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    New(InitSubcommand),
    Init(InitSubcommand),
    Build(BuildSubcommand),
    Clean,
    Eject,
    #[cfg(debug_assertions)]
    DebugLargo,
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
        let mut default_profiles = HashMap::new();
        default_profiles.insert(
            "debug".to_string(),
            project::Profile {
                output_format: OutputFormat::Pdf,
            },
        );
        default_profiles.insert(
            "release".to_string(),
            project::Profile {
                output_format: OutputFormat::Pdf,
            },
        );
        project::ProjectConfig {
            project: project::ProjectConfigGeneral {
                name: self.name.clone(),
                system: self.system,
                engine: self.engine,
                shell_escape: None,
            },
            profile: default_profiles,
            dependencies: HashMap::new(),
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

fn new_project(init_cmd: &InitSubcommand) -> Result<()> {
    // Create the project directory
    std::fs::create_dir(&init_cmd.name)?;
    init_cmd.execute(std::path::PathBuf::from(&init_cmd.name))
}

impl Subcommand {
    fn execute(&self, conf: &LargoConfig) -> Result<()> {
        match &self {
            Subcommand::New(init_cmd) => Ok(new_project(&init_cmd)?),
            Subcommand::Init(init_cmd) => {
                let path = std::env::current_dir().unwrap();
                Ok(init_cmd.execute(path)?)
            }
            Subcommand::Build(build_cmd) => {
                let project = project::Project::find()?;
                let build_cmd = largo::building::BuildCmd::new(&build_cmd.profile, &project, conf)?;
                let mut shell_cmd: std::process::Command = build_cmd.into();
                shell_cmd.output()?;
                Ok(())
            }
            Subcommand::Clean => {
                // Reasonable proof that this is a valid project: the manifest
                // file parses. It's *reasonably* safe to delete a directory if
                // `proj` is constructed.
                let proj = project::Project::find()?;
                let root = proj.root;
                let build_dir = dirs::proj::BuildDir::from(root);
                std::fs::remove_dir_all(&build_dir.as_ref())?;
                std::fs::create_dir(&build_dir.as_ref())?;
                Ok(())
            }
            #[cfg(debug_assertions)]
            Subcommand::DebugBuild(build_cmd) => {
                let project = project::Project::find()?;
                let build_cmd = largo::building::BuildCmd::new(&build_cmd.profile, &project, conf)?;
                let shell_cmd: std::process::Command = build_cmd.into();
                println!("{:#?}", shell_cmd);
                Ok(())
            }
            #[cfg(debug_assertions)]
            Subcommand::DebugLargo => {
                println!("{:#?}", conf);
                Ok(())
            }
            #[cfg(debug_assertions)]
            Subcommand::DebugProject => {
                let proj = project::Project::find()?;
                println!("{:#?}", proj);
                Ok(())
            }
            Subcommand::Eject => todo!(),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conf = LargoConfig::new()?;
    cli.command.execute(&conf)
}
