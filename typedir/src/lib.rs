//! Utilities for building strongly-typed directory structures

use std::marker::PhantomData;

/// This module mimics `#[sealed]` traits, which are not yet implemented in
/// rustc.
mod __sealed {
    use super::{Link, Node};
    pub trait Extend<L: Link, T> {}
    pub trait AsPath<N: Node> {}
}

pub trait Node: Sized {}

pub trait Link {}

impl<T> Link for T {}

pub trait Child<P: Node, L: Link>: Node {
    fn link<'a>(l: &'a L) -> &'a std::path::Path;
}

pub trait Extend<L: Link, T>: __sealed::Extend<L, T> {
    fn extend(self, link: L) -> T;
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct PathBuf<N: Node> {
    path: std::path::PathBuf,
    m: PhantomData<N>,
}
impl<N: Node> PathBuf<N> {
    pub fn new<I: Into<std::path::PathBuf>>(_m: N, path: I) -> Self {
        Self {
            path: path.into(),
            m: PhantomData,
        }
    }
}

impl<L, P, C> __sealed::Extend<L, PathBuf<C>> for PathBuf<P>
where
    L: Link,
    P: Node,
    C: Child<P, L>,
{
}
impl<L, P, C> Extend<L, PathBuf<C>> for PathBuf<P>
where
    L: Link,
    P: Node,
    C: Child<P, L>,
{
    fn extend(mut self, link: L) -> PathBuf<C> {
        self.path.push(C::link(&link));
        PathBuf {
            path: self.path,
            m: PhantomData,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct PathRef<'a, N: Node> {
    path: &'a mut std::path::PathBuf,
    m: PhantomData<N>,
}

impl<'a1, 'a2, N1, N2, L> __sealed::Extend<L, PathRef<'a2, N2>> for &'a2 mut PathRef<'a1, N1>
where
    'a1: 'a2,
    N1: Node,
    N2: Node + Child<N1, L>,
    L: Link,
{
}
impl<'a1, 'a2, N1, N2, L> Extend<L, PathRef<'a2, N2>> for &'a2 mut PathRef<'a1, N1>
where
    'a1: 'a2,
    N1: Node,
    N2: Node + Child<N1, L>,
    L: Link,
{
    fn extend(self, link: L) -> PathRef<'a2, N2> {
        self.path.push(N2::link(&link));
        // SAFETY: TODO
        unsafe {
            let ptr = self.path as *mut std::path::PathBuf;
            PathRef {
                path: (&mut *ptr),
                m: PhantomData,
            }
        }
    }
}

impl<'a, N1, N2, L> __sealed::Extend<L, PathRef<'a, N2>> for &'a mut PathBuf<N1>
where
    N1: Node,
    N2: Node + Child<N1, L>,
    L: Link,
{
}
impl<'a, N1, N2, L> Extend<L, PathRef<'a, N2>> for &'a mut PathBuf<N1>
where
    N1: Node,
    N2: Node + Child<N1, L>,
    L: Link,
{
    fn extend(self, link: L) -> PathRef<'a, N2> {
        self.path.push(N2::link(&link));
        // SAFETY: TODO
        unsafe {
            let ptr = (&mut self.path) as *mut std::path::PathBuf;
            PathRef {
                path: (&mut *ptr),
                m: PhantomData,
            }
        }
    }
}

impl<'a, N: Node> std::ops::Drop for PathRef<'a, N> {
    fn drop(&mut self) {
        self.path.pop();
    }
}

#[macro_export]
macro_rules! dir {
    ($root:expr) => { $root };
    ($root:expr => $sub:ty $(=>$($tail:tt)*)*) => {
        {
            let r = <$sub>::from(dir!($root));
            dir!(r $(/$($tail)*)*)
        }

    };
}

impl<N: Node> std::ops::Deref for PathBuf<N> {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl<N: Node> AsRef<std::path::Path> for PathBuf<N> {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}

impl<N: Node> std::borrow::Borrow<std::path::Path> for PathBuf<N> {
    fn borrow(&self) -> &std::path::Path {
        &self.path
    }
}

impl<'a, N: Node> std::ops::Deref for PathRef<'a, N> {
    type Target = std::path::Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl<'a, N: Node> AsRef<std::path::Path> for PathRef<'a, N> {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}

impl<'a, N: Node> std::borrow::Borrow<std::path::Path> for PathRef<'a, N> {
    fn borrow(&self) -> &std::path::Path {
        &self.path
    }
}

pub trait AsPath<N: Node>:
    std::ops::Deref<Target = std::path::Path>
    + AsRef<std::path::Path>
    + std::ops::Deref
    + __sealed::AsPath<N>
{
}

impl<N: Node> __sealed::AsPath<N> for PathBuf<N> {}
impl<N: Node> AsPath<N> for PathBuf<N> {}

impl<'a, N: Node> __sealed::AsPath<N> for PathRef<'a, N> {}
impl<'a, N: Node> AsPath<N> for PathRef<'a, N> {}

// An internal helper macro for assembling subdirectories in a context
// of their parent
#[macro_export]
macro_rules! __parent_ctx {
    ($Parent:ident / ) => {};
    ($Parent:ident / $link:expr => node $Name:ident $({$($subdirs:tt)*})?; $($tail:tt)*) => {

        $crate::typedir!(node $Name;);

        impl $crate::Child<$Parent, ()> for $Name {
            fn link(_: &()) -> &::std::path::Path { ($link).as_ref() }
        }

        // Children have *this* node as parent
        $crate::__parent_ctx!($Name / $($($subdirs)*)?);

        // Continue in the tail with the same parent context
        $crate::__parent_ctx!($Parent / $($tail)*);
    };
    ($Parent:ident / forall $x:ident : $type:ty , $e:expr => node $Name:ident $({$($subdirs:tt)*})?; $($tail:tt)*) => {

        $crate::typedir!(node $Name;);

        impl $crate::Child<$Parent, $type> for $Name {
            fn link<'a>($x: &'a $type) -> &'a ::std::path::Path { ($e).as_ref() }
        }

        // Children have *this* node as parent
        $crate::__parent_ctx!($Name / $($($subdirs)*)?);

        // Continue in the tail with the same parent context
        $crate::__parent_ctx!($Parent / $($tail)*);
    };
}

#[macro_export]
macro_rules! typedir {
    // Empty `tt`: nothing to do!
    () => {};
    // Node with subnodes
    (node $Name:ident $({$($subdirs:tt)*})?; $($tail:tt)*) => {
        #[derive(Debug, Clone, Copy)]
        /// Newtype for typesafe handling of project directory structure
        pub struct $Name(());

        impl $crate::Node for $Name {}

        // Children have *this* node as parent
        $($crate::__parent_ctx!($Name / $($subdirs)*);)?

        // Continue in the tail
        $crate::typedir!($($tail)*);
    };
}

#[macro_export]
macro_rules! path {
    ($root:expr) => {
        $root
    };
    ($root:expr => $segment:ty) => {
        $crate::Extend::<_, $crate::PathBuf<$segment>>::extend($root, ())
    };
    ($root:expr => $segment:ty => $($tail:tt)*) => {
        $crate::path!($crate::path!($root:expr => $segment:ty) => $($tail::tt)*)
    };
}

#[macro_export]
macro_rules! pathref {
    ($root:expr => $segment:ty) => {
        $crate::Extend::<_, $crate::PathRef<$segment>>::extend(&mut $root, ())
    };
    ($root:expr => $segment:ty => $($tail:tt)*) => {
        $crate::pathref!($crate::pathref!($root:expr => $segment:ty) => $($tail::tt)*)
    };
}

#[cfg(test)]
mod tests {
    use super::{PathBuf as P, PathRef as R, *};

