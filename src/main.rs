use std::collections::HashMap;

use anyhow::{anyhow, Result};
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
                system: self.system,
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

/// Check that a path is a directory that conforms with the layout of a xargo
/// project.
fn project_structure_conformant(path: &std::path::Path) -> bool {
    // Ugh, lousy allocation.
    let mut path = path.to_owned();
    path.push(dirs::proj::CONFIG_FILE);
    if !path.exists() {
        return false;
    }
    path.pop();
    path.push("src");
    if !path.exists() {
        return false;
    }
    path.pop();
    path.push("target");
    if !path.exists() {
        return false;
    }
    path.pop();
    true
}

impl BuildSubcommand {
    fn choose_profile<'a>(
        &'a self,
        proj: &'a project::ProjectConfig,
        conf: &'a XargoConfig,
    ) -> Result<(&'a str, &'a project::Profile)> {
        let prof_name = self.profile.as_deref().unwrap_or(conf.default_profile());
        let profile = proj
            .profile
            .get(prof_name)
            .ok_or_else(|| anyhow!("no profile found"))?;
        Ok((prof_name, profile))
    }

    fn tex_input(&self, prof_name: &str) -> String {
        format!(
            concat!(r#"\def\XPROFILE{{{}}}"#, r#"\input{{src/main.tex}}"#),
            prof_name
        )
    }

    fn envvars(&self, proj: &project::ProjectConfig) -> HashMap<&'static str, String> {
        let mut vars = HashMap::new();

        let mut tex_inputs = String::new();
        for (_dep_name, dep_body) in &proj.dependencies {
            match &dep_body {
                project::Dependency::Path { path } => {
                    tex_inputs += &path;
                    tex_inputs.push(':');
                }
            }
        }
        if !tex_inputs.is_empty() {
            vars.insert("TEXINPUTS", tex_inputs);
        }

        vars
    }

    fn to_command(
        &self,
        proj: &project::Project,
        conf: &XargoConfig,
    ) -> Result<std::process::Command> {
        let (prof_name, _profile) = self.choose_profile(&proj.config, conf)?;
        let program = conf.choose_program(proj.config.project.engine, proj.config.project.system);
        let envvars = self.envvars(&proj.config);
        let mut cmd = std::process::Command::new(program);
        for (var, val) in &envvars {
            cmd.env(var, val);
        }
        cmd.current_dir(&proj.root);
        cmd.args(["-output-directory", dirs::proj::BUILD_DIR]);
        cmd.arg(&self.tex_input(&prof_name));
        Ok(cmd)
    }
}

impl Subcommand {
    fn execute(&self, conf: &XargoConfig) -> Result<()> {
        match &self {
            Subcommand::New(init_cmd) => Ok(new_project(&init_cmd, conf)?),
            Subcommand::Init(init_cmd) => Ok(init_cmd.execute(conf)?),
            Subcommand::Build(build_cmd) => {
                let project = project::Project::find()?;
                build_cmd.to_command(&project, conf)?.output()?;
                Ok(())
            }
            Subcommand::Clean => {
                let root = dirs::proj::RootDir::find()?;
                assert!(project_structure_conformant(root.as_ref()));
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
