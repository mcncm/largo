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
        // FIXME: race condition!
        if std::path::Path::new(XARGO_CONFIG_DIR_FILE).exists() {
            // Use a *local* config as the primary source.
            builder = builder.add_source(config::File::new(
                XARGO_CONFIG_DIR_FILE,
                config::FileFormat::Toml,
            ));
        }
        if let Some(path) = xargo_global_config_path() {
            // FIXME: race condition!
            if path.exists() {
                // Fall back on a *global* config
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
    project: ProjectConfigGeneral,
    profile: HashMap<String, Profile>,
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

fn find_project_root() -> Option<PathBuf> {
    let mut path = std::env::current_dir().unwrap();
    loop {
        path.push(PROJ_CONFIG_FILE);
        if path.exists() {
            path.pop();
            return Some(path);
        }
        path.pop();
        if !path.pop() {
            break;
        }
    }
    None
}

#[derive(Debug)]
struct Project {
    root: PathBuf,
    config: ProjectConfig,
}

impl Project {
    fn find_enclosing() -> Option<Self> {
        find_project_root().and_then(|mut path| {
            path.push(PROJ_CONFIG_FILE);
            let conf: ProjectConfig = config::Config::builder()
                .add_source(config::File::new(
                    path.as_os_str()
                        .to_str()
                        .expect("non-UTF-8 path or something"),
                    config::FileFormat::Toml,
                ))
                .build()
                .expect("failed to build project config")
                .try_deserialize()
                .expect("failed to deserialize project config");
            path.pop();
            Some(Self {
                root: path,
                config: conf,
            })
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

/// Check that a path is a directory that conforms with the layout of a xargo
/// project.
fn project_structure_conformant(path: &std::path::Path) -> bool {
    // Ugh, lousy allocation.
    let mut path = path.to_owned();
    path.push(XARGO_CONFIG_FILE);
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
    fn to_command(&self, proj: &Project, conf: &XargoConfig) -> std::process::Command {
        let program = match (&proj.config.project.engine, &proj.config.project.system) {
            (TexEngine::Tex, TexFormat::Tex) => conf.tex_executable(),
            (TexEngine::Tex, TexFormat::Latex) => conf.latex_executable(),
            (TexEngine::Pdftex, TexFormat::Tex) => conf.pdftex_executable(),
            (TexEngine::Pdftex, TexFormat::Latex) => conf.pdflatex_executable(),
            (TexEngine::Xetex, TexFormat::Tex) => conf.xetex_executable(),
            (TexEngine::Xetex, TexFormat::Latex) => conf.xelatex_executable(),
            (TexEngine::Luatex, TexFormat::Tex) => conf.luatex_executable(),
            (TexEngine::Luatex, TexFormat::Latex) => conf.lualatex_executable(),
        };
        let mut cmd = std::process::Command::new(program);
        cmd.current_dir(&proj.root);
        cmd.args(["-output-directory", "target"]);
        let arg = String::from(r#""\input{main.tex}""#);
        cmd.arg(&arg);
        cmd
    }
}

impl Subcommand {
    fn execute(&self, conf: &XargoConfig) -> std::io::Result<()> {
        match &self {
            Subcommand::New(init_cmd) => new_project(&init_cmd, conf),
            Subcommand::Init(init_cmd) => init_cmd.execute(conf),
            Subcommand::Build(build_cmd) => {
                let project = Project::find_enclosing().expect("no enclosing project");
                build_cmd
                    .to_command(&project, conf)
                    .output()
                    .expect("failed to copy src dir");
                Ok(())
            }
            Subcommand::Clean => {
                let mut path = find_project_root().expect("no enclosing project");
                assert!(project_structure_conformant(&path));
                path.push("target");
                std::fs::remove_dir_all(&path).expect("failed to unlink target directory");
                std::fs::create_dir(&path).expect("failed to create new target directory");
                Ok(())
            }
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
