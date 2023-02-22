pub mod proj {
    use anyhow::{anyhow, Result};

    pub const SRC_DIR: &'static str = "src";
    pub const MAIN_FILE: &'static str = "main.tex";
    pub const BUILD_DIR: &'static str = "build";
    pub const CONFIG_FILE: &'static str = "xargo.toml";
    pub const LOCK_FILE: &'static str = "Xargo.lock";
    pub const GITIGNORE: &'static str = ".gitignore";

    typedir::typedir! {
        node RootDir {
            CONFIG_FILE => node ConfigFile;
            LOCK_FILE => node LockFile;
            SRC_DIR => node SrcDir;
            MAIN_FILE => node MainFile;
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

pub mod conf {
    use anyhow::{anyhow, Error, Result};
    use std::path::PathBuf;

    pub const CONFIG_DIR: &'static str = ".xargo";
    pub const CONFIG_FILE: &'static str = "config.toml";

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
