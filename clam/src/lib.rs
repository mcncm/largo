pub trait Command {
    fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self;

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>;
}

impl Command for std::process::Command {
    fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.arg(arg)
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.args(args)
    }
}

#[cfg(feature = "async-process")]
impl Command for async_process::Command {
    fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.arg(arg)
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.args(args)
    }
}

pub use clam_macro::Options;

pub trait Options {
    fn apply<C: Command>(self, cmd: &mut C);
}

pub trait ArgValue {
    fn set_cmd_arg<C: Command>(&self, name: &str, cmd: &mut C);
}

impl ArgValue for bool {
    fn set_cmd_arg<C: Command>(&self, name: &str, cmd: &mut C) {
        if *self {
            cmd.arg(name);
        }
    }
}

macro_rules! arg_value_basic_types {
    ($($type:ty),*) => {
        $(
            impl ArgValue for $type {
                fn set_cmd_arg<C: Command>(&self, name: &str, cmd: &mut C) {
                    cmd.args(&[name, &self.to_string()]);
                }
            }
        )*
    }
}

arg_value_basic_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl<T: ArgValue> ArgValue for Option<T> {
    fn set_cmd_arg<C: Command>(&self, name: &str, cmd: &mut C) {
        if let Some(inner) = self {
            inner.set_cmd_arg(name, cmd);
        }
    }
}

impl ArgValue for std::path::Path {
    fn set_cmd_arg<C: Command>(&self, name: &str, cmd: &mut C) {
        let name: &std::ffi::OsStr = name.as_ref();
        cmd.args(&[name, &self.as_ref()]);
    }
}

impl ArgValue for std::path::PathBuf {
    fn set_cmd_arg<C: Command>(&self, name: &str, cmd: &mut C) {
        let name: &std::ffi::OsStr = name.as_ref();
        cmd.args(&[name, &self.as_ref()]);
    }
}

impl ArgValue for str {
    fn set_cmd_arg<C: Command>(&self, name: &str, cmd: &mut C) {
        cmd.args(&[name, &self]);
    }
}

impl ArgValue for String {
    fn set_cmd_arg<C: Command>(&self, name: &str, cmd: &mut C) {
        cmd.args(&[name, &self]);
    }
}

impl<T: ArgValue> ArgValue for Vec<T> {
    fn set_cmd_arg<C: Command>(&self, _name: &str, _cmd: &mut C) {
        ()
    }
}
