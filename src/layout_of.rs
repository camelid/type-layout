use std::convert::TryInto;

use crate::hir;
use crate::layout::extract::{extract_niches_from_variants, ExtractedNiche};
use crate::layout::{IntNiches, Layout, TagLayout, TaggedLayout, VariantLayout, VariantRepr};
use crate::name::Name;
use crate::util::{expect_singleton_vec, range_values_count, Map};

pub fn layout_of(ty: hir::Ty) -> Layout {
    hir::validate_ty(&ty);
    match ty {
        hir::Ty::U64 => Layout::U64(IntNiches::none()),
        hir::Ty::Box(boxed) => Layout::ptr(layout_of(*boxed)),
        hir::Ty::Record(fields) => Layout::Aggregate {
            fields: fields.into_iter().map(|(n, t)| (n, layout_of(t))).collect(),
        },
        hir::Ty::Variant(variants) => match variants.len() {
            0 => layout_of_empty_type(),
            1 => {
                let field_ty = expect_singleton_vec(variants.into_values().collect());
                layout_of_singleton_variant(field_ty)
            }
            _ => layout_of_multi_variant_type(variants),
        },
        hir::Ty::Recursive(body) => Layout::Recursive(Box::new(layout_of(*body))),
        hir::Ty::Named(k) => Layout::RecurId(k),
    }
}

fn layout_of_empty_type() -> Layout {
    // TODO: layout types containing empty types more efficiently
    Layout::Variant(VariantLayout::Single {
        field: Box::new(Layout::Aggregate { fields: map! {} }),
    })
}

fn layout_of_singleton_variant(field_ty: hir::Ty) -> Layout {
    let field_lyt = layout_of(field_ty);
    Layout::Variant(VariantLayout::Single { field: Box::new(field_lyt) })
}

fn layout_of_multi_variant_type(variants: Map<Name, hir::Ty>) -> Layout {
    let variants = variants.into_iter().map(|(n, t)| (n, layout_of(t))).collect();
    let lyt = match is_nicheable(&variants) {
        Nicheable::Yes { field: _, nullary_variants } => {
            layout_of_tagged_niche_type(variants, nullary_variants)
        }
        Nicheable::No => layout_of_tagged_direct_type(variants),
    };
    Layout::Variant(VariantLayout::Tagged(lyt))
}

fn layout_of_tagged_niche_type(
    mut variants: Map<Name, Layout>,
    nullary_variants: Vec<Name>,
) -> TaggedLayout {
    let needed_tag_values_count: u64 = nullary_variants.len().try_into().unwrap();

    // If the niche extraction is successful, the variants will be transparent.
    let variant_repr = VariantRepr::Transparent;
    // TODO: could just extract from `field` in `Nicheable` and wrap in variant proj
    let ExtractedNiche { path: tag_path, niche } =
        match extract_niches_from_variants(&mut variants, needed_tag_values_count, variant_repr) {
            Ok(niche) => niche,
            // We couldn't find a niche, so we need a direct tag layout.
            Err(()) => return layout_of_tagged_direct_type(variants),
        };
    let niche_range = niche.as_range().unwrap();

    let niche_values_count = range_values_count(niche_range.clone()).unwrap();
    assert_eq!(needed_tag_values_count, niche_values_count);

    let tag_values = nullary_variants.into_iter().zip(niche_range).collect();

    let tag = TagLayout::Niche { path: tag_path, values: tag_values };
    TaggedLayout { tag, variants }
}

fn layout_of_tagged_direct_type(variants: Map<Name, Layout>) -> TaggedLayout {
    let variant_count: u64 = variants.len().try_into().unwrap();

    let max_tag_value = variant_count.checked_sub(1).unwrap();
    let tag_values = variants.keys().cloned().zip(0..=max_tag_value).collect();
    let tag = TagLayout::direct(tag_values);

    TaggedLayout { tag, variants }
}

pub(crate) fn is_nicheable(variants: &Map<Name, Layout>) -> Nicheable<'_> {
    let mut found_field = None;
    let mut nullary_variants = vec![];

    for (variant_name, field) in variants {
        if field.is_zst() {
            nullary_variants.push(variant_name.clone());
        } else {
            if found_field.is_some() {
                // Multiple variants have fields.
                return Nicheable::No;
            } else {
                found_field = Some(field);
            }
        }
    }

    if let Some(field) = found_field {
        // All variants except one have a field.
        Nicheable::Yes { field, nullary_variants }
    } else {
        // No variant has a field, so there's no niche.
        Nicheable::No
    }
}

// TODO: replace with Result?
pub(crate) enum Nicheable<'a> {
    Yes { field: &'a Layout, nullary_variants: Vec<Name> },
    No,
}

impl<'a> Nicheable<'a> {
    pub(crate) fn as_field(self) -> Option<&'a Layout> {
        match self {
            Nicheable::Yes { field, nullary_variants: _ } => Some(field),
            Nicheable::No => None,
        }
    }
}
