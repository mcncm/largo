use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};

const PROJ_CONFIG_FILE: &'static str = "xargo.toml";
const XARGO_CONFIG_DIR: &'static str = ".xargo";
const XARGO_CONFIG_FILE: &'static str = "config.toml";
const XARGO_CONFIG_DIR_FILE: &'static str = ".xargo/config.toml";

#[allow(dead_code)]
fn xargo_home() -> Option<PathBuf> {
    // This if/else chain should optimize away, it's guaranteed to be exhaustive
    // (unlike `#[cfg(...)]`), and rust-analyzer won't ignore the dead cases.
    if cfg!(target_family = "unix") {
        Some(PathBuf::from(std::env::var("HOME").ok()?))
    } else if cfg!(target_family = "windows") {
        Some(PathBuf::from(std::env::var("USERPROFILE").ok()?))
    } else {
        // The only other `target_family` at this time is `wasm`.
        None
    }
}

#[allow(dead_code)]
fn xargo_global_config_path() -> Option<PathBuf> {
    xargo_home().and_then(|mut path| {
        path.push(XARGO_CONFIG_DIR);
        path.push(XARGO_CONFIG_FILE);
        Some(path)
    })
}

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
}

#[allow(dead_code)]
// All the executable functions that you will probably want to generate with a macro
impl XargoConfig {
    fn tex_executable(&self) -> &str {
        self.tex_executable.as_deref().unwrap_or("tex")
    }

    fn latex_executable(&self) -> &str {
        self.latex_executable.as_deref().unwrap_or("latex")
    }

    fn pdftex_executable(&self) -> &str {
        self.pdftex_executable.as_deref().unwrap_or("pdftex")
    }

    fn pdflatex_executable(&self) -> &str {
        self.pdflatex_executable.as_deref().unwrap_or("pdflatex")
    }

    fn xetex_executable(&self) -> &str {
        self.xetex_executable.as_deref().unwrap_or("xetex")
    }

    fn xelatex_executable(&self) -> &str {
        self.xelatex_executable.as_deref().unwrap_or("xelatex")
    }

    fn luatex_executable(&self) -> &str {
        self.luatex_executable.as_deref().unwrap_or("luatex")
    }

    fn lualatex_executable(&self) -> &str {
        self.lualatex_executable.as_deref().unwrap_or("lualatex")
    }
}

