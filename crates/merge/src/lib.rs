extern crate self as merge;

pub use merge_macros::Merge;

pub trait Merge {
    fn merge_left(&mut self, other: Self) -> &mut Self;

    fn merge_right(&mut self, other: Self) -> &mut Self;
}

impl<T> Merge for Option<T> {
    fn merge_left(&mut self, other: Self) -> &mut Self {
        match self {
            Some(_) => (),
            None => {
                *self = other;
            }
        };
        self
    }

    fn merge_right(&mut self, other: Self) -> &mut Self {
        match other {
            Some(_) => {
                *self = other;
            }
            None => (),
        };
        self
    }
}

impl<K: Ord, V: Merge> Merge for std::collections::BTreeMap<K, V> {
    fn merge_left(&mut self, other: Self) -> &mut Self {
        use std::collections::btree_map::Entry;
        for (k, v) in other.into_iter() {
            match self.entry(k) {
                Entry::Vacant(e) => {
                    e.insert(v);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().merge_left(v);
                }
            }
        }
        self
    }

    fn merge_right(&mut self, other: Self) -> &mut Self {
        use std::collections::btree_map::Entry;
        for (k, v) in other.into_iter() {
            match self.entry(k) {
                Entry::Vacant(e) => {
                    e.insert(v);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().merge_right(v);
                }
            }
        }
        self
    }
}

impl<T> Merge for Vec<T> {
    fn merge_left(&mut self, other: Self) -> &mut Self {
        self.extend(other.into_iter());
        self
    }

    fn merge_right(&mut self, other: Self) -> &mut Self {
        self.extend(other.into_iter());
        self
    }
}

impl<K: std::hash::Hash + Eq + PartialEq, V: Merge> Merge for std::collections::HashMap<K, V> {
    fn merge_left(&mut self, other: Self) -> &mut Self {
        use std::collections::hash_map::Entry;
        for (k, v) in other.into_iter() {
            match self.entry(k) {
                Entry::Vacant(e) => {
                    e.insert(v);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().merge_left(v);
                }
            }
        }
        self
    }

    fn merge_right(&mut self, other: Self) -> &mut Self {
        use std::collections::hash_map::Entry;
        for (k, v) in other.into_iter() {
            match self.entry(k) {
                Entry::Vacant(e) => {
                    e.insert(v);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().merge_right(v);
                }
            }
        }
        self
    }
}

impl<'a> Merge for &'a str {
    fn merge_left(&mut self, _: Self) -> &mut Self {
        self
    }

    fn merge_right(&mut self, other: Self) -> &mut Self {
        *self = other;
        self
    }
}

impl<'a> Merge for &'a std::path::PathBuf {
    fn merge_left(&mut self, _: Self) -> &mut Self {
        self
    }

    fn merge_right(&mut self, other: Self) -> &mut Self {
        *self = other;
        self
    }
}

macro_rules! merge_basic_types {
    ($($t:ty,)*) => {
        $(
            impl Merge for $t {
                fn merge_left(&mut self, _: Self) -> &mut Self {
                    self
                }

                fn merge_right(&mut self, other: Self) -> &mut Self {
                    *self = other;
                    self
                }
            }
        )*
    };
}

merge_basic_types! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    (),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Merge, Debug, PartialEq, Eq)]
    struct S {
        a: i32,
        b: Option<i32>,
    }

    #[test]
    fn merge_left_works1() {
        let mut s1 = S { a: 1, b: None };
        let s2 = S { a: 3, b: Some(4) };
        s1.merge_left(s2);
        assert_eq!(s1, S { a: 1, b: Some(4) })
    }

    #[test]
    fn merge_left_works2() {
        let mut s1 = S { a: 1, b: Some(2) };
        let s2 = S { a: 3, b: Some(4) };
        s1.merge_left(s2);
        assert_eq!(s1, S { a: 1, b: Some(2) })
    }

    #[test]
    fn merge_right_works1() {
        let mut s1 = S { a: 1, b: None };
        let s2 = S { a: 3, b: Some(4) };
        s1.merge_right(s2);
        assert_eq!(s1, S { a: 3, b: Some(4) })
    }

    #[test]
    fn merge_right_works2() {
        let mut s1 = S { a: 1, b: Some(2) };
        let s2 = S { a: 3, b: Some(4) };
        s1.merge_right(s2);
        assert_eq!(s1, S { a: 3, b: Some(4) })
    }
}
