//! Utilities for building strongly-typed directory structures

pub trait Node: AsRef<std::path::Path> + Sized {}

pub trait SubDir: From<Self::Parent> {
    type Parent: Node;
    type Link: AsRef<std::path::Path>;

    fn parent(self) -> Self::Parent;
}

// macro_rules! typedir_ctx {
//     (sup $Parent:ident in $)
//     (node $Parent:ident / $link:expr => $sub:ident)
// }

// An internal helper macro for assembling subdirectories in a context
// of their parent
#[macro_export]
macro_rules! __parent_ctx {
    ($Parent:ident / ) => {};
    ($Parent:ident / $link:expr => node $Name:ident $({$($subdirs:tt)*})?; $($tail:tt)*) => {
        // Create the node as normal
        $crate::typedir!(node $Name;);

        impl From<$Parent> for $Name {
            fn from(sup: $Parent) -> Self {
                let mut path = sup.0;
                let link: &'static str = $link;
                path.push($link);
                Self(path)
            }
        }

        impl $crate::SubDir for $Name {
            type Parent = $Parent;
            type Link = &'static str;

            fn parent(mut self) -> $Parent {
                self.0.pop();
                $Parent(self.0)
            }
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
        #[derive(Clone, Debug)]
        /// Newtype for typesafe handling of project directory structure
        pub struct $Name(std::path::PathBuf);

        impl AsRef<std::path::Path> for $Name {
            fn as_ref(&self) -> &std::path::Path {
                self.0.as_ref()
            }
        }

        impl $crate::Node for $Name {}

        // Children have *this* node as parent
        $($crate::__parent_ctx!($Name / $($subdirs)*);)?

        // Continue in the tail
        $crate::typedir!($($tail)*);
    };
}
