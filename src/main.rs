use std::collections::HashMap;

use anyhow::{anyhow, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct XargoConfig {
    tex_executable: Option<String>,
    latex_executable: Option<String>,
    pdftex_executable: Option<String>,
    pdflatex_executable: Option<String>,
    xetex_executable: Option<String>,
    xelatex_executable: Option<String>,
    luatex_executable: Option<String>,
    lualatex_executable: Option<String>,

    /// The default profile selected if no other profile is chosen.
    default_profile: String,
}

impl XargoConfig {
    fn new() -> Result<Self> {
        let mut builder = config::Config::builder()
            .set_default("default-profile", "debug")
            .unwrap();

        // TODO: project-local config override
        // // FIXME: race condition!
        // if config_dir.as_ref().exists() {
        //     // Use a *local* config as the primary source.
        //     builder = builder.add_source(config_dir::ConfigFileSource::try_from(&config_file)?);
        // }

        let config_dir = config_dir::ConfigDir::global_config()?;
        let config_file = config_dir::ConfigFile::from(config_dir);
        // Fall back on a *global* config
        builder = builder.add_source(config_dir::ConfigFileSource::try_from(&config_file)?);
        Ok(builder.build()?.try_deserialize()?)
    }

    fn choose_program(&self, engine: TexEngine, format: TexFormat) -> &str {
        match (engine, format) {
            (TexEngine::Tex, TexFormat::Tex) => self.tex_executable.as_deref().unwrap_or("tex"),
            (TexEngine::Tex, TexFormat::Latex) => {
                self.latex_executable.as_deref().unwrap_or("latex")
            }
            (TexEngine::Pdftex, TexFormat::Tex) => {
                self.pdftex_executable.as_deref().unwrap_or("pdftex")
            }
            (TexEngine::Pdftex, TexFormat::Latex) => {
                self.pdflatex_executable.as_deref().unwrap_or("pdflatex")
            }
            (TexEngine::Xetex, TexFormat::Tex) => {
                self.xetex_executable.as_deref().unwrap_or("xetex")
            }
            (TexEngine::Xetex, TexFormat::Latex) => {
                self.xelatex_executable.as_deref().unwrap_or("xelatex")
            }
            (TexEngine::Luatex, TexFormat::Tex) => {
                self.luatex_executable.as_deref().unwrap_or("luatex")
            }
            (TexEngine::Luatex, TexFormat::Latex) => {
                self.lualatex_executable.as_deref().unwrap_or("lualatex")
            }
        }
    }
}

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
    DebugXargo,
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

/// The document preparation systems that can be used by a package.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
enum TexFormat {
    Tex,
    Latex,
}

/// The document preparation systems that can be used by a package.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
enum TexEngine {
    Tex,
    Pdftex,
    Xetex,
    Luatex,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProjectConfig {
    project: ProjectConfigGeneral,
    profile: HashMap<String, Profile>,
    dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProjectConfigGeneral {
    name: String,
    system: TexFormat,
    engine: TexEngine,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Profile {
    output_format: OutputFormat,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum OutputFormat {
    Dvi,
    Ps,
    Pdf,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Dependency {
    Path { path: String },
}

mod config_dir {
    use anyhow::{anyhow, Error, Result};
    use std::path::PathBuf;

    const CONFIG_DIR: &'static str = ".xargo";
    const CONFIG_FILE: &'static str = "config.toml";

    // The directory that the
    #[allow(dead_code)]
    fn home_directory() -> Result<PathBuf> {
        // This if/else chain should optimize away, it's guaranteed to be exhaustive
        // (unlike `#[cfg(...)]`), and rust-analyzer won't ignore the dead cases.
        if cfg!(target_family = "unix") {
            Ok(PathBuf::from(std::env::var("HOME")?))
        } else if cfg!(target_family = "windows") {
            Ok(PathBuf::from(std::env::var("USERPROFILE")?))
        } else {
            // The only other `target_family` at this time is `wasm`.
            unreachable!("target unsupported");
        }
    }

    typedir::typedir! {
        node ConfigDir {
            CONFIG_FILE => node ConfigFile;
        };
    }

    impl ConfigDir {
        #[allow(dead_code)]
        pub fn global_config() -> Result<Self> {
            let mut path = home_directory()?;
            path.push(CONFIG_DIR);
            Ok(Self(path))
        }
    }

    #[derive(Debug)]
    pub struct ConfigFileSource(config::File<config::FileSourceFile, config::FileFormat>);

    impl<'a> TryFrom<&'a ConfigFile> for ConfigFileSource {
        type Error = Error;

        fn try_from(path: &'a ConfigFile) -> Result<Self> {
            let source = config::File::new(
                path.as_ref()
                    .to_str()
                    .ok_or(anyhow!("failed to convert config file path to string"))?,
                config::FileFormat::Toml,
            );
            Ok(Self(source))
        }
    }

    impl config::Source for ConfigFileSource {
        fn clone_into_box(&self) -> Box<dyn config::Source + Send + Sync> {
            self.0.clone_into_box()
        }

        fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
            Ok(self.0.collect()?)
        }
    }
}

mod proj_dir {
    use anyhow::{anyhow, Result};

    pub const SRC_DIR: &'static str = "src";
    pub const BUILD_DIR: &'static str = "build";
    pub const CONFIG_FILE: &'static str = "xargo.toml";
    pub const LOCK_FILE: &'static str = "Xargo.lock";

    typedir::typedir! {
        node RootDir {
            CONFIG_FILE => node ConfigFile;
            LOCK_FILE => node LockFile;
            SRC_DIR => node SrcDir;
            BUILD_DIR => node BuildDir;
        };
    }

    impl RootDir {
        pub fn find() -> Result<Self> {
            let mut path = std::env::current_dir().unwrap();
            let path_cpy = path.clone();
            loop {
                path.push(CONFIG_FILE);
                if path.exists() {
                    path.pop();
                    return Ok(RootDir(path));
                }
                path.pop();
                if !path.pop() {
                    break;
                }
            }
            Err(anyhow!(
                "failed to find project containing `{}`",
                path_cpy.display()
            ))
        }
    }
}

#[derive(Debug)]
struct Project {
    root: proj_dir::RootDir,
    config: ProjectConfig,
}

impl Project {
    fn find() -> Result<Self> {
        use typedir::SubDir;
        let root = proj_dir::RootDir::find()?;
        let path = proj_dir::ConfigFile::from(root);
        let conf: ProjectConfig = config::Config::builder()
            .add_source(config::File::new(
                path.as_ref()
                    .as_os_str()
                    .to_str()
                    .expect("non-UTF-8 path or something"),
                config::FileFormat::Toml,
            ))
            .build()?
            .try_deserialize()?;
        Ok(Self {
            root: path.parent(),
            config: conf,
        })
    }
}

impl InitSubcommand {
    fn project_toml(&self) -> ProjectConfig {
        let mut default_profiles = HashMap::new();
        default_profiles.insert(
            "debug".to_string(),
            Profile {
                output_format: OutputFormat::Pdf,
            },
        );
        default_profiles.insert(
            "release".to_string(),
            Profile {
                output_format: OutputFormat::Pdf,
            },
        );
        ProjectConfig {
            project: ProjectConfigGeneral {
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
        let mut toml = std::fs::File::create(proj_dir::CONFIG_FILE)?;
        toml.write_all(&project_toml)?;
        // Prepare the source directory
        std::fs::create_dir(proj_dir::SRC_DIR)?;
        // Create the `main.tex` file
        let mut main = std::fs::File::create("src/main.tex")?;
        main.write_all(match self.system {
            TexFormat::Tex => include_bytes!("files/main_tex.tex"),
            TexFormat::Latex => include_bytes!("files/main_latex.tex"),
        })?;
        let mut gitignore = std::fs::File::create(".gitignore")?;
        gitignore.write_all(include_bytes!("files/gitignore.txt"))?;
        // Prepare the build directory
        std::fs::create_dir(proj_dir::BUILD_DIR)?;
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
    path.push(proj_dir::CONFIG_FILE);
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
        proj: &'a ProjectConfig,
        conf: &'a XargoConfig,
    ) -> Result<(&'a str, &'a Profile)> {
        let prof_name = self.profile.as_deref().unwrap_or(&conf.default_profile);
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

    fn envvars(&self, proj: &ProjectConfig) -> HashMap<&'static str, String> {
        let mut vars = HashMap::new();

        let mut tex_inputs = String::new();
        for (_dep_name, dep_body) in &proj.dependencies {
            match &dep_body {
                Dependency::Path { path } => {
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

    fn to_command(&self, proj: &Project, conf: &XargoConfig) -> Result<std::process::Command> {
        let (prof_name, _profile) = self.choose_profile(&proj.config, conf)?;
        let program = conf.choose_program(proj.config.project.engine, proj.config.project.system);
        let envvars = self.envvars(&proj.config);
        let mut cmd = std::process::Command::new(program);
        for (var, val) in &envvars {
            cmd.env(var, val);
        }
        cmd.current_dir(&proj.root);
        cmd.args(["-output-directory", proj_dir::BUILD_DIR]);
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
                let project = Project::find()?;
                build_cmd.to_command(&project, conf)?.output()?;
                Ok(())
            }
            Subcommand::Clean => {
                let root = proj_dir::RootDir::find()?;
                assert!(project_structure_conformant(root.as_ref()));
                let build_dir = proj_dir::BuildDir::from(root);
                std::fs::remove_dir_all(&build_dir.as_ref())?;
                std::fs::create_dir(&build_dir.as_ref())?;
                Ok(())
            }
            Subcommand::DebugXargo => {
                println!("{:?}", conf);
                Ok(())
            }
            Subcommand::DebugProject => todo!(),
            Subcommand::Eject => todo!(),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conf = XargoConfig::new()?;
    cli.command.execute(&conf)
}
