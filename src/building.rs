use std::collections::BTreeMap;
use std::ffi::OsStr;

use anyhow::{anyhow, Result};

use crate::conf::{Executable, LargoConfig};
use crate::dirs;
use crate::project::{self, Project, ProjectSettings, SystemSettings};

struct TexInput(String);

impl AsRef<OsStr> for TexInput {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

/// Variables available at TeX run time
// FIXME: this implementation is very, very suboptimal. It's particularly bad
// for documentation for the keys to be dynamic.
struct LargoVars<'a>(std::collections::BTreeMap<&'static str, &'a str>);

impl<'a> LargoVars<'a> {
    fn new(profile_name: &'a str) -> Self {
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("Profile", profile_name.clone());
        Self(vars)
    }

    fn to_defs(self) -> String {
        use std::fmt::Write;
        let mut defs = String::new();
        for (k, v) in self.0.into_iter() {
            write!(&mut defs, r#"\def\L:{k}{{{v}}}"#).unwrap();
        }
        defs
    }
}

// TODO Other TeX vars: `\X:OUTPUTDIR`
fn tex_input(profile_name: &str) -> TexInput {
    let vars = LargoVars::new(profile_name);
    let vars = vars.to_defs();
    let main_file = "src/main.tex";
    TexInput(format!(r#"{vars}\input{{{main_file}}}"#))
}

/// Environment variables for the build command
#[derive(Debug, Default)]
struct BuildVars(BTreeMap<&'static str, String>);

impl BuildVars {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl BuildVars {
    fn with_dependencies(mut self, deps: &BTreeMap<String, project::Dependency>) -> Self {
        let mut tex_inputs = String::new();
        for (_dep_name, dep_body) in deps {
            match &dep_body {
                project::Dependency::Path { path } => {
                    tex_inputs += &path;
                    tex_inputs.push(':');
                }
            }
        }
        if !tex_inputs.is_empty() {
            self.0.insert("TEXINPUTS", tex_inputs);
        }
        self
    }
}

pub struct BuildCmd<'a> {
    build_root: dirs::proj::RootDir,
    build_vars: BuildVars,
    tex_input: TexInput,
    executable: &'a Executable,
    project_settings: ProjectSettings,
}

struct _BuildSettings {
    system_settings: SystemSettings,
    project_settings: ProjectSettings,
}

impl<'a> BuildCmd<'a> {
    pub fn new(profile: &'a Option<String>, proj: Project, conf: &'a LargoConfig) -> Result<Self> {
        let prof_name = profile.as_deref().unwrap_or(conf.default_profile());
        let mut profiles = proj.config.profiles;
        let profile = profiles
            .remove(prof_name)
            .ok_or_else(|| anyhow!("profile `{}` found", prof_name))?;
        let proj_config = proj.config.project;
        let project_settings = proj_config.project_settings.merge(profile.project_settings);
        let system_settings = proj_config.system_settings.merge(profile.system_settings);
        let engine = system_settings
            .tex_engine
            .unwrap_or(conf.default_tex_engine);
        let system = system_settings
            .tex_format
            .unwrap_or(conf.default_tex_format);
        let build_vars = BuildVars::new().with_dependencies(&proj.config.dependencies);

        Ok(Self {
            build_root: proj.root,
            build_vars,
            tex_input: tex_input(&prof_name),
            executable: conf.choose_program(engine, system),
            project_settings,
        })
    }
}

impl Into<std::process::Command> for BuildCmd<'_> {
    fn into(self) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.executable);
        cmd.current_dir(self.build_root);
        for (var, val) in &self.build_vars.0 {
            cmd.env(var, val);
        }
        let mut pdflatex_options = crate::engines::pdflatex::CommandLineOptions::default();
        match self.project_settings.shell_escape {
            Some(true) => {
                pdflatex_options.shell_escape = true;
            }
            Some(false) => {
                pdflatex_options.no_shell_escape = true;
            }
            None => (),
        };
        use clam::Options;
        pdflatex_options.apply(&mut cmd);
        match &self.project_settings.shell_escape {
            Some(true) => cmd.arg("-shell-escape"),
            Some(false) => cmd.arg("-no-shell-escape"),
            // Needed to make types match
            None => &mut cmd,
        }
        .args(["-output-directory", dirs::proj::BUILD_DIR])
        .arg(&self.tex_input);
        cmd
    }
}
