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
}

#[derive(Deserialize)]
struct Config {
    project: Project,
}

#[derive(Deserialize)]
struct Project {
    _name: String,
    build_command: String,
}

fn do_in_root<T, F: Fn(Config) -> T>(f: F) -> T {
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
                return f(config);
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

fn build_project(conf: Config) -> std::io::Result<()> {
    // Copy the source directory to `target/src`
    std::process::Command::new("cp")
        .args(["-r", "src/", "target/src/"])
        .output()
        .expect("failed to copy src dir");
    // Build the project
    let project_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir("target/src").unwrap();
    std::process::Command::new(&conf.project.build_command)
        .arg("main.tex")
        .output()
        .expect("build command failed");
    std::env::set_current_dir(&project_dir).unwrap();
    std::fs::copy("target/src/main.pdf", "target/main.pdf").expect("failed to copy target");
    Ok(())
}

fn execute(cmd: &Command) -> std::io::Result<()> {
    match cmd {
        Command::New { name } => new_project(&name),
        Command::Init { name } => init_project(&name),
        Command::Build => do_in_root(build_project),
    }
}

fn main() {
    let cli = Cli::parse();
    execute(&cli.command).unwrap();
}
