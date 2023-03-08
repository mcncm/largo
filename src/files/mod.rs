//! Files (and code for building them) that go in the Largo repository.

pub mod packages;

pub const GITIGNORE: &[u8] = include_bytes!("gitignore.txt");
pub const MAIN_LATEX: &[u8] = include_bytes!("main_latex.tex");
