//! Tool configuration

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::dirs;
use crate::tex::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Executable(String);

impl AsRef<str> for Executable {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for Executable {
    fn as_ref(&self) -> &std::ffi::OsStr {
        &self.0.as_ref()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct XargoConfig {
    tex_executable: Executable,
    latex_executable: Executable,
    pdftex_executable: Executable,
    pdflatex_executable: Executable,
    xetex_executable: Executable,
    xelatex_executable: Executable,
    luatex_executable: Executable,
    lualatex_executable: Executable,

    /// The default profile selected if no other profile is chosen.
    default_profile: String,
}

impl XargoConfig {
    pub fn new() -> Result<Self> {
        let mut builder = config::Config::builder()
            .set_default("tex-executable", "tex")?
            .set_default("latex-executable", "latex")?
            .set_default("pdftex-executable", "pdftex")?
            .set_default("pdflatex-executable", "pdflatex")?
            .set_default("xetex-executable", "xetex")?
            .set_default("xelatex-executable", "xelatex")?
            .set_default("luatex-executable", "luatex")?
            .set_default("lualatex-executable", "lualatex")?
            .set_default("default-profile", "debug")?;

        // TODO: project-local config override
        // // FIXME: race condition!
        // if dirs::conf.as_ref().exists() {
        //     // Use a *local* config as the primary source.
        //     builder = builder.add_source(dirs::conf::ConfigFileSource::try_from(&config_file)?);
        // }

        // Fall back on a *global* config
        let config_dir = dirs::conf::ConfigDir::global_config()?;
        let config_file = dirs::conf::ConfigFile::from(config_dir);
        // FIXME: race condition!
        if config_file.as_ref().exists() {
            builder = builder.add_source(dirs::conf::ConfigFileSource::try_from(&config_file)?);
        }
        Ok(builder.build()?.try_deserialize()?)
    }

    pub fn choose_program(&self, engine: TexEngine, format: TexFormat) -> &Executable {
        match (engine, format) {
            (TexEngine::Tex, TexFormat::Tex) => &self.tex_executable,
            (TexEngine::Tex, TexFormat::Latex) => &self.latex_executable,
            (TexEngine::Pdftex, TexFormat::Tex) => &self.pdftex_executable,
            (TexEngine::Pdftex, TexFormat::Latex) => &self.pdflatex_executable,
            (TexEngine::Xetex, TexFormat::Tex) => &self.xetex_executable,
            (TexEngine::Xetex, TexFormat::Latex) => &self.xelatex_executable,
            (TexEngine::Luatex, TexFormat::Tex) => &self.luatex_executable,
            (TexEngine::Luatex, TexFormat::Latex) => &self.lualatex_executable,
        }
    }

    pub fn default_profile(&self) -> &str {
        &self.default_profile
    }
}
