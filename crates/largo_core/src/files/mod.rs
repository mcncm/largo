//! Files (and code for building them) that go in the Largo repository.

pub mod packages;

pub const GITIGNORE: &str = include_str!("gitignore.txt");
pub const MAIN_LATEX: &str = include_str!("main_latex.tex");

macro_rules! cachedir_tag_signature {
    () => {
        "Signature: 8a477f597d28d172789f06886806bc55"
    };
}

pub const CACHEDIR_TAG_SIGNATURE: &str = cachedir_tag_signature!();
pub const CACHEDIR_TAG: &str = concat!(
    cachedir_tag_signature!(),
    '\n',
    "# This file is a cache directory tag created by largo.
# For information about cache directory tags, see:
#	https://bford.info/cachedir/",
    '\n',
);
