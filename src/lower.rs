use std::collections::BTreeMap;

use crate::hir;
use crate::layout::{
    Layout, TagLayout, TagPath, TaggedLayout, ValueProj, VariantLayout, VariantRepr,
};
use crate::layout_of::{is_nicheable, layout_of};
use crate::lir;
use crate::name::Name;
use crate::util::expect_singleton_vec;

#[derive(Debug)]
pub struct Ctxt {
    next_temp_var: u64,
}

impl Ctxt {
    pub fn new() -> Self {
        Self { next_temp_var: 0 }
    }

    fn temp_var(&mut self, ty: lir::Ty) -> lir::Var {
        let idx = self.next_temp_var;
        self.next_temp_var += 1;
        lir::Var::temp(idx, ty)
    }
}

pub fn lower_root_expr(expr: hir::Expr) -> lir::Expr {
    lower_expr(&mut Ctxt::new(), expr)
}

fn lower_expr(cx: &mut Ctxt, expr: hir::Expr) -> lir::Expr {
    let hir_ty = expr.ty();
    let layout = layout_of(hir_ty);

    match expr {
        hir::Expr::Var(var) => lir::Expr::Var(lower_var(var)),
        hir::Expr::U64(u) => lir::Expr::U64(u),
        hir::Expr::Box(boxed) => lir::Expr::Box(Box::new(lower_expr(cx, *boxed))),
        hir::Expr::Record(fields) => {
            lir::Expr::Record(fields.into_iter().map(|(n, e)| (n, lower_expr(cx, e))).collect())
        }
        hir::Expr::Variant { ty: _, variant, field } => {
            let variant_layout = layout.expect_variant();
            lower_variant_expr(cx, variant_layout, variant, *field)
        }
        hir::Expr::Fold { ty: _, value } => lower_expr(cx, *value),
        hir::Expr::Unfold { ty: _, value } => lower_expr(cx, *value),
        hir::Expr::Let { binder, value, body } => lir::Expr::Let {
            binder: lower_var(binder),
            value: Box::new(lower_expr(cx, *value)),
            body: Box::new(lower_expr(cx, *body)),
        },
        hir::Expr::Match { subj, cases } => lower_match(cx, *subj, cases),
    }
}

fn lower_variant_expr(
    cx: &mut Ctxt,
    layout: VariantLayout,
    variant: Name,
    field: hir::Expr,
) -> lir::Expr {
    let field = lower_expr(cx, field);
    let field_ty = field.ty();
    match layout {
        VariantLayout::Single { field: _ } => field,
        VariantLayout::Tagged(TaggedLayout { tag: tag_lyt, variants: variants_lyt }) => {
            match tag_lyt {
                TagLayout::Direct { values: tag_vals, niches: _ } => {
                    let tag_expr = lir::Expr::U64(tag_vals[&variant]);
                    let union_ty = lir::Ty::UntaggedUnion(
                        variants_lyt.into_iter().map(|(n, l)| (n, lower_layout(l))).collect(),
                    );
                    let union_expr = lir::Expr::UntaggedUnion {
                        ty: union_ty,
                        field: variant.clone(),
                        value: Box::new(field),
                    };
                    lir::Expr::Record(map! { "tag" => tag_expr, "data" => union_expr })
                }
                TagLayout::Niche { path, values } => {
                    if field_ty.is_zst() {
                        construct_niche_nullary_variant(path, values[&variant])
                    } else {
                        field
                    }
                }
            }
        }
    }
}

/// This is like a "reverse projection".
fn construct_niche_nullary_variant(path: TagPath, tag_value: u64) -> lir::Expr {
    path.rfold(lir::Expr::U64(tag_value), |prev_expr, proj| {
        match proj {
            // FIXME: what about the other fields of the record's type?
            ValueProj::Field(name) => lir::Expr::Record(map! { name => prev_expr }),
            ValueProj::Variant { repr, name: _ } => match repr {
                // FIXME: is this correct? is it even reachable?
                VariantRepr::Wrapper => lir::Expr::Record(map! { "data" => prev_expr }),
                VariantRepr::Transparent => prev_expr,
            },
            ValueProj::Tag => lir::Expr::Record(map! { "tag" => prev_expr }),
        }
    })
}

fn select_value_at_path(root_value: lir::Expr, path: TagPath) -> lir::Expr {
    path.rfold(root_value, |prev_expr, proj| match proj {
        ValueProj::Field(field) => lir::Expr::Select { record: Box::new(prev_expr), field },
        ValueProj::Variant { repr, name: _ } => match repr {
            // FIXME: is this correct? is it even reachable?
            VariantRepr::Wrapper => {
                lir::Expr::Select { record: Box::new(prev_expr), field: "data".into() }
            }
            VariantRepr::Transparent => prev_expr,
        },
        ValueProj::Tag => lir::Expr::Select { record: Box::new(prev_expr), field: "tag".into() },
    })
}

fn lower_match(
    cx: &mut Ctxt,
    hir_subj: hir::Expr,
    hir_cases: Vec<(hir::Pat, hir::Expr)>,
) -> lir::Expr {
    let subj_hir_ty = hir_subj.ty();
    let subj_layout = layout_of(subj_hir_ty);

    let lir_subj_expr = lower_expr(cx, hir_subj);
    let lir_subj = cx.temp_var(lir_subj_expr.ty());

    let match_lir = match subj_layout {
        Layout::U64(..) => todo!(),
        Layout::Aggregate { fields: _ } => todo!(),
        Layout::Ptr { .. } => panic!(),
        Layout::Recursive(_) | Layout::RecurId(_) => panic!(),
        Layout::Variant(layout) => lower_variant_match(cx, layout, lir_subj.clone(), hir_cases),
    };

    lir::Expr::Let { binder: lir_subj, value: Box::new(lir_subj_expr), body: Box::new(match_lir) }
}

