use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{dirs, tex};

#[derive(Debug)]
pub struct Project {
    pub root: dirs::proj::RootDir,
    pub config: ProjectConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectConfigGeneral,
    pub profile: HashMap<String, Profile>,
    pub dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfigGeneral {
    pub name: String,
    pub format: tex::TexFormat,
    pub engine: tex::TexEngine,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
    pub output_format: tex::OutputFormat,
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
