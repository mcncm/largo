use std::collections::HashMap;

use clap::Parser;
use serde::{Deserialize, Serialize};

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
#[derive(clap::ValueEnum, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum TexFormat {
    Tex,
    Latex,
}

/// The document preparation systems that can be used by a package.
#[derive(clap::ValueEnum, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum TexEngine {
    Tex,
    Pdftex,
    Xetex,
    Luatex,
}

#[derive(Deserialize, Serialize)]
struct ProjectToml {
    project: Project,
    profile: HashMap<String, Profile>,
}

#[derive(Deserialize, Serialize)]
struct Project {
    name: String,
    system: TexFormat,
    engine: TexEngine,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Profile {
    output_format: OutputFormat,
}

#[derive(Deserialize, Serialize)]
enum OutputFormat {
    Dvi,
    Ps,
    Pdf,
}

fn do_in_root<T, F: Fn(&ProjectToml) -> T>(f: F) -> T {
    let initial_path = std::env::current_dir().unwrap();
    let config_builder = config::Config::builder()
        .add_source(config::File::new("xargo.toml", config::FileFormat::Toml));
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
    fn project_toml(&self) -> ProjectToml {
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
        ProjectToml {
            project: Project {
                name: self.name.clone(),
                system: self.system,
                engine: self.engine,
            },
            profile: default_profiles,
        }
    }
}

/// Only call in project directory
fn init_project(init_cmd: &InitSubcommand) -> std::io::Result<()> {
    use std::io::Write;
    // Prepare the project config file
    let project_toml = init_cmd.project_toml();
    let project_toml = toml::ser::to_vec(&project_toml).expect("failed to serialize toml file");
    let mut toml = std::fs::File::create("xargo.toml")?;
    toml.write_all(&project_toml)?;
    // Prepare the source directory
    std::fs::create_dir("src")?;
    // Create the `main.tex` file
    let mut main = std::fs::File::create("src/main.tex")?;
    main.write_all(match init_cmd.system {
        TexFormat::Tex => include_bytes!("files/main_tex.tex"),
        TexFormat::Latex => include_bytes!("files/main_latex.tex"),
    })?;
    let mut gitignore = std::fs::File::create(".gitignore")?;
    gitignore.write_all(include_bytes!("files/gitignore.txt"))?;
    // Prepare the build directory
    std::fs::create_dir("target")?;
    Ok(())
}

fn new_project(init_cmd: &InitSubcommand) -> std::io::Result<()> {
    // Create the project directory
    std::fs::create_dir(&init_cmd.name)?;
    std::env::set_current_dir(&init_cmd.name)?;
    init_project(init_cmd)
}

fn build_command(conf: &ProjectToml) -> std::process::Command {
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

// /// Recursively copy a directory, only copying the files that are newer in the source directory.
// fn recursive_copy_update(
//     mut from: std::path::PathBuf,
//     mut to: std::path::PathBuf,
// ) -> std::io::Result<()> {
//     recursive_copy_update_inner(&mut from, &mut to)
// }

// fn recursive_copy_update_inner(
//     from: &mut std::path::PathBuf,
//     to: &mut std::path::PathBuf,
// ) -> std::io::Result<()> {
//     for dir_entry in std::fs::read_dir(&from)? {
//         let dir_entry = dir_entry?;
//         let from_metadata = dir_entry.metadata()?;
//         let from_file_type = from_metadata.file_type();
//         let name = dir_entry.file_name();
//         from.push(&name);
//         to.push(&name);
//         match std::fs::metadata(&to) {
//             Ok(_) => todo!(),
//             Err(_) => todo!(),
//         }
//         let to_file_type = to_metadata.file_type();
//         if from_file_type.is_symlink() {
//             unimplemented!("No symlink handling yet");
//         }
//         if from_file_type.is_file() {
//             if std::fs::try_exists(&to)? {}
//         }
//         if from_file_type.is_dir() {}
//         from.pop();
//         to.pop();
//     }
//     Ok(())
// }

/// Only call in project directory
fn build_project(_build_cmd: &BuildSubcommand) -> impl Fn(&ProjectToml) -> std::io::Result<()> {
    |conf: &ProjectToml| {
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
fn clean_project(_conf: &ProjectToml) -> std::io::Result<()> {
    std::process::Command::new("rm")
        .args(["-rf", "target/*"])
        .output()?;
    Ok(())
}

fn execute(cmd: &Subcommand) -> std::io::Result<()> {
    match cmd {
        Subcommand::New(init_cmd) => new_project(&init_cmd),
        Subcommand::Init(init_cmd) => init_project(&init_cmd),
        Subcommand::Build(build_cmd) => do_in_root(build_project(build_cmd)),
        Subcommand::Clean => do_in_root(clean_project),
    }
}

fn main() {
    let cli = Cli::parse();
    execute(&cli.command).unwrap();
}
