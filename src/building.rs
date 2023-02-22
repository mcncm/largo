use std::collections::HashMap;
use std::ffi::OsStr;

use anyhow::{anyhow, Result};

use crate::conf::{Executable, XargoConfig};
use crate::dirs;
use crate::project::{self, Project};

struct TexInput(String);

impl AsRef<OsStr> for TexInput {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

fn tex_input(profile_name: &str) -> TexInput {
    TexInput(format!(
        concat!(r#"\def\XPROFILE{{{}}}"#, r#"\input{{{}}}"#),
        profile_name, "src/main.tex"
    ))
}

/// Environment variables for the build command
#[derive(Debug, Default)]
struct BuildVars(HashMap<&'static str, String>);

impl BuildVars {
    fn with_dependencies(mut self, deps: &HashMap<String, project::Dependency>) -> Self {
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

impl From<&project::ProjectConfig> for BuildVars {
    fn from(project_config: &project::ProjectConfig) -> Self {
        BuildVars(HashMap::new()).with_dependencies(&project_config.dependencies)
    }
}

pub struct BuildCmd<'a> {
    build_root: &'a dirs::proj::RootDir,
    build_vars: BuildVars,
    tex_input: TexInput,
    executable: &'a Executable,
}

impl<'a> BuildCmd<'a> {
    pub fn new(
        profile: &'a Option<String>,
        proj: &'a Project,
        conf: &'a XargoConfig,
    ) -> Result<Self> {
        let prof_name = profile.as_deref().unwrap_or(conf.default_profile());
        let _profile = proj
            .config
            .profile
            .get(prof_name)
            .ok_or_else(|| anyhow!("profile `{}` found", prof_name))?;

        let (engine, format) = (proj.config.project.system, proj.config.project.engine);
        Ok(Self {
            build_root: &proj.root,
            build_vars: BuildVars::from(&proj.config),
            tex_input: tex_input(&prof_name),
            executable: conf.choose_program(format, engine),
        })
    }
}

impl Into<std::process::Command> for BuildCmd<'_> {
    fn into(self) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.executable);
        for (var, val) in &self.build_vars.0 {
            cmd.env(var, val);
        }
        cmd.current_dir(self.build_root);
        cmd.args(["-output-directory", dirs::proj::BUILD_DIR]);
        cmd.arg(&self.tex_input);
        cmd
    }
}