impl XargoConfig {
    fn new() -> Self {
        let mut builder = config::Config::builder();
        // Use a *local* config as the primary source.
        if std::path::Path::new(XARGO_CONFIG_DIR_FILE).exists() {
            builder = builder.add_source(config::File::new(
                XARGO_CONFIG_DIR_FILE,
                config::FileFormat::Toml,
            ));
        }
        // Fall back on a *global* config
        if let Some(path) = xargo_global_config_path() {
            if path.exists() {
                builder = builder.add_source(config::File::new(
                    path.as_os_str()
                        .to_str()
                        .expect("global config file has some kind of non-UTF-8 path"),
                    config::FileFormat::Toml,
                ));
            }
        }
        builder
            .build()
            .expect("failed to build config")
            .try_deserialize()
            .expect("config error!")
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
    project: Project,
    profile: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Project {
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

fn do_in_root<T, F: Fn(&ProjectConfig) -> T>(f: F) -> T {
    let initial_path = std::env::current_dir().unwrap();
    let config_builder =
        config::Config::builder().add_source(config::File::new("", config::FileFormat::Toml));
    for ancestor in initial_path.ancestors() {
        std::env::set_current_dir(&ancestor).unwrap();
        match config_builder.build_cloned() {
            Ok(config) => {
                let config = config
                    .try_deserialize()
                    .expect("failed to deserialize config");
                return f(&config);
            }
            Err(e) => {
                eprintln!("{}", e);
                panic!("error building config");
            }
        }
    }
    unreachable!();
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
            project: Project {
                name: self.name.clone(),
                system: self.system,
                engine: self.engine,
            },
            profile: default_profiles,
        }
    }
}

impl InitSubcommand {
    /// Only call in project directory
    fn execute(&self, _conf: &XargoConfig) -> std::io::Result<()> {
        use std::io::Write;
        // Prepare the project config file
        let project_toml = self.project_toml();
        let project_toml = toml::ser::to_vec(&project_toml).expect("failed to serialize toml file");
        let mut toml = std::fs::File::create(PROJ_CONFIG_FILE)?;
        toml.write_all(&project_toml)?;
        // Prepare the source directory
        std::fs::create_dir("src")?;
        // Create the `main.tex` file
        let mut main = std::fs::File::create("src/main.tex")?;
        main.write_all(match self.system {
            TexFormat::Tex => include_bytes!("files/main_tex.tex"),
            TexFormat::Latex => include_bytes!("files/main_latex.tex"),
        })?;
        let mut gitignore = std::fs::File::create(".gitignore")?;
        gitignore.write_all(include_bytes!("files/gitignore.txt"))?;
        // Prepare the build directory
        std::fs::create_dir("target")?;
        Ok(())
    }
}

fn new_project(init_cmd: &InitSubcommand, conf: &XargoConfig) -> std::io::Result<()> {
    // Create the project directory
    std::fs::create_dir(&init_cmd.name)?;
    std::env::set_current_dir(&init_cmd.name)?;
    init_cmd.execute(conf)
}

fn build_command(conf: &ProjectConfig) -> std::process::Command {
    let cmd = match (&conf.project.engine, &conf.project.system) {
        (TexEngine::Tex, TexFormat::Tex) => "tex",
        (TexEngine::Tex, TexFormat::Latex) => "latex",
        (TexEngine::Pdftex, TexFormat::Tex) => "pdftex",
        (TexEngine::Pdftex, TexFormat::Latex) => "pdflatex",
        (TexEngine::Xetex, TexFormat::Tex) => "xetex",
        (TexEngine::Xetex, TexFormat::Latex) => "xelatex",
        (TexEngine::Luatex, TexFormat::Tex) => "luatex",
        (TexEngine::Luatex, TexFormat::Latex) => "lualatex",
    };
    let mut cmd = std::process::Command::new(cmd);
    let arg = String::from(r#""\input{main.tex}""#);
    cmd.arg(&arg);
    cmd
}

/// Only call in project directory
fn build_project(_build_cmd: &BuildSubcommand) -> impl Fn(&ProjectConfig) -> std::io::Result<()> {
    |conf: &ProjectConfig| {
        // Copy the source directory to `target/src`
        std::process::Command::new("cp")
            .args(["-r", "src/", "target/build/"])
            .output()
            .expect("failed to copy src dir");
        // Build the project
        let project_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir("target/build").unwrap();
        build_command(&conf).output().expect("build command failed");
        std::env::set_current_dir(&project_dir).unwrap();
        std::fs::copy("target/build/main.pdf", "target/main.pdf").expect("failed to copy target");
        Ok(())
    }
}

/// Only call in project directory
fn clean_project(_conf: &ProjectConfig) -> std::io::Result<()> {
    std::process::Command::new("rm")
        .args(["-rf", "target/*"])
        .output()?;
    Ok(())
}

impl Subcommand {
    fn execute(&self, conf: &XargoConfig) -> std::io::Result<()> {
        match &self {
            Subcommand::New(init_cmd) => new_project(&init_cmd, conf),
            Subcommand::Init(init_cmd) => init_cmd.execute(conf),
            Subcommand::Build(build_cmd) => do_in_root(build_project(build_cmd)),
            Subcommand::Clean => do_in_root(clean_project),
            Subcommand::DebugXargo => {
                println!("{:?}", conf);
                Ok(())
            }
            Subcommand::DebugProject => todo!(),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let conf = XargoConfig::new();
    cli.command.execute(&conf).unwrap();
}
