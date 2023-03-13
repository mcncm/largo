use crate::conf;
use anyhow::{anyhow, Result};
use typedir::{path, pathref, AsPath, Extend, PathBuf as P, PathRef as R};

// Project
pub const SRC_DIR: &str = "src";
pub const MAIN_FILE: &str = "main.tex";
pub const TARGET_DIR: &str = "target";
pub const BUILD_DIR: &str = "build";
pub const DEPS_DIR: &str = "deps";
pub const PROJECT_CONFIG_FILE: &str = "largo.toml";
pub const LOCK_FILE: &str = "largo.lock";
pub const GITIGNORE: &str = ".gitignore";
pub const GIT_DIR: &str = ".git";

// Largo
pub const CONFIG_DIR: &str = ".largo";
pub const LARGO_CONFIG_FILE: &str = "config.toml";

/// Strongly-typed file contents
pub struct ContentString<N: typedir::Node>(String, std::marker::PhantomData<N>);

impl<N: typedir::Node> std::ops::Deref for ContentString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N: typedir::Node> AsRef<str> for ContentString<N> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

typedir::typedir! {
    node RootDir {
        PROJECT_CONFIG_FILE => node ProjectConfigFile;
        LOCK_FILE => node LockFile;
        SRC_DIR => node SrcDir {
            forall s: &str, s => node SrcFile;
        };
        TARGET_DIR => node TargetDir {
            forall s: &crate::conf::ProfileName<'_>, s.as_ref() => node ProfileTargetDir {
                DEPS_DIR => node DepsDir;
                BUILD_DIR => node BuildDir;
            };
        };
        GIT_DIR => node GitDir;
        GITIGNORE => node Gitignore;
    };

    node HomeDir {
        CONFIG_DIR => node LargoConfigDir {
            LARGO_CONFIG_FILE => node LargoConfigFile;
        };
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
    fn project_toml(&self) -> conf::ProjectConfig {
        let package = match self.kind {
            ProjectKind::Package => Some(conf::PackageConfig::default()),
            _ => None,
        };
        let class = match self.kind {
            ProjectKind::Class => Some(conf::ClassConfig::default()),
            _ => None,
        };
        conf::ProjectConfig {
            project: conf::ProjectConfigHead {
                name: self.name.to_string(),
                system_settings: conf::SystemSettings::default(),
                project_settings: conf::ProjectSettings::default(),
            },
            package,
            class,
            profiles: None,
            dependencies: conf::Dependencies::new(),
        }
    }

    /// Create a `main.tex`, `abc.sty`, or `xyz.cls`
    fn try_create_src_file(&self, src_dir: &mut R<SrcDir>) -> Result<()> {
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
            let proj_conf = pathref!(root => ProjectConfigFile);
            ProjectConfigFile::try_create(&proj_conf, &self.project_toml())?;
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
        let build_dir = path!(root => TargetDir);
        try_create(&build_dir, ToCreate::Dir)?;
        Ok(())
    }
}

impl RootDir {
    pub fn find() -> Result<P<Self>> {
        let mut path = std::env::current_dir().unwrap();
        let path_cpy = path.clone();
        loop {
            path.push(PROJECT_CONFIG_FILE);
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

impl ProjectConfigFile {
    fn try_create<P: typedir::AsPath<Self>>(
        path: &P,
        project_config: &crate::conf::ProjectConfig,
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
        ToCreate::Dir => std::fs::create_dir(path)?,
        ToCreate::File(contents) => {
            // FIXME race condition! TOC/TOU! Not good!
            if path.exists() {
                return Err(anyhow!("file already exists: `{}`", path.display()));
            }
            let mut f = std::fs::File::create(path)?;
            f.write_all(contents)?;
        }
    }
    Ok(())
}

/// A thin wrapper around `std::fs::remove_dir_all` that ignores `NotFound` errors.
pub fn remove_dir_all<N: typedir::Node, P: typedir::AsPath<N>>(dir: &P) -> crate::Result<()> {
    let res = std::fs::remove_dir_all(dir);
    if let Err(err) = res {
        match err.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(err.into()),
        }
    } else {
        Ok(())
    }
}

impl HomeDir {
    /// NOTE: Intentionally not globally visible!
    fn try_get() -> Result<P<Self>> {
        // This if/else chain should optimize away, it's guaranteed to be exhaustive
        // (unlike `#[cfg(...)]`), and rust-analyzer won't ignore the dead cases.
        let path = if cfg!(target_family = "unix") {
            std::path::PathBuf::from(std::env::var("HOME")?)
        } else if cfg!(target_family = "windows") {
            std::path::PathBuf::from(std::env::var("USERPROFILE")?)
        } else {
            // The only other `target_family` at this time is `wasm`.
            unreachable!("target unsupported");
        };
        Ok(P::new(HomeDir(()), path))
    }
}

impl LargoConfigDir {
    #[allow(dead_code)]
    pub fn global_config() -> Result<P<Self>> {
        let home = HomeDir::try_get()?;
        Ok(home.extend(()))
    }
}

impl LargoConfigFile {
    pub fn try_read<P: AsPath<Self>>(path: &P) -> Result<ContentString<Self>> {
        let content = std::fs::read_to_string(path)?;
        Ok(ContentString(content, std::marker::PhantomData))
    }
}

impl ProjectConfigFile {
    pub fn try_read<P: AsPath<Self>>(path: &P) -> Result<ContentString<Self>> {
        let content = std::fs::read_to_string(path)?;
        Ok(ContentString(content, std::marker::PhantomData))
    }
}
