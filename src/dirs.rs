pub mod proj {
    use anyhow::{anyhow, Result};
    use typedir::{path, pathref, PathBuf as P};

    pub const SRC_DIR: &'static str = "src";
    pub const MAIN_FILE: &'static str = "main.tex";
    pub const BUILD_DIR: &'static str = "build";
    pub const CONFIG_FILE: &'static str = "largo.toml";
    pub const LOCK_FILE: &'static str = "largo.lock";
    pub const GITIGNORE: &'static str = ".gitignore";
    pub const GIT_DIR: &'static str = ".git";

    typedir::typedir! {
        node RootDir {
            CONFIG_FILE => node ConfigFile;
            LOCK_FILE => node LockFile;
            SRC_DIR => node SrcDir {
                MAIN_FILE => node MainFile;
            };
            BUILD_DIR => node BuildDir {
                forall s: &crate::project::ProfileName, s.as_ref() => node ProfileBuildDir;
            };
            GIT_DIR => node GitDir;
            GITIGNORE => node Gitignore;
        };
    }

    /// Initialize a largo project directory at the passed root
    pub fn init(root: std::path::PathBuf, new_proj: NewProject) -> Result<()> {
        // NOTE: This is *extremely* verbose without some kind of "pop-on-drop"
        // list structure. Unfortunately, that seems to be tricky to mix with
        // lots of newtypes and generics and macros.
        let mut root = P::new(RootDir(()), root);
        // Project config file
        {
            let proj_conf = pathref!(root => ConfigFile);
            ConfigFile::try_create(&proj_conf, &new_proj.project_config)?;
        }
        // Gitignore
        {
            let gitignore = pathref!(root => Gitignore);
            try_create(
                &gitignore,
                ToCreate::File(include_bytes!("files/gitignore.txt")),
            )?;
        }
        // Git directory
        {
            let git_dir = pathref!(root => GitDir);
            std::process::Command::new("git")
                .arg("init")
                .arg(git_dir.as_os_str())
                .output()?;
        }
        // Source
        {
            let mut src_dir = pathref!(root => SrcDir);
            try_create(&src_dir, ToCreate::Dir)?;
            {
                let main_file = pathref!(src_dir => MainFile);
                try_create(
                    &main_file,
                    ToCreate::File(include_bytes!("files/main_latex.tex")),
                )?;
            }
        }
        // Build directory
        let build_dir = path!(root => BuildDir);
        try_create(&build_dir, ToCreate::Dir)?;
        Ok(())
    }

    pub struct NewProject<'a> {
        pub project_config: &'a crate::project::ProjectConfig,
    }

    impl RootDir {
        pub fn find() -> Result<P<Self>> {
            let mut path = std::env::current_dir().unwrap();
            let path_cpy = path.clone();
            loop {
                path.push(CONFIG_FILE);
                if path.exists() {
                    path.pop();
                    return Ok(P::new(Self(()), path));
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
        fn try_create<P: typedir::AsPath<Self>>(
            path: &P,
            project_config: &crate::project::ProjectConfig,
        ) -> Result<()> {
            try_create(path, ToCreate::File(&toml::ser::to_vec(&project_config)?))
        }
    }

    // What to create
    enum ToCreate<'a> {
        Dir,
        File(&'a [u8]),
    }

    fn try_create<N: typedir::Node, P: typedir::AsPath<N>>(
        path: &P,
        to_create: ToCreate,
    ) -> Result<()> {
        use std::io::Write;
        match to_create {
            ToCreate::Dir => std::fs::create_dir(&path)?,
            ToCreate::File(contents) => {
                // FIXME race condition! TOC/TOU! Not good!
                if path.exists() {
                    return Err(anyhow!("file already exists: `{}`", path.display()));
                }
                let mut f = std::fs::File::create(&path)?;
                f.write_all(contents)?;
            }
        }
        Ok(())
    }
}

pub mod conf {
    use anyhow::{anyhow, Result};
    use std::path::PathBuf;
    use typedir::{AsPath, Extend, PathBuf as P};

    pub const CONFIG_DIR: &'static str = ".largo";
    pub const CONFIG_FILE: &'static str = "config.toml";

    typedir::typedir! {
        node HomeDir {
            CONFIG_DIR => node ConfigDir {
                CONFIG_FILE => node ConfigFile;
            };
        };
    }

    impl HomeDir {
        /// NOTE: Intentionally not globally visible!
        fn try_get() -> Result<P<Self>> {
            // This if/else chain should optimize away, it's guaranteed to be exhaustive
            // (unlike `#[cfg(...)]`), and rust-analyzer won't ignore the dead cases.
            let path = if cfg!(target_family = "unix") {
                PathBuf::from(std::env::var("HOME")?)
            } else if cfg!(target_family = "windows") {
                PathBuf::from(std::env::var("USERPROFILE")?)
            } else {
                // The only other `target_family` at this time is `wasm`.
                unreachable!("target unsupported");
            };
            Ok(P::new(HomeDir(()), path))
        }
    }

    impl ConfigDir {
        #[allow(dead_code)]
        pub fn global_config() -> Result<P<Self>> {
            let home = HomeDir::try_get()?;
            Ok(home.extend(()))
        }
    }

    #[derive(Debug)]
    pub struct ConfigFileSource(config::File<config::FileSourceFile, config::FileFormat>);

    impl ConfigFileSource {
        pub fn try_from_path<P: AsPath<ConfigFile>>(path: &P) -> Result<Self> {
            let source = config::File::new(
                path.to_str()
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
