//! High-level IR.

mod display;
mod ty_subst;
mod typeck;

pub(crate) use typeck::validate_ty;

use crate::debruijn::Debruijn;
use crate::name::Name;
use crate::util::Map;

#[derive(Debug, Clone)]
pub enum Expr {
    Var(Var),
    U64(u64),

    Box(Box<Expr>),

    Record(Map<Name, Expr>),
    Variant { ty: Ty, variant: Name, field: Box<Expr> },

    Fold { ty: Ty, value: Box<Expr> },
    Unfold { ty: Ty, value: Box<Expr> },

    Let { binder: Var, value: Box<Expr>, body: Box<Expr> },
    Match { subj: Box<Expr>, cases: Vec<(Pat, Expr)> },
}

#[derive(Debug, Clone)]
pub enum Pat {
    Variant { ty: Ty, variant: Name, field: Var },
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: Name,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub enum Ty {
    U64,
    Box(Box<Ty>),

    Record(Map<Name, Ty>),
    Variant(Map<Name, Ty>),

    Recursive(Box<Ty>),
    Named(Debruijn),
}

impl Ty {
    fn as_recursive(&self) -> Option<&Ty> {
        match self {
            Ty::Recursive(body) => Some(body),
            _ => None,
        }
    }
}
