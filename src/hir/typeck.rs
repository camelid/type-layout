use crate::debruijn::Debruijn;

use super::ty_subst::subst_ty;
use super::{Expr, Ty};

impl Expr {
    pub(crate) fn ty(&self) -> Ty {
        let ty = match self {
            Expr::Var(var) => var.ty.clone(),
            Expr::U64(_) => Ty::U64,
            Expr::Box(boxed) => Ty::Box(Box::new(boxed.ty())),
            Expr::Record(rec) => Ty::Record(rec.iter().map(|(n, e)| (n.clone(), e.ty())).collect()),
            // FIXME: check the variant type too?
            Expr::Variant { ty, variant: _, field: _ } => ty.clone(),
            // FIXME: check types?
            Expr::Fold { ty, value: _ } => ty.clone(),
            Expr::Unfold { ty, value } => {
                let v_ty = value.ty();
                let rec_body = ty.as_recursive().unwrap();
                let subst = (Debruijn::ZERO, v_ty);
                subst_ty(subst, rec_body.clone())
            }
            // FIXME: check types?
            Expr::Let { binder: _, value: _, body } => body.ty(),
            // FIXME: check the subj's and other cases' types too?
            Expr::Match { subj: _, cases } => {
                let (_, first_body) = cases.first().expect("empty match is unsupported");
                first_body.ty()
            }
        };
        validate_ty(&ty);
        ty
    }
}

pub(crate) fn validate_ty(ty: &Ty) {
    validate_ty_helper(Debruijn::ZERO, ty)
}

fn validate_ty_helper(max_recur_id: Debruijn, ty: &Ty) {
    match ty {
        Ty::U64 => {}
        Ty::Box(boxed) => validate_ty_helper(Debruijn::ZERO, boxed),
        Ty::Record(fields) => fields.values().for_each(|t| validate_ty_helper(max_recur_id, t)),
        Ty::Variant(variants) => {
            variants.values().for_each(|t| validate_ty_helper(max_recur_id, t))
        }
        Ty::Recursive(body) => validate_ty_helper(max_recur_id.shift_by(1), body),
        Ty::Named(k) => {
            if *k < max_recur_id {
                panic!("type error: infinite recursive type; insert a Box");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_ty;

    use super::*;

    fn t(src: &str) {
        validate_ty(&parse_ty(src))
    }

    #[test]
    fn valid_types() {
        t("U64");
        t("Box[U64]");
        t("{x:{}}");
        t("<None of {} | Some of <False of {} | True of {}>>");
        t("µX. <Nil of {} | Cons of { hd : {}, tl : Box[X] }>");
        t("µX. Box[X]");
        t("µX. Box[Box[X]]");
        t("µX. Box[{ x : X, y : µY. Box[Y] }]");
    }

    #[test]
    #[should_panic = "infinite recursive type"]
    fn invalid_type_1() {
        t("µX. <Nil of {} | Cons of { hd : {}, tl : X }>");
    }

    #[test]
    #[should_panic = "infinite recursive type"]
    fn invalid_type_2() {
        t("µX. X");
    }

    #[test]
    #[should_panic = "infinite recursive type"]
    fn invalid_type_3() {
        t("Box[µX. X]");
    }

    #[test]
    #[should_panic = "infinite recursive type"]
    fn invalid_type_4() {
        t("µX. Box[{ x : X, y : µY. Y }]");
    }

    #[test]
    #[should_panic = "syntax error encountered"]
    fn invalid_type_5() {
        t("X");
    }
}
