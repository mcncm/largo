pub mod proj {
    use crate::project;
    use anyhow::{anyhow, Result};
    use typedir::{path, pathref, PathBuf as P, PathRef as R};

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
                forall s: &str, s => node SrcFile;
            };
            BUILD_DIR => node BuildDir {
                forall s: &crate::project::ProfileName, s.as_ref() => node ProfileBuildDir;
            };
            GIT_DIR => node GitDir;
            GITIGNORE => node Gitignore;
        };
    }

    pub enum ProjectKind {
        Package,
        Class,
        Document,
    }

    pub struct NewProject<'a> {
        /// Project name
        pub name: &'a str,
        /// What kind of project is this?
        pub kind: ProjectKind,
    }

    impl<'a> NewProject<'a> {
        fn project_toml(&self) -> project::ProjectConfig {
            let package = match self.kind {
                ProjectKind::Package => Some(project::PackageConfig::default()),
                _ => None,
            };
            let class = match self.kind {
                ProjectKind::Class => Some(project::ClassConfig::default()),
                _ => None,
            };
            project::ProjectConfig {
                project: project::ProjectConfigHead {
                    name: self.name.to_string(),
                    system_settings: project::SystemSettings::default(),
                    project_settings: project::ProjectSettings::default(),
                },
                package,
                class,
                profiles: project::Profiles::new(),
                dependencies: project::Dependencies::new(),
            }
        }

        /// Create a `main.tex`, `abc.sty`, or `xyz.cls`
        fn try_create_src_file(&self, src_dir: &mut R<SrcDir>) -> Result<()> {
            use typedir::Extend;
            match self.kind {
                ProjectKind::Package => {
                    let src_file: R<SrcFile> = src_dir.extend("main.sty");
                    let template = crate::files::packages::PackageTemplate::new(&self.name.into());
                    try_create(
                        &src_file,
                        ToCreate::File(format!("{}", template).as_bytes()),
                    )
                }
                ProjectKind::Class => {
                    let src_file: R<SrcFile> = src_dir.extend("main.cls");
                    let template = crate::files::packages::ClassTemplate::new(&self.name.into());
                    try_create(
                        &src_file,
                        ToCreate::File(format!("{}", template).as_bytes()),
                    )
                }
                ProjectKind::Document => {
                    let src_file: R<SrcFile> = src_dir.extend("main.tex");
                    try_create(&src_file, ToCreate::File(crate::files::MAIN_LATEX))
                }
            }
        }

        /// Initialize a largo project directory at the passed root
        pub fn init(self, root: std::path::PathBuf) -> Result<()> {
            // NOTE: This is *extremely* verbose without some kind of "pop-on-drop"
            // list structure. Unfortunately, that seems to be tricky to mix with
            // lots of newtypes and generics and macros.
            let mut root = P::new(RootDir(()), root);
            // Init git
            std::process::Command::new("git")
                .arg("init")
                .arg(root.as_os_str())
                .output()?;
            // Project config file
            {
                let proj_conf = pathref!(root => ConfigFile);
                ConfigFile::try_create(&proj_conf, &self.project_toml())?;
            }
            // Gitignore
            {
                let gitignore = pathref!(root => Gitignore);
                try_create(&gitignore, ToCreate::File(crate::files::GITIGNORE))?;
            }
            // Source
            {
                let mut src_dir = pathref!(root => SrcDir);
                try_create(&src_dir, ToCreate::Dir)?;
                self.try_create_src_file(&mut src_dir)?;
            }
            // Build directory
            let build_dir = path!(root => BuildDir);
            try_create(&build_dir, ToCreate::Dir)?;
            Ok(())
        }
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
