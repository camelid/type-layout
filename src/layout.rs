mod display;
pub mod extract;
mod niches;
mod path;

pub use self::niches::IntNiches;
pub use self::path::{TagPath, ValueProj};

use crate::debruijn::Debruijn;
use crate::name::Name;
use crate::util::Map;

#[derive(Debug, Clone)]
pub enum Layout {
    U64(IntNiches),
    /// **Note:** Use [`Layout::ptr()`] to construct this layout.
    Ptr {
        pointee: Box<Layout>,
        niches: IntNiches,
    },

    Aggregate {
        fields: Map<Name, Layout>,
    },
    Variant(VariantLayout),

    /// This functions as a "marker" layout.
    ///
    /// Its sole raison d'être is to provide a "backreference" target for
    /// [`Layout::RecurId`]s.
    Recursive(Box<Layout>),
    RecurId(Debruijn),
}

#[derive(Debug, Clone)]
pub enum VariantLayout {
    // TODO: rename to Transparent?
    Single { field: Box<Layout> },
    Tagged(TaggedLayout),
}

#[derive(Debug, Clone)]
pub struct TaggedLayout {
    pub tag: TagLayout,
    pub variants: Map<Name, Layout>,
}

#[derive(Debug, Clone)]
pub enum TagLayout {
    Direct { values: Map<Name, u64>, niches: IntNiches },
    Niche { path: TagPath, values: Map<Name, u64> },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VariantRepr {
    /// Has a `data` field holding the variant's field.
    Wrapper,
    /// Has the same representation as its field.
    Transparent,
}

impl VariantRepr {
    pub fn descr(self) -> &'static str {
        match self {
            VariantRepr::Wrapper => "wrapper",
            VariantRepr::Transparent => "transparent",
        }
    }
}

impl Layout {
    pub fn ptr(pointee: Layout) -> Self {
        Layout::Ptr { pointee: Box::new(pointee), niches: IntNiches::range(0..=0) }
    }

    #[track_caller]
    pub fn expect_variant(self) -> VariantLayout {
        match self {
            Layout::Variant(layout) => layout,
            _ => panic!(),
        }
    }

    pub fn is_zst(&self) -> bool {
        match self {
            Layout::U64(..) | Layout::Ptr { .. } => false,
            Layout::Aggregate { fields } => fields.values().all(Layout::is_zst),
            Layout::Variant(VariantLayout::Single { field }) => field.is_zst(),
            Layout::Variant(VariantLayout::Tagged(TaggedLayout { tag, variants })) => {
                tag.is_zst() && variants.values().all(Layout::is_zst)
            }
            Layout::Recursive(body) => body.is_zst(),
            // FIXME: is this correct?
            Layout::RecurId(_) => false,
        }
    }
}

impl TagLayout {
    pub fn direct(values: Map<Name, u64>) -> Self {
        let niches = IntNiches::range(0..=u64::MAX);
        let niches =
            values.values().fold(niches, |niches, &value| niches.remove_value(value).unwrap());
        Self::Direct { values, niches }
    }

    pub fn is_zst(&self) -> bool {
        match self {
            TagLayout::Direct { .. } => false,
            TagLayout::Niche { .. } => true,
        }
    }

    pub fn niches(&self) -> IntNiches {
        match self {
            TagLayout::Direct { niches, values: _ } => niches.clone(),
            TagLayout::Niche { path: _, values: _ } => IntNiches::none(),
        }
    }

    pub fn as_variant_repr(&self) -> VariantRepr {
        match self {
            TagLayout::Direct { .. } => VariantRepr::Wrapper,
            TagLayout::Niche { .. } => VariantRepr::Transparent,
        }
    }
}
