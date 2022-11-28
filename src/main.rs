use clap::{Parser, Subcommand};
use serde::Deserialize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    New { name: String },
    Init { name: String },
    Build,
    Clean,
}

#[derive(Deserialize)]
struct Config {
    project: Project,
}

// Must allow dead code because we want to deserialize e.g. `name`, not `_name`,
// even if it isn't used.
#[allow(dead_code)]
#[derive(Deserialize)]
struct Project {
    name: String,
    build_command: String,
}

fn do_in_root<T, F: Fn(&Config) -> T>(f: F) -> T {
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

fn init_project(_name: &String) -> std::io::Result<()> {
    use std::io::Write;
    // Prepare the project config file
    let mut toml = std::fs::File::create("xargo.toml")?;
    toml.write_all(include_bytes!("files/xargo.toml"))?;
    // Prepare the source directory
    std::fs::create_dir("src")?;
    let mut main = std::fs::File::create("src/main.tex")?;
    main.write_all(include_bytes!("files/main.tex"))?;
    let mut preamble = std::fs::File::create("src/preamble.tex")?;
    preamble.write_all(include_bytes!("files/preamble.tex"))?;
    let mut gitignore = std::fs::File::create(".gitignore")?;
    gitignore.write_all(include_bytes!("files/gitignore.txt"))?;
    // Prepare the build directory
    std::fs::create_dir("target")?;
    Ok(())
}

fn new_project(name: &String) -> std::io::Result<()> {
    // Create the project directory
    std::fs::create_dir(&name)?;
    std::env::set_current_dir(&name)?;
    init_project(&name)
}

fn build_command(conf: &Config) -> std::process::Command {
    let mut cmd = std::process::Command::new(&conf.project.build_command);
    let arg = String::from(r#""\input{main.tex}""#);
    cmd.arg(&arg);
    cmd
}

fn build_project(conf: &Config) -> std::io::Result<()> {
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

fn clean_project(_conf: &Config) -> std::io::Result<()> {
    std::process::Command::new("rm")
        .args(["-rf", "target/*"])
        .output()?;
    Ok(())
}

fn execute(cmd: &Command) -> std::io::Result<()> {
    match cmd {
        Command::New { name } => new_project(&name),
        Command::Init { name } => init_project(&name),
        Command::Build => do_in_root(build_project),
        Command::Clean => do_in_root(clean_project),
    }
}

fn main() {
    let cli = Cli::parse();
    execute(&cli.command).unwrap();
}
