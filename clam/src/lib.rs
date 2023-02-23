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

impl<T: ArgValue> ArgValue for Option<T> {
    fn set_cmd_arg(&self, name: &str, cmd: &mut Command) {
        if let Some(inner) = self {
            inner.set_cmd_arg(name, cmd);
        }
    }
}

impl ArgValue for str {
    fn set_cmd_arg(&self, _name: &str, _cmd: &mut Command) {
        ()
    }
}

impl ArgValue for String {
    fn set_cmd_arg(&self, _name: &str, _cmd: &mut Command) {
        ()
    }
}

impl<T: ArgValue> ArgValue for Vec<T> {
    fn set_cmd_arg(&self, _name: &str, _cmd: &mut Command) {
        ()
    }
}
