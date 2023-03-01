//! Types for LaTeX package file templates
#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone)]
pub enum PackageTexFormat {
    Latex2e,
}

impl fmt::Display for PackageTexFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PackageTexFormat::*;
        match self {
            Latex2e => write!(f, "LaTeX2e"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageName(String);

impl AsRef<str> for PackageName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct PackageDate {
    /// Perhaps LaTeX packages were written in antiquity
    year: i32,
    month: u32,
    day: u32,
}

impl fmt::Display for PackageDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.year, self.month, self.day)
    }
}

impl PackageDate {
    fn current() -> Self {
        use chrono::Datelike;
        let local_time: chrono::DateTime<_> = chrono::Local::now();
        PackageDate {
            year: local_time.year(),
            month: local_time.month(),
            day: local_time.day(),
        }
    }
}

/// The package identification banner that may appear in the optional argument to the `\Provides...` macro
#[derive(Debug, Clone)]
pub struct IdentBanner(String);

impl TryFrom<String> for IdentBanner {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.contains("Standard LaTeX") {
            Err(anyhow::anyhow!(
                "The phrase \"Standard LaTeX\" must not be used in the identification banner."
            ))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for IdentBanner {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ProvidesOptionalArg {
    date: PackageDate,
    banner: Option<IdentBanner>,
}

impl fmt::Display for ProvidesOptionalArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.date)?;
        if let Some(banner) = &self.banner {
            write!(f, " {}", banner.as_ref())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum PackageKind {
    Package,
    Class,
}

impl PackageKind {
    fn provides_macro(&self) -> &'static str {
        match self {
            PackageKind::Package => "ProvidesPackage",
            PackageKind::Class => "ProvidesClass",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LatexPackageTemplate {
    kind: PackageKind,
    name: PackageName,
    needs_format: PackageTexFormat,
    provides_options: Option<ProvidesOptionalArg>,
}

impl fmt::Display for LatexPackageTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Declare required TeX format
        writeln!(f, r#"\NeedsTexFormat{{{}}}"#, self.needs_format)?;
        // Declare provided package, with options
        write!(
            f,
            r#"\{}{{{}}}"#,
            self.kind.provides_macro(),
            self.name.as_ref()
        )?;
        if let Some(opts) = &self.provides_options {
            write!(f, "[{}]", opts)?;
        }
        writeln!(f)?;
        Ok(())
    }
}
