//! TeX-build-time variables (macros, that is) defined by Largo.

use crate::{conf::ProfileName, dirs};

use typedir::PathBuf as P;

/// Variables available at TeX run time
#[derive(Debug, Clone)]
pub struct LargoVars<'a> {
    pub profile: ProfileName<'a>,
    pub bibliography: Option<&'a str>,
    pub output_directory: &'a P<dirs::BuildDir>,
}

// For use in `LargoVars::to_defs`
macro_rules! write_lv {
    ($defs:expr, $var:expr, $val:expr) => {
        write!($defs, r#"\def\Largo{}{{{}}}"#, $var, $val).expect("internal error");
    };
}

impl<'a> LargoVars<'a> {
    pub fn to_defs(self) -> String {
        use std::fmt::Write;
        let mut defs = String::new();
        {
            let defs = &mut defs;
            write_lv!(defs, "Profile", &self.profile);
            if let Some(bib) = self.bibliography {
                write_lv!(defs, "Bibliography", bib);
            }
            write_lv!(defs, "OutputDirectory", &self.output_directory.display());
        }
        defs
    }
}