    /// Test directory structure
    mod proj {
        use super::*;

        pub const ROOT: &'static str = "/my/root/path";
        pub const SRC: &'static str = "src";
        pub const MAIN_RS: &'static str = "main.rs";
        pub const TARGET: &'static str = "target";

        typedir! {
            node Root {
                SRC => node Src {
                    MAIN_RS => node MainRs;
                };
                TARGET => node Target {
                    forall s: &str, s => node Profile;
                };
            };
        }

        impl P<Root> {
            /// This should be accessible to tests, but `Root` itself should not
            /// be constructible
            pub fn init() -> Self {
                Self::new(Root(()), ROOT)
            }
        }
    }

    use proj::*;

    macro_rules! assert_path_eq {
        ($path:expr, $string:expr) => {
            assert_eq!(($path).to_str().expect("path was not a string"), $string);
        };
    }

    #[test]
    #[cfg(unix)]
    fn pathbuf_builds_correct_paths() {
        let root = P::<Root>::init();
        assert_path_eq!(root, ROOT);
        let src: P<Src> = root.extend(());
        assert_path_eq!(src, &format!("{}/{}", ROOT, SRC));
        let main_rs: P<MainRs> = src.extend(());
        assert_path_eq!(main_rs, &format!("{}/{}/{}", ROOT, SRC, MAIN_RS));
    }

    // This behavior is *correct*, but I don't want to have to these explicit drops.
    #[test]
    #[cfg(unix)]
    fn pathref_builds_correct_paths_explicit_drop() {
        let mut root = P::<Root>::init();
        assert_path_eq!(root, ROOT);
        let mut src: R<Src> = (&mut root).extend(());
        assert_path_eq!(src, &format!("{}/{}", ROOT, SRC));
        let main_rs: R<MainRs> = (&mut src).extend(());
        assert_path_eq!(main_rs, &format!("{}/{}/{}", ROOT, SRC, MAIN_RS));
        drop(main_rs);
        assert_path_eq!(src, &format!("{}/{}", ROOT, SRC));
        drop(src);
        assert_path_eq!(root, ROOT);
    }

    // Same test, but with the macro
    #[test]
    #[cfg(unix)]
    fn pathref_builds_correct_paths_pathref_macro() {
        let mut root = P::<Root>::init();
        assert_path_eq!(root, ROOT);
        {
            let mut src = pathref!(root => Src);
            assert_path_eq!(src, &format!("{}/{}", ROOT, SRC));
            {
                let main_rs = pathref!(src => MainRs);
                assert_path_eq!(main_rs, &format!("{}/{}/{}", ROOT, SRC, MAIN_RS));
            }
            assert_path_eq!(src, &format!("{}/{}", ROOT, SRC));
        }
        assert_path_eq!(root, ROOT);
    }

    #[test]
    #[cfg(unix)]
    fn simple_parametric_paths_work() {
        let root = P::<Root>::init();
        let target = path!(root => Target);
        // No macro for this yet
        let profile: P<Profile> = target.extend("someprofile");
        assert_path_eq!(profile, &format!("{}/{}/{}", ROOT, TARGET, "someprofile"));
    }
}
