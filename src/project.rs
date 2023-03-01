use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{dirs, options};

use typedir::pathref;

#[derive(Debug)]
pub struct Project {
    pub root: typedir::PathBuf<dirs::proj::RootDir>,
    pub config: ProjectConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectConfigHead,
    pub package: Option<PackageConfig>,
    pub class: Option<ClassConfig>,
    #[serde(rename = "profile", default)]
    pub profiles: Profiles,
    #[serde(default)]
    pub dependencies: Dependencies,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfigHead {
    pub name: String,
    #[serde(flatten)]
    pub project_settings: ProjectSettings,
    #[serde(flatten)]
    pub system_settings: SystemSettings,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PackageConfig {}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ClassConfig {}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ProfileName(String);

impl AsRef<str> for ProfileName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<String> for ProfileName {
    type Error = anyhow::Error;

    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self(s))
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Profiles(BTreeMap<ProfileName, Profile>);

impl Profiles {
    pub fn new() -> Profiles {
        Self(BTreeMap::new())
    }

    pub fn select_profile(mut self, name: &ProfileName) -> Option<Profile> {
        self.0.remove(name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
    #[serde(flatten)]
    pub project_settings: ProjectSettings,
    #[serde(flatten)]
    pub system_settings: SystemSettings,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SystemSettings {
    pub tex_format: Option<options::TexFormat>,
    pub tex_engine: Option<options::TexEngine>,
    pub bib_engine: Option<options::BibEngine>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProjectSettings {
    pub output_format: Option<options::OutputFormat>,
    pub shell_escape: Option<bool>,
}

impl SystemSettings {
    pub fn merge(self, other: Self) -> Self {
        Self {
            tex_format: self.tex_format.or(other.tex_format),
            tex_engine: self.tex_engine.or(other.tex_engine),
            bib_engine: self.bib_engine.or(other.bib_engine),
        }
    }
}

impl ProjectSettings {
    pub fn merge(self, other: Self) -> Self {
        Self {
            output_format: self.output_format.or(other.output_format),
            shell_escape: self.shell_escape.or(other.shell_escape),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct DependencyName(String);

impl AsRef<str> for DependencyName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DependencyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<String> for DependencyName {
    type Error = anyhow::Error;

    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self(s))
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dependencies(BTreeMap<DependencyName, Dependency>);

impl Dependencies {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl<'a> IntoIterator for &'a Dependencies {
    type Item = <&'a BTreeMap<DependencyName, Dependency> as IntoIterator>::Item;

    type IntoIter = <&'a BTreeMap<DependencyName, Dependency> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dependency {
    Path { path: String },
}

impl Project {
    pub fn find() -> Result<Self> {
        use dirs::proj::*;
        let mut root = RootDir::find()?;
        let conf: ProjectConfig = {
            let path = pathref!(root => ConfigFile);
            config::Config::builder()
                .add_source(config::File::new(
                    path.to_str().expect("non-UTF-8 path or something"),
                    config::FileFormat::Toml,
                ))
                .build()?
                .try_deserialize()?
        };
        Ok(Self { root, config: conf })
    }
}
