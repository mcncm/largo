use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{conf, dirs};

#[derive(Debug)]
pub struct Project<'c> {
    pub root: typedir::PathBuf<dirs::RootDir>,
    pub config: ProjectConfig<'c>,
}

#[derive(Debug, Deserialize, Serialize)]
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
    type Error = anyhow::Error;

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
    #[serde(flatten)]
    pub system_settings: SystemSettings,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SystemSettings {
    pub tex_format: Option<conf::TexFormat>,
    pub tex_engine: Option<conf::TexEngine>,
    pub bib_engine: Option<conf::BibEngine>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProjectSettings {
    pub output_format: Option<conf::OutputFormat>,
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
    type Error = anyhow::Error;

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
pub enum Dependency<'c> {
    Path { path: &'c str },
}
