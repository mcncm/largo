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

    /// The default profile selected if no other profile is chosen.
    default_profile: String,
}

impl XargoConfig {
    fn new() -> Self {
        let mut builder = config::Config::builder()
            .set_default("default-profile", "debug")
            .unwrap();
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
    path.push(PROJ_CONFIG_FILE);
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
    ) -> (&'a str, &'a Profile) {
        let prof_name = self.profile.as_deref().unwrap_or(&conf.default_profile);
        let profile = proj.profile.get(prof_name).expect("Profile not found");
        (prof_name, profile)
    }

    fn tex_input(&self, prof_name: &str) -> String {
        format!(
            concat!(r#"\def\XPROFILE{{{}}}"#, r#"\input{{src/main.tex}}"#),
            prof_name
        )
    }

    fn to_command(&self, proj: &Project, conf: &XargoConfig) -> std::process::Command {
        let (prof_name, _profile) = self.choose_profile(&proj.config, conf);
        let program = conf.choose_program(proj.config.project.engine, proj.config.project.system);
        let mut cmd = std::process::Command::new(program);
        cmd.current_dir(&proj.root);
        cmd.args(["-output-directory", "target"]);
        cmd.arg(&self.tex_input(&prof_name));
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
