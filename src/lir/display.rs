use std::fmt::{Display, Formatter, Result};

use crate::util::display_map_like;

use super::*;

const PRINT_VAR_TYPES: bool = false;

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::Var(v) => v.fmt(f),

            Expr::U64(n) => write!(f, "{}_u64", n),
            Expr::Record(fields) => {
                write!(f, "{}", display_map_like(fields.iter(), " = ", ", "))
            }
            Expr::UntaggedUnion { ty, field, value } => {
                write!(f, "(<{} = {}> as {})", field, value, ty)
            }

            Expr::Box(val) => write!(f, "Box({})", val),
            Expr::Deref(ptr) => write!(f, "Deref({})", ptr),

            Expr::Select { record, field } => write!(f, "({}).{}", record, field),
            Expr::Switch { subj, cases, default } => write!(
                f,
                "switch {} {{\n{}{}\n}}",
                subj,
                cases
                    .iter()
                    .map(|(p, e)| format!("{} => {{\n{}\n}}", p, e))
                    .intersperse("\n".to_owned())
                    .collect::<String>(),
                default
                    .as_ref()
                    .map(|e| format!("\n_ => {{\n{}\n}}", e))
                    .unwrap_or_else(String::new)
            ),
            Expr::Let { binder, value, body } => {
                write!(f, "let {} = {}\nin  {}", binder, value, body)
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::U64(n) => write!(f, "{}_u64", n),
            Value::Record(fields) => write!(f, "{}", display_map_like(fields.iter(), " = ", ", ")),
            Value::Box(val) => write!(f, "Box({})", val),
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
            Ty::Ptr(pointee) => write!(f, "Ptr({})", pointee),
            Ty::Record(fields) => write!(f, "{}", display_map_like(fields.iter(), " : ", ", ")),
            Ty::UntaggedUnion(fields) => {
                write!(f, "union {}", display_map_like(fields.iter(), " : ", " | "))
            }
            Ty::Recursive(body) => write!(f, "µ. {}", body),
            Ty::RecurId(k) => write!(f, "{}", k),
        }
    }
}
