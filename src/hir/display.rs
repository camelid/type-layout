use std::fmt::{Display, Formatter, Result};

use crate::util::display_map_like;

use super::*;

const PRINT_VAR_TYPES: bool = false;

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::Var(v) => v.fmt(f),
            Expr::U64(u) => write!(f, "{}_u64", u),
            Expr::Box(boxed) => write!(f, "box({})", boxed),
            Expr::Record(fields) => {
                write!(f, "{}", display_map_like(fields.iter(), " = ", ", "))
            }
            Expr::Variant { ty, variant, field } => {
                write!(f, "(<{} = {}> as {})", variant, field, ty)
            }
            Expr::Fold { ty, value } => write!(f, "fold [{}] ({})", ty, value),
            Expr::Unfold { ty, value } => write!(f, "unfold [{}] ({})", ty, value),
            Expr::Let { binder, value, body } => {
                write!(f, "let {} = {}\nin  {}", binder, value, body)
            }
            Expr::Match { subj, cases } => write!(
                f,
                "match {} {{\n{}\n}}",
                subj,
                cases
                    .iter()
                    .map(|(p, e)| format!("{} => {{\n{}\n}}", p, e))
                    .intersperse("\n".to_owned())
                    .collect::<String>()
            ),
        }
    }
}

impl Display for Pat {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Pat::Variant { ty, variant, field } => {
                write!(f, "(<{} = {}> as {})", variant, field, ty)
            }
        }
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { name, ty } = self;
        if PRINT_VAR_TYPES {
            write!(f, "({} : {})", name, ty)
        } else {
            write!(f, "{}", name)
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Ty::U64 => write!(f, "U64"),
            Ty::Box(boxed) => write!(f, "Box[{}]", boxed),
            Ty::Record(fields) => write!(f, "{}", display_map_like(fields.iter(), " : ", ", ")),
            Ty::Variant(variants) => write!(
                f,
                "< {} >",
                variants
                    .iter()
                    .map(|(n, t)| format!("{} of {}", n, t))
                    .intersperse(" | ".into())
                    .collect::<String>()
            ),
            Ty::Recursive(body) => write!(f, "µ. {}", body),
            Ty::Named(name) => write!(f, "{}", name),
        }
    }
}
