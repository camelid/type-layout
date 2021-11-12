use std::fmt::{Display, Formatter, Result};

use crate::util::display_map;

use super::*;

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Layout::U64(niches) => write!(f, "U64(niches: {})", niches),
            Layout::Ptr { pointee, niches } => {
                write!(f, "Ptr(pointee: {}, niches: {})", pointee, niches)
            }
            Layout::Aggregate { fields } => write!(f, "Aggregate {}", display_map(fields.iter())),
            Layout::Variant(lyt) => write!(f, "Variant({})", lyt),
            Layout::Recursive(body) => write!(f, "Recursive({})", body),
            Layout::RecurId(k) => write!(f, "recur{}", k),
        }
    }
}

impl Display for VariantLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            VariantLayout::Single { field } => write!(f, "Single(field: {})", field),
            VariantLayout::Tagged(lyt) => write!(f, "Tagged({})", lyt),
        }
    }
}

impl Display for TaggedLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { tag, variants } = self;
        writeln!(f, "tag: {}, variants:", tag)?;
        for (variant, field) in variants {
            writeln!(f, "| {} => {}", variant, field)?;
        }
        Ok(())
    }
}

impl Display for TagLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            TagLayout::Direct { values, niches } => {
                write!(f, "Direct(values: {}, niches: {})", display_map(values.iter()), niches)
            }
            TagLayout::Niche { path, values } => {
                write!(f, "Niche(path: {}, values: {})", path, display_map(values.iter()))
            }
        }
    }
}
