use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;

use xargo::{conf::XargoConfig, dirs, project, tex::*};

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
    DebugXargo,
    #[cfg(debug_assertions)]
    DebugProject,
}

#[derive(Parser)]
struct InitSubcommand {
    name: String,
    /// Create a (La)TeX package if passing the `--package` flag
    #[arg(long)]
    package: bool,
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
                format: self.system,
                engine: self.engine,
            },
            profile: default_profiles,
            dependencies: HashMap::new(),
        }
    }
}

impl InitSubcommand {
    /// Only call in project directory
    fn execute(&self, _conf: &XargoConfig) -> Result<()> {
        use std::io::Write;
        // Prepare the project config file
        let project_toml = self.project_toml();
        let project_toml = toml::ser::to_vec(&project_toml)?;
        let mut toml = std::fs::File::create(dirs::proj::CONFIG_FILE)?;
        toml.write_all(&project_toml)?;
        // Prepare the source directory
        std::fs::create_dir(dirs::proj::SRC_DIR)?;
        // Create the `main.tex` file
        let mut main = std::fs::File::create("src/main.tex")?;
        main.write_all(match self.system {
            TexFormat::Tex => include_bytes!("files/main_tex.tex"),
            TexFormat::Latex => include_bytes!("files/main_latex.tex"),
        })?;
        let mut gitignore = std::fs::File::create(".gitignore")?;
        gitignore.write_all(include_bytes!("files/gitignore.txt"))?;
        // Prepare the build directory
        std::fs::create_dir(dirs::proj::BUILD_DIR)?;
        Ok(())
    }
}

fn new_project(init_cmd: &InitSubcommand, conf: &XargoConfig) -> Result<()> {
    // Create the project directory
    std::fs::create_dir(&init_cmd.name)?;
    std::env::set_current_dir(&init_cmd.name)?;
    init_cmd.execute(conf)
}

impl Subcommand {
    fn execute(&self, conf: &XargoConfig) -> Result<()> {
        match &self {
            Subcommand::New(init_cmd) => Ok(new_project(&init_cmd, conf)?),
            Subcommand::Init(init_cmd) => Ok(init_cmd.execute(conf)?),
            Subcommand::Build(build_cmd) => {
                let project = project::Project::find()?;
                let build_cmd = xargo::building::BuildCmd::new(&build_cmd.profile, &project, conf)?;
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
            Subcommand::DebugXargo => {
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
    let conf = XargoConfig::new()?;
    cli.command.execute(&conf)
}
