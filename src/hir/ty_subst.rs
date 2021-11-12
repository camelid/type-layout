use crate::debruijn::Debruijn;
use crate::hir::Ty;

pub(super) type Subst = (Debruijn, Ty);

pub(super) fn subst_ty(subst: Subst, target: Ty) -> Ty {
    match target {
        Ty::U64 => target,
        Ty::Box(mut boxed) => {
            *boxed = subst_ty(subst, *boxed);
            Ty::Box(boxed)
        }
        Ty::Record(fields) => {
            Ty::Record(fields.into_iter().map(|(n, t)| (n, subst_ty(subst.clone(), t))).collect())
        }
        Ty::Variant(variants) => Ty::Variant(
            variants.into_iter().map(|(n, t)| (n, subst_ty(subst.clone(), t))).collect(),
        ),
        Ty::Recursive(mut body) => {
            let subst = (subst.0.shift_by(1), shift_ty(subst.1, 1));
            *body = subst_ty(subst, *body);
            Ty::Recursive(body)
        }
        Ty::Named(this) => {
            if subst.0 == this {
                subst.1
            } else {
                Ty::Named(this)
            }
        }
    }
}

fn shift_ty(ty: Ty, offset: u64) -> Ty {
    shift_ty_inner(ty, offset, Debruijn::ZERO)
}

fn shift_ty_inner(ty: Ty, offset: u64, cutoff: Debruijn) -> Ty {
    match ty {
        Ty::U64 => ty,
        Ty::Box(mut boxed) => {
            *boxed = shift_ty_inner(*boxed, offset, cutoff);
            Ty::Box(boxed)
        }
        Ty::Record(fields) => Ty::Record(
            fields.into_iter().map(|(n, t)| (n, shift_ty_inner(t, offset, cutoff))).collect(),
        ),
        Ty::Variant(variants) => Ty::Variant(
            variants.into_iter().map(|(n, t)| (n, shift_ty_inner(t, offset, cutoff))).collect(),
        ),
        Ty::Recursive(mut body) => {
            *body = shift_ty_inner(*body, offset, cutoff.shift_by(1));
            Ty::Recursive(body)
        }
        Ty::Named(k) => Ty::Named(if k < cutoff { k } else { k.shift_by(offset) }),
    }
}
