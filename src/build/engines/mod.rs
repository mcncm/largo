use crate::dirs;

use smol::{io::BufReader, process::ChildStdout};

pub mod pdflatex;

/// A TeX engine
#[derive(Debug)]
pub struct Engine {
    /// Internal command
    cmd: super::Command,
}

impl Engine {
    pub fn run(&mut self) -> anyhow::Result<BufReader<ChildStdout>> {
        // `async_process::Child` does not require a manual call to `.wait()`.
        let mut child = self.cmd.spawn()?;
        let stdout = child.stdout.take().expect("failed to take child's stdout");
        Ok(smol::io::BufReader::new(stdout))
    }
}

/// This module is visible to _other_ submodules of `engine`, but not to `super`.
mod private {
    /// A builder that wraps a command.
    pub trait CommandBuilder {
        fn inner_cmd(&self) -> &super::super::Command;

        fn inner_cmd_mut(&mut self) -> &mut super::super::Command;
    }
}

/// An interface for cunstructing TeX engines
pub trait EngineBuilder: private::CommandBuilder + Sized {
    fn with_src_dir<P: typedir::AsPath<dirs::SrcDir>>(mut self, dir: P) -> Self {
        self.inner_cmd_mut().current_dir(dir);
        self
    }

    fn with_output_dir<P: typedir::AsPath<dirs::ProfileBuildDir>>(self, path: P) -> Self;

    fn with_verbosity(self, verbosity: &super::Verbosity) -> Self;

    fn with_synctex(self, use_synctex: bool) -> anyhow::Result<Self>;

    fn with_largo_vars(self, vars: &crate::vars::LargoVars) -> anyhow::Result<Self>;

    /// This function takes an `Option<bool>` because many TeX engines have two
    /// flags, `-shell-escape` and `-no-shell-escape`, and I'm not sure they
    /// aren't simple opposites.
    fn with_shell_escape(self, shell_escape: Option<bool>) -> anyhow::Result<Self>;

    fn finish(self) -> Engine;
}
