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
            SRC_DIR => node SrcDir {
                MAIN_FILE => node MainFile;
            };
            BUILD_DIR => node BuildDir;
            GITIGNORE => node Gitignore;
        };
    }

    /// Initialize a xargo project directory at the passed root
    pub fn init(root: std::path::PathBuf, new_proj: NewProject) -> Result<()> {
        // NOTE: This is *extremely* verbose without some kind of "pop-on-drop"
        // list structure. Unfortunately, that seems to be tricky to mix with
        // lots of newtypes and generics and macros.
        use typedir::SubDir;
        let root = RootDir(root);
        // Project config file

        let proj_conf = ConfigFile::from(root);
        proj_conf.try_create(&new_proj.project_config)?;
        let root = proj_conf.parent();
        // Gitignore
        let gitignore = Gitignore::from(root);
        try_create(
            &gitignore,
            ToCreate::File(include_bytes!("files/gitignore.txt")),
        )?;
        let root = gitignore.parent();
        // Source
        let src_dir = SrcDir::from(root);
        try_create(&src_dir, ToCreate::Dir)?;
        let main_file = MainFile::from(src_dir);
        try_create(
            &main_file,
            ToCreate::File(include_bytes!("files/main_latex.tex")),
        )?;
        let root = main_file.parent().parent();
        // Build directory
        let build_dir = BuildDir::from(root);
        try_create(&build_dir, ToCreate::Dir)?;
        Ok(())
    }

    pub struct NewProject<'a> {
        pub project_config: &'a crate::project::ProjectConfig,
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

    impl ConfigFile {
        fn try_create(&self, project_config: &crate::project::ProjectConfig) -> Result<()> {
            try_create(self, ToCreate::File(&toml::ser::to_vec(&project_config)?))
        }
    }

    // What to create
    enum ToCreate<'a> {
        Dir,
        File(&'a [u8]),
    }

    fn try_create<N: typedir::Node>(node: &N, to_create: ToCreate) -> Result<()> {
        use std::io::Write;
        match to_create {
            ToCreate::Dir => std::fs::create_dir(node.as_ref())?,
            ToCreate::File(contents) => {
                // FIXME race condition! TOC/TOU! Not good!
                if node.as_ref().exists() {
                    return Err(anyhow!(
                        "file already exists: `{}`",
                        node.as_ref().display()
                    ));
                }
                let mut f = std::fs::File::create(node.as_ref())?;
                f.write_all(contents)?;
            }
        }
        Ok(())
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
