//! Tool configuration

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::dirs::{self, ContentString as S};

pub const DEBUG_PROFILE: &'static str = "debug";
pub const RELEASE_PROFILE: &'static str = "release";

// FIXME: these shouldn't know about `clap`.
/// The document preparation systems that can be used by a package.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum TexFormat {
    Tex,
    #[default]
    Latex,
}

/// The document preparation systems that can be used by a package.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum TexEngine {
    Tex,
    #[default]
    Pdftex,
    Xetex,
    Luatex,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Dvi,
    Ps,
    Pdf,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum BibEngine {
    Biber,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Executable<'c>(&'c str);

impl<'c> AsRef<str> for Executable<'c> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'c> AsRef<std::ffi::OsStr> for Executable<'c> {
    fn as_ref(&self) -> &std::ffi::OsStr {
        &self.0.as_ref()
    }
}

macro_rules! executable_config {
    ($($exec:ident),*) => {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(default)]
        pub struct ExecutableConfig<'c> {
            $(
                #[serde(borrow)]
                pub $exec: Executable<'c>,
            )*
        }

        impl<'c> Default for ExecutableConfig<'c> {
            fn default() -> Self {
                Self {
                    $(
                        $exec: Executable(stringify!($exec)),
                    )*
                }
            }
        }
    };
}

executable_config![tex, latex, pdftex, pdflatex, xetex, xelatex, luatex, lualatex, biber];

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LargoConfig<'c> {
    #[serde(flatten, borrow)]
    pub executables: ExecutableConfig<'c>,
    /// The default profile selected if no other profile is chosen.
    pub default_profile: crate::project::ProfileName,
    /// The default TeX format
    pub default_tex_format: TexFormat,
    /// The default TeX engine
    pub default_tex_engine: TexEngine,
    /// Global bibliography file
    pub default_bibliography: Option<&'c str>,
}

impl<'c> LargoConfig<'c> {
    fn new(content: &'c S<dirs::LargoConfigFile>) -> Result<Self> {
        let config = toml::from_str(content)?;
        Ok(config)
    }

    pub fn choose_program(&self, engine: TexEngine, format: TexFormat) -> &Executable<'c> {
        match (engine, format) {
            (TexEngine::Tex, TexFormat::Tex) => &self.executables.tex,
            (TexEngine::Tex, TexFormat::Latex) => &self.executables.latex,
            (TexEngine::Pdftex, TexFormat::Tex) => &self.executables.pdftex,
            (TexEngine::Pdftex, TexFormat::Latex) => &self.executables.pdflatex,
            (TexEngine::Xetex, TexFormat::Tex) => &self.executables.xetex,
            (TexEngine::Xetex, TexFormat::Latex) => &self.executables.xelatex,
            (TexEngine::Luatex, TexFormat::Tex) => &self.executables.luatex,
            (TexEngine::Luatex, TexFormat::Latex) => &self.executables.lualatex,
        }
    }
}

/// Get configuration in the current working directory
pub fn with_config<T, F: FnOnce(&LargoConfig, Option<crate::project::Project>) -> T>(
    f: F,
) -> Result<T> {
    // Global config
    let global_config_dir = dirs::LargoConfigDir::global_config()?;
    let global_config_file = typedir::path!(global_config_dir => dirs::LargoConfigFile);
    // TODO: shouldn't crash if you have no config file; instead, just give you
    // the default config.
    let global_config_contents =
        dirs::LargoConfigFile::try_read(&global_config_file).expect("here's the bug boss");
    let global_config = LargoConfig::new(&global_config_contents)?;

    // Project configuration
    let root = dirs::RootDir::find().ok();
    let project = if let Some(mut root) = root {
        let project_config_file = typedir::pathref!(root => dirs::ProjectConfigFile);
        let project_config_contents = dirs::ProjectConfigFile::try_read(&project_config_file)?;
        let project_config = toml::from_str(&project_config_contents)?;
        drop(project_config_file);
        Some(crate::project::Project {
            root,
            config: project_config,
        })
    } else {
        None
    };

    Ok(f(&global_config, project))
}
