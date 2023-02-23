use serde::{Deserialize, Serialize};

/// The document preparation systems that can be used by a package.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum TexFormat {
    Tex,
    Latex,
}

/// The document preparation systems that can be used by a package.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum TexEngine {
    Tex,
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
