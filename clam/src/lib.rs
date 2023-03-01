use std::process::Command;

pub use clam_macro::Options;

pub trait Options {
    fn apply(self, cmd: &mut Command);
}

pub trait ArgValue {
    fn set_cmd_arg(&self, name: &str, cmd: &mut Command);
}

impl ArgValue for bool {
    fn set_cmd_arg(&self, name: &str, cmd: &mut Command) {
        if *self {
            cmd.arg(name);
        }
    }
}

macro_rules! arg_value_basic_types {
    ($($type:ty),*) => {
        $(
            impl ArgValue for $type {
                fn set_cmd_arg(&self, name: &str, cmd: &mut Command) {
                    cmd.args(&[name, &self.to_string()]);
                }
            }
        )*
    }
}

arg_value_basic_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl<T: ArgValue> ArgValue for Option<T> {
    fn set_cmd_arg(&self, name: &str, cmd: &mut Command) {
        if let Some(inner) = self {
            inner.set_cmd_arg(name, cmd);
        }
    }
}

impl ArgValue for std::path::Path {
    fn set_cmd_arg(&self, name: &str, cmd: &mut Command) {
        let name: &std::ffi::OsStr = name.as_ref();
        cmd.args(&[name, &self.as_ref()]);
    }
}

impl ArgValue for std::path::PathBuf {
    fn set_cmd_arg(&self, name: &str, cmd: &mut Command) {
        let name: &std::ffi::OsStr = name.as_ref();
        cmd.args(&[name, &self.as_ref()]);
    }
}

impl ArgValue for str {
    fn set_cmd_arg(&self, name: &str, cmd: &mut Command) {
        cmd.args(&[name, &self]);
    }
}

impl ArgValue for String {
    fn set_cmd_arg(&self, name: &str, cmd: &mut Command) {
        cmd.args(&[name, &self]);
    }
}

impl<T: ArgValue> ArgValue for Vec<T> {
    fn set_cmd_arg(&self, _name: &str, _cmd: &mut Command) {
        ()
    }
}
