use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{dirs, options};

#[derive(Debug)]
pub struct Project {
    pub root: dirs::proj::RootDir,
    pub config: ProjectConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectConfigHead,
    #[serde(rename = "profile", flatten)]
    pub profiles: Profiles,
    #[serde(flatten)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Profiles(BTreeMap<String, Profile>);

impl Profiles {
    pub fn new() -> Profiles {
        Self(BTreeMap::new())
    }

    pub fn select_profile(mut self, name: &str) -> Option<Profile> {
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
            tex_format: self.tex_format.and(other.tex_format),
            tex_engine: self.tex_engine.and(other.tex_engine),
            bib_engine: self.bib_engine.and(other.bib_engine),
        }
    }
}

impl ProjectSettings {
    pub fn merge(self, other: Self) -> Self {
        Self {
            output_format: self.output_format.and(other.output_format),
            shell_escape: self.shell_escape.and(other.shell_escape),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dependencies(BTreeMap<String, Dependency>);

impl Dependencies {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl<'a> IntoIterator for &'a Dependencies {
    type Item = <&'a BTreeMap<String, Dependency> as IntoIterator>::Item;

    type IntoIter = <&'a BTreeMap<String, Dependency> as IntoIterator>::IntoIter;

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
        use typedir::SubDir;
        let root = dirs::proj::RootDir::find()?;
        let path = dirs::proj::ConfigFile::from(root);
        let conf: ProjectConfig = config::Config::builder()
            .add_source(config::File::new(
                path.as_ref()
                    .as_os_str()
                    .to_str()
                    .expect("non-UTF-8 path or something"),
                config::FileFormat::Toml,
            ))
            .build()?
            .try_deserialize()?;
        Ok(Self {
            root: path.parent(),
            config: conf,
        })
    }
}