fn lower_variant_match(
    cx: &mut Ctxt,
    layout: VariantLayout,
    lir_subj: lir::Var,
    hir_cases: Vec<(hir::Pat, hir::Expr)>,
) -> lir::Expr {
    match layout {
        VariantLayout::Single { field: _ } => lower_single_variant_match(cx, lir_subj, hir_cases),
        VariantLayout::Tagged(lyt) => lower_tagged_variant_match(cx, lyt, lir_subj, hir_cases),
    }
}

fn lower_single_variant_match(
    cx: &mut Ctxt,
    lir_subj: lir::Var,
    hir_cases: Vec<(hir::Pat, hir::Expr)>,
) -> lir::Expr {
    let (pat, body) = expect_singleton_vec(hir_cases);
    match pat {
        hir::Pat::Variant { ty: _, variant: _, field } => {
            lower_match_arm_body(cx, (field, lir::Expr::Var(lir_subj)), body)
        }
    }
}

fn lower_tagged_variant_match(
    cx: &mut Ctxt,
    lyt: TaggedLayout,
    lir_subj: lir::Var,
    hir_cases: Vec<(hir::Pat, hir::Expr)>,
) -> lir::Expr {
    let mut default = None;
    let cases: BTreeMap<_, _> = hir_cases
        .into_iter()
        .map(|(p, e)| lower_tagged_variant_match_arm(cx, &lyt, lir_subj.clone(), p, e))
        .filter_map(|(v, e)| match v {
            Some(v) => Some((v, e)),
            None => {
                assert!(default.is_none());
                default = Some(e);
                None
            }
        })
        .collect();

    let switch_subj_expr = match lyt.tag {
        TagLayout::Direct { .. } => lir::Expr::Select {
            record: Box::new(lir::Expr::Var(lir_subj.clone())),
            field: "tag".into(),
        },
        TagLayout::Niche { path, values: _ } => {
            select_value_at_path(lir::Expr::Var(lir_subj), path)
        }
    };
    let switch_subj = cx.temp_var(switch_subj_expr.ty());

    let default = default.map(Box::new);
    let switch_expr = Box::new(lir::Expr::Switch { subj: switch_subj.clone(), cases, default });
    lir::Expr::Let {
        binder: switch_subj.clone(),
        value: Box::new(switch_subj_expr),
        body: switch_expr,
    }
}

fn lower_tagged_variant_match_arm(
    cx: &mut Ctxt,
    lyt: &TaggedLayout,
    lir_subj: lir::Var,
    pat: hir::Pat,
    body: hir::Expr,
) -> (Option<u64>, lir::Expr) {
    match &lyt.tag {
        TagLayout::Direct { values: tag_vals, niches: _ } => match pat {
            hir::Pat::Variant { ty: _, variant, field } => {
                let select_field = lir::Expr::Select {
                    record: Box::new(lir::Expr::Var(lir_subj)),
                    field: Name::from("data"),
                };
                let body = lower_match_arm_body(cx, (field, select_field), body);
                (Some(tag_vals[&variant]), body)
            }
        },
        TagLayout::Niche { path: _, values: tag_vals } => match pat {
            hir::Pat::Variant { ty: _, variant, field } => {
                let body = lower_match_arm_body(cx, (field, lir::Expr::Var(lir_subj)), body);
                (tag_vals.get(&variant).copied(), body)
            }
        },
    }
}

fn lower_match_arm_body(
    cx: &mut Ctxt,
    binding: (hir::Var, lir::Expr),
    body: hir::Expr,
) -> lir::Expr {
    let (binder, value) = binding;
    let binder = lower_var(binder);
    let body = lower_expr(cx, body);
    lir::Expr::Let { binder, value: Box::new(value), body: Box::new(body) }
}

fn lower_var(var: hir::Var) -> lir::Var {
    let hir::Var { name, ty } = var;
    lir::Var::new(name, lower_layout(layout_of(ty)))
}

// FIXME: this is only pub(crate) because it's used in a crate-level test
pub(crate) fn lower_layout(layout: Layout) -> lir::Ty {
    match layout {
        Layout::U64(_) => lir::Ty::U64,
        Layout::Ptr { pointee, niches: _ } => lir::Ty::Ptr(Box::new(lower_layout(*pointee))),
        Layout::Aggregate { fields } => {
            lir::Ty::Record(fields.into_iter().map(|(n, l)| (n, lower_layout(l))).collect())
        }
        Layout::Variant(VariantLayout::Single { field }) => lower_layout(*field),
        Layout::Variant(VariantLayout::Tagged(TaggedLayout { tag, variants })) => {
            match tag {
                TagLayout::Direct { values: _, niches: _ } => {
                    let tag_ty = lir::Ty::U64;
                    let variant_tys =
                        variants.into_iter().map(|(n, l)| (n, lower_layout(l))).collect();
                    let data_ty = lir::Ty::UntaggedUnion(variant_tys);
                    // FIXME: these should be represented differently from user-written fields.
                    // Otherwise, there could be name conflicts in some situations.
                    lir::Ty::Record(map! { "tag" => tag_ty, "data" => data_ty })
                }
                TagLayout::Niche { path: _, values: _ } => {
                    let field_lyt = is_nicheable(&variants).as_field().unwrap().clone();
                    let field_ty = lower_layout(field_lyt);
                    field_ty
                }
            }
        }
        Layout::Recursive(body) => lir::Ty::Recursive(Box::new(lower_layout(*body))),
        Layout::RecurId(k) => lir::Ty::RecurId(k),
    }
}

#[cfg(test)]
mod tests;
