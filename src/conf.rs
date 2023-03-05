//! Tool configuration

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::dirs::{self, ContentString as S};
use crate::Result;

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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Dvi,
    Ps,
    #[default]
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
    pub default_profile: ProfileName<'c>,
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
pub fn with_config<T, F: FnOnce(&LargoConfig, Option<crate::conf::Project>) -> T>(
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
    if let Some(mut root) = root {
        let project_config_file = typedir::pathref!(root => dirs::ProjectConfigFile);
        let project_config_contents = dirs::ProjectConfigFile::try_read(&project_config_file)?;
        let project_config = toml::from_str(&project_config_contents)?;
        drop(project_config_file);
        let project = Some(crate::conf::Project {
            root,
            config: project_config,
        });
        Ok(f(&global_config, project))
    } else {
        Ok(f(&global_config, None))
    }
}

#[derive(Debug)]
pub struct Project<'c> {
    pub root: typedir::PathBuf<dirs::RootDir>,
    pub config: ProjectConfig<'c>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectConfig<'c> {
    pub project: ProjectConfigHead,
    pub package: Option<PackageConfig>,
    pub class: Option<ClassConfig>,
    #[serde(rename = "profile", default, borrow)]
    pub profiles: Profiles<'c>,
    #[serde(default)]
    pub dependencies: Dependencies<'c>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectConfigHead {
    pub name: String,
    #[serde(flatten)]
    pub project_settings: ProjectSettings,
    #[serde(flatten)]
    pub system_settings: SystemSettings,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackageConfig {}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClassConfig {}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ProfileName<'c>(&'c str);

impl<'c> Default for ProfileName<'c> {
    fn default() -> Self {
        Self(crate::conf::DEBUG_PROFILE)
    }
}

impl<'c> AsRef<str> for ProfileName<'c> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'c> std::fmt::Display for ProfileName<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'c> TryFrom<&'c str> for ProfileName<'c> {
    type Error = crate::Error;

    fn try_from(s: &'c str) -> std::result::Result<Self, Self::Error> {
        Ok(Self(s))
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Profiles<'c>(#[serde(borrow)] BTreeMap<ProfileName<'c>, Profile>);

impl<'c> Profiles<'c> {
    pub fn new() -> Profiles<'c> {
        Self(BTreeMap::new())
    }

    pub fn select_profile(mut self, name: &ProfileName<'c>) -> Option<Profile> {
        self.0.remove(name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
    #[serde(flatten)]
    pub project_settings: ProjectSettings,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemSettings {
    pub tex_format: TexFormat,
    pub tex_engine: TexEngine,
    pub bib_engine: Option<BibEngine>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectSettings {
    pub output_format: Option<OutputFormat>,
    /// Whether to use shell-escape (if present and `true`), no-shell-escape (if
    /// present and `false`), or neither.
    pub shell_escape: Option<bool>,
    /// whether to use SyncTeX to synchronize between TeX source and the
    /// compiled document
    #[serde(default)]
    pub synctex: bool,
}

impl ProjectSettings {
    pub fn merge(self, other: Self) -> Self {
        Self {
            output_format: self.output_format.or(other.output_format),
            shell_escape: self.shell_escape.or(other.shell_escape),
            // TODO: think: is this really how we want to merge these? Isn't this
            // too infectious?
            synctex: self.synctex || other.synctex,
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct DependencyName<'c>(&'c str);

impl<'c> AsRef<str> for DependencyName<'c> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'c> std::fmt::Display for DependencyName<'c> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'c> TryFrom<&'c str> for DependencyName<'c> {
    type Error = crate::Error;

    fn try_from(s: &'c str) -> std::result::Result<Self, Self::Error> {
        Ok(Self(s))
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dependencies<'c>(#[serde(borrow)] BTreeMap<DependencyName<'c>, Dependency<'c>>);

impl<'c> Dependencies<'c> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl<'a> IntoIterator for &'a Dependencies<'a> {
    type Item = <&'a BTreeMap<DependencyName<'a>, Dependency<'a>> as IntoIterator>::Item;

    type IntoIter = <&'a BTreeMap<DependencyName<'a>, Dependency<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Dependency<'c> {
    #[serde(default)]
    pub largo: bool,
    #[serde(flatten, borrow)]
    pub kind: DependencyKind<'c>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum DependencyKind<'c> {
    Path {
        #[serde(borrow)]
        path: &'c std::path::Path,
    },
}
