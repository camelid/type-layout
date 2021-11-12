//! Low-level IR.

mod display;
mod size;

use crate::{debruijn::Debruijn, name::Name, util::Map};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Var(Var),

    U64(u64),
    Record(Map<Name, Expr>),
    UntaggedUnion { ty: Ty, field: Name, value: Box<Expr> },

    Box(Box<Expr>),
    Deref(Box<Expr>),

    Select { record: Box<Expr>, field: Name },

    Switch { subj: Var, cases: Map<u64, Expr>, default: Option<Box<Expr>> },

    Let { binder: Var, value: Box<Expr>, body: Box<Expr> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    U64(u64),
    Record(Map<Name, Value>),
    Box(Box<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    pub name: Name,
    pub ty: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    U64,
    Ptr(Box<Ty>),

    Record(Map<Name, Ty>),
    UntaggedUnion(Map<Name, Ty>),

    Recursive(Box<Ty>),
    RecurId(Debruijn),
}

impl Expr {
    pub fn ty(&self) -> Ty {
        match self {
            Expr::Var(var) => var.ty.clone(),
            Expr::U64(_) => Ty::U64,
            Expr::Record(fields) => {
                Ty::Record(fields.iter().map(|(n, e)| (n.clone(), e.ty())).collect())
            }
            // FIXME: check types?
            Expr::UntaggedUnion { ty, field: _, value: _ } => ty.clone(),
            Expr::Box(val) => Ty::Ptr(Box::new(val.ty())),
            Expr::Deref(ptr) => match ptr.ty() {
                Ty::Ptr(val_ty) => *val_ty,
                _ => panic!(),
            },
            Expr::Select { record, field } => match record.ty() {
                Ty::Record(field_tys) => field_tys[field].clone(),
                _ => panic!(),
            },
            // FIXME: check types?
            Expr::Switch { subj: _, cases, default: _ } => {
                let first_body = cases.values().next().unwrap();
                first_body.ty()
            }
            // FIXME: check types?
            Expr::Let { binder: _, value: _, body } => body.ty(),
        }
    }
}

impl Var {
    pub fn new(name: Name, ty: Ty) -> Self {
        Self { name, ty }
    }

    pub fn temp(idx: u64, ty: Ty) -> Self {
        Self { name: Name::Temp(idx), ty }
    }
}

impl Ty {
    // FIXME: remove this and use Layout::is_zst instead
    pub fn is_zst(&self) -> bool {
        match self {
            Ty::U64 | Ty::Ptr(_) => false,
            Ty::Record(fields) => fields.values().all(|t| t.is_zst()),
            Ty::UntaggedUnion(fields) => fields.values().all(|t| t.is_zst()),
            Ty::Recursive(body) => body.is_zst(),
            // FIXME: is this correct?
            Ty::RecurId(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zst() {
        assert!(Ty::Record(map! {}).is_zst());
        assert!(Ty::Record(map! { "x" => Ty::Record(map!{}), "y" => Ty::Record(map!{}) }).is_zst());
        assert!(Ty::UntaggedUnion(map! {}).is_zst());
        assert!(Ty::UntaggedUnion(map! { "x" => Ty::Record(map!{}), "y" => Ty::Record(map!{}) })
            .is_zst());

        assert!(!Ty::U64.is_zst());
        assert!(!Ty::UntaggedUnion(map! { "x" => Ty::Record(map! { "x" => Ty::U64 }) }).is_zst())
    }
}
