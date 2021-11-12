use crate::name::Name;
use crate::util::Map;

use super::{
    IntNiches, Layout, TagLayout, TagPath, TaggedLayout, ValueProj, VariantLayout, VariantRepr,
};

pub struct ExtractedNiche {
    pub path: TagPath,
    pub niche: IntNiches,
}

impl ExtractedNiche {
    pub fn empty_path(niche: IntNiches) -> Self {
        Self { path: TagPath::empty(), niche }
    }
}

impl Layout {
    pub fn extract_niche(&mut self, count: u64) -> Result<ExtractedNiche, ()> {
        match self {
            Layout::U64(niches) | Layout::Ptr { pointee: _, niches } => {
                niches.remove_some_values_mut(count).map(ExtractedNiche::empty_path)
            }
            Layout::Aggregate { fields } => {
                let layouts = fields.iter_mut().map(|(n, l)| with_field_proj(n.clone(), l));
                extract_niches_from_many(layouts, count)
            }
            // FIXME: does this need a projection?
            Layout::Variant(VariantLayout::Single { field }) => field.extract_niche(count),
            Layout::Variant(VariantLayout::Tagged(TaggedLayout { tag, variants })) => {
                if let Ok(niche) = tag.extract_niche(count) {
                    return Ok(ExtractedNiche { path: TagPath::singleton(ValueProj::Tag), niche });
                }
                extract_niches_from_variants(variants, count, tag.as_variant_repr())
            }
            Layout::Recursive(body) => body.extract_niche(count),
            Layout::RecurId(_) => Err(()),
        }
    }
}

fn with_field_proj<T>(field: Name, other: T) -> (TagPath, T) {
    (TagPath::singleton(ValueProj::Field(field)), other)
}
fn with_variant_proj<T>(repr: VariantRepr, name: Name, other: T) -> (TagPath, T) {
    (TagPath::singleton(ValueProj::Variant { repr, name }), other)
}

pub fn extract_niches_from_variants(
    variants: &mut Map<Name, Layout>,
    count: u64,
    repr: VariantRepr,
) -> Result<ExtractedNiche, ()> {
    let layouts = variants.iter_mut().map(|(n, l)| with_variant_proj(repr, n.clone(), l));
    extract_niches_from_many(layouts, count)
}

fn extract_niches_from_many<'a, I>(layouts: I, count: u64) -> Result<ExtractedNiche, ()>
where
    I: Iterator<Item = (TagPath, &'a mut Layout)>,
{
    for (this_path, lyt) in layouts {
        if let Ok(ExtractedNiche { path: inner_path, niche }) = lyt.extract_niche(count) {
            return Ok(ExtractedNiche { path: inner_path.with_outer_path(this_path), niche });
        }
    }
    Err(())
}

impl TagLayout {
    pub fn extract_niche(&mut self, count: u64) -> Result<IntNiches, ()> {
        match self {
            TagLayout::Direct { values: _, niches } => niches.remove_some_values_mut(count),
            TagLayout::Niche { path: _, values: _ } => Err(()),
        }
    }
}
