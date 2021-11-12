use crate::name::Name;
use crate::util::list::{Cons, List, Nil};

use super::VariantRepr;

/// The path to the variant tag.
// FIXME: rename to ValuePath?
#[derive(Debug, Clone)]
pub struct TagPath {
    /// **Note:** The path is *reversed*. In other words, if the path is
    /// `<root>.field1.field2`, then the representation will be `[field2, field1]`.
    reversed: List<ValueProj>,
}

impl TagPath {
    pub fn empty() -> Self {
        Self { reversed: Nil }
    }

    pub fn singleton(proj: ValueProj) -> Self {
        Self { reversed: Cons(proj, Box::new(Nil)) }
    }

    pub fn with_outer_path(self, outer: TagPath) -> Self {
        Self { reversed: self.reversed.concat(outer.reversed) }
    }

    pub fn rfold<R, F>(self, init: R, f: F) -> R
    where
        F: FnMut(R, ValueProj) -> R,
    {
        self.reversed.into_iter().fold(init, f)
    }
}

#[derive(Debug, Clone)]
pub enum ValueProj {
    Field(Name),
    Variant { repr: VariantRepr, name: Name },
    Tag,
}

// DISPLAY //

impl std::fmt::Display for TagPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reversed.fmt(f)
    }
}

// FIXME: this impl is public but is an implementation detail
impl std::fmt::Display for List<ValueProj> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Nil => write!(f, "{{root}}"),
            Cons(ValueProj::Field(field), prev) => write!(f, "{}.{}", prev, field),
            Cons(ValueProj::Variant { repr, name }, prev) => {
                write!(f, "({} as({}) {})", prev, repr.descr(), name)
            }
            Cons(ValueProj::Tag, prev) => write!(f, "{}.{{tag}}", prev),
        }
    }
}
