//! Tool configuration

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::dirs;
use crate::tex::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct XargoConfig {
    tex_executable: Option<String>,
    latex_executable: Option<String>,
    pdftex_executable: Option<String>,
    pdflatex_executable: Option<String>,
    xetex_executable: Option<String>,
    xelatex_executable: Option<String>,
    luatex_executable: Option<String>,
    lualatex_executable: Option<String>,

    /// The default profile selected if no other profile is chosen.
    default_profile: String,
}

impl XargoConfig {
    pub fn new() -> Result<Self> {
        let mut builder = config::Config::builder()
            .set_default("default-profile", "debug")
            .unwrap();

        // TODO: project-local config override
        // // FIXME: race condition!
        // if dirs::conf.as_ref().exists() {
        //     // Use a *local* config as the primary source.
        //     builder = builder.add_source(dirs::conf::ConfigFileSource::try_from(&config_file)?);
        // }

        let config_dir = dirs::conf::ConfigDir::global_config()?;
        let config_file = dirs::conf::ConfigFile::from(config_dir);
        // Fall back on a *global* config
        builder = builder.add_source(dirs::conf::ConfigFileSource::try_from(&config_file)?);
        Ok(builder.build()?.try_deserialize()?)
    }

    pub fn choose_program(&self, engine: TexEngine, format: TexFormat) -> &str {
        match (engine, format) {
            (TexEngine::Tex, TexFormat::Tex) => self.tex_executable.as_deref().unwrap_or("tex"),
            (TexEngine::Tex, TexFormat::Latex) => {
                self.latex_executable.as_deref().unwrap_or("latex")
            }
            (TexEngine::Pdftex, TexFormat::Tex) => {
                self.pdftex_executable.as_deref().unwrap_or("pdftex")
            }
            (TexEngine::Pdftex, TexFormat::Latex) => {
                self.pdflatex_executable.as_deref().unwrap_or("pdflatex")
            }
            (TexEngine::Xetex, TexFormat::Tex) => {
                self.xetex_executable.as_deref().unwrap_or("xetex")
            }
            (TexEngine::Xetex, TexFormat::Latex) => {
                self.xelatex_executable.as_deref().unwrap_or("xelatex")
            }
            (TexEngine::Luatex, TexFormat::Tex) => {
                self.luatex_executable.as_deref().unwrap_or("luatex")
            }
            (TexEngine::Luatex, TexFormat::Latex) => {
                self.lualatex_executable.as_deref().unwrap_or("lualatex")
            }
        }
    }

    pub fn default_profile(&self) -> &str {
        &self.default_profile
    }
}
