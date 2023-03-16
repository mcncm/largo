use crate::{build, dirs, Result};

use smol::{io::BufReader, process::ChildStdout};

pub mod pdflatex;

pub type DependencyPaths = Vec<std::path::PathBuf>;

/// A TeX engine
#[derive(Debug)]
pub struct Engine {
    /// Internal command
    cmd: crate::Command,
}

#[derive(Debug)]
pub enum EngineInfo {
    Error { line: usize, msg: String },
}

#[derive(Debug)]
pub struct EngineOutput {
    lines: smol::io::Lines<BufReader<ChildStdout>>,
}

impl smol::stream::Stream for EngineOutput {
    type Item = EngineInfo;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use smol::stream::StreamExt;
        use std::task::Poll;
        match self.lines.poll_next(cx) {
            Poll::Ready(Some(Ok(mut line))) => {
                if line.starts_with("! ") {
                    // First two characters are "! "
                    let msg = line.split_off(2);
                    let info = EngineInfo::Error { line: 0, msg };
                    Poll::Ready(Some(info.into()))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(_err))) => panic!("unexpected error"),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

impl Engine {
    pub fn run(&mut self) -> Result<EngineOutput> {
        use smol::prelude::*;
        let stdout = self.run_inner()?;
        let lines = stdout.lines();
        Ok(EngineOutput { lines })
    }

    fn run_inner(&mut self) -> Result<BufReader<ChildStdout>> {
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
    fn with_src_dir<P: typedir::AsPath<dirs::SrcDir>>(self, dir: P) -> Self;

    fn with_build_dir<P: typedir::AsPath<dirs::BuildDir>>(mut self, dir: P) -> Self {
        self.inner_cmd_mut().current_dir(dir);
        self
    }

    fn with_verbosity(self, verbosity: &build::Verbosity) -> Self;

    fn with_synctex(self, use_synctex: bool) -> Result<Self>;

    /// This function takes an `Option<bool>` because many TeX engines have two
    /// flags, `-shell-escape` and `-no-shell-escape`, and I'm not sure they
    /// aren't simple opposites.
    fn with_shell_escape(self, shell_escape: Option<bool>) -> Result<Self>;

    fn with_jobname(self, jobname: String) -> Result<Self>;

    fn with_dependencies(mut self, deps: &DependencyPaths) -> Self {
        use itertools::Itertools;
        if !deps.is_empty() {
            let tex_inputs = format!("{}", deps.iter().map(|p| p.display()).format(","));
            self.inner_cmd_mut().env("TEXINPUTS", tex_inputs);
        }
        self
    }

    fn finish(self) -> Engine;
}
