use insta::assert_display_snapshot;

use crate::debruijn::Debruijn;

use super::*;

// HELPERS

fn unit_ty() -> hir::Ty {
    parse_ty("{}")
}

fn pair_of(field0: hir::Ty, field1: hir::Ty) -> hir::Ty {
    hir::Ty::Record(map! { 0 => field0, 1 => field1 })
}

fn empty_ty() -> hir::Ty {
    parse_ty("<>")
}

fn bool_ty() -> hir::Ty {
    parse_ty("<False of {} | True of {}>")
}

fn maybe_of(ty: hir::Ty) -> hir::Ty {
    hir::Ty::Variant(map! { "None" => hir::Ty::Record(map!{}), "Some" => ty })
}

fn maybe_empty_ty() -> hir::Ty {
    maybe_of(empty_ty())
}

fn maybe_bool_ty() -> hir::Ty {
    maybe_of(bool_ty())
}

fn either_of(left: hir::Ty, right: hir::Ty) -> hir::Ty {
    hir::Ty::Variant(map! { "Left" => left, "Right" => right })
}

fn list_of(elem: hir::Ty) -> hir::Ty {
    hir::Ty::Recursive(Box::new(hir::Ty::Variant(map! {
        "Nil" => hir::Ty::Record(map! {}),
        "Cons" => hir::Ty::Record(map! {
            "hd" => elem,
            "tl" => hir::Ty::Box(Box::new(hir::Ty::Named(Debruijn::ZERO))),
        }),
    })))
}

// TESTS

#[test]
fn unit_ty_layout() {
    assert_display_snapshot!(layout_of(unit_ty()), @"Aggregate {}");
}

#[test]
fn unit_ty_lir() {
    assert_display_snapshot!(lower_root_expr(parse("{}")), @"{}");
}

#[test]
fn empty_variant_layout() {
    assert_display_snapshot!(layout_of(empty_ty()), @"Variant(Single(field: Aggregate {}))");
}

#[test]
fn empty_variant_lty() {
    assert_display_snapshot!(crate::lower::lower_layout(layout_of(empty_ty())), @"{}");
}

#[test]
fn bool_layout() {
    assert_display_snapshot!(layout_of(bool_ty()), @r###"
    Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 2..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    ))
    "###);
}

#[test]
fn maybe_empty_layout() {
    assert_display_snapshot!(layout_of(maybe_empty_ty()), @r###"
    Variant(Tagged(tag: Direct(values: { None => 0, Some => 1 }, niches: 2..=18446744073709551615), variants:
    | None => Aggregate {}
    | Some => Variant(Single(field: Aggregate {}))
    ))
    "###);
}

#[test]
fn maybe_empty_lty() {
    assert_display_snapshot!(crate::lower::lower_layout(layout_of(maybe_empty_ty())), @"{ data : union { None : {} | Some : {} }, tag : U64 }");
}

#[test]
fn maybe_bool_layout() {
    assert_display_snapshot!(layout_of(maybe_bool_ty()), @r###"
    Variant(Tagged(tag: Niche(path: ({root} as(transparent) Some).{tag}, values: { None => 2 }), variants:
    | None => Aggregate {}
    | Some => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 3..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    ))
    ))
    "###);
}

#[test]
fn either_unit_unit_layout() {
    assert_display_snapshot!(layout_of(either_of(unit_ty(), unit_ty())), @r###"
    Variant(Tagged(tag: Direct(values: { Left => 0, Right => 1 }, niches: 2..=18446744073709551615), variants:
    | Left => Aggregate {}
    | Right => Aggregate {}
    ))
    "###);
}

#[test]
fn either_unit_bool_layout() {
    assert_display_snapshot!(layout_of(either_of(unit_ty(), bool_ty())), @r###"
    Variant(Tagged(tag: Niche(path: ({root} as(transparent) Right).{tag}, values: { Left => 2 }), variants:
    | Left => Aggregate {}
    | Right => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 3..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    ))
    ))
    "###);
}

#[test]
fn either_bool_unit_layout() {
    assert_display_snapshot!(layout_of(either_of(bool_ty(), unit_ty())), @r###"
    Variant(Tagged(tag: Niche(path: ({root} as(transparent) Left).{tag}, values: { Right => 2 }), variants:
    | Left => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 3..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    ))
    | Right => Aggregate {}
    ))
    "###);
}

#[test]
fn either_bool_bool_layout() {
    assert_display_snapshot!(layout_of(either_of(bool_ty(), bool_ty())), @r###"
    Variant(Tagged(tag: Direct(values: { Left => 0, Right => 1 }, niches: 2..=18446744073709551615), variants:
    | Left => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 2..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    ))
    | Right => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 2..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    ))
    ))
    "###);
}

#[test]
fn either_unit_maybe_bool_layout() {
    assert_display_snapshot!(layout_of(either_of(unit_ty(), maybe_of(bool_ty()))), @r###"
    Variant(Tagged(tag: Niche(path: (({root} as(transparent) Right) as(transparent) Some).{tag}, values: { Left => 3 }), variants:
    | Left => Aggregate {}
    | Right => Variant(Tagged(tag: Niche(path: ({root} as(transparent) Some).{tag}, values: { None => 2 }), variants:
    | None => Aggregate {}
    | Some => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 4..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    ))
    ))
    ))
    "###);
}

// TODO: add tests for values of type Either<(), Maybe<Bool>>

#[test]
fn maybe_of_pair_of_unit_and_unit_layout() {
    assert_display_snapshot!(layout_of(maybe_of(pair_of(unit_ty(), unit_ty()))), @r###"
    Variant(Tagged(tag: Direct(values: { None => 0, Some => 1 }, niches: 2..=18446744073709551615), variants:
    | None => Aggregate {}
    | Some => Aggregate { 0 => Aggregate {}, 1 => Aggregate {} }
    ))
    "###);
}

#[test]
fn maybe_of_pair_of_unit_and_bool_layout() {
    assert_display_snapshot!(layout_of(maybe_of(pair_of(unit_ty(), bool_ty()))), @r###"
    Variant(Tagged(tag: Niche(path: ({root} as(transparent) Some).1.{tag}, values: { None => 2 }), variants:
    | None => Aggregate {}
    | Some => Aggregate { 0 => Aggregate {}, 1 => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 3..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    )) }
    ))
    "###);
}

#[test]
fn maybe_of_pair_of_bool_and_unit_layout() {
    assert_display_snapshot!(layout_of(maybe_of(pair_of(bool_ty(), unit_ty()))), @r###"
    Variant(Tagged(tag: Niche(path: ({root} as(transparent) Some).0.{tag}, values: { None => 2 }), variants:
    | None => Aggregate {}
    | Some => Aggregate { 0 => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 3..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    )), 1 => Aggregate {} }
    ))
    "###);
}

#[test]
fn maybe_of_pair_of_bool_and_bool_layout() {
    assert_display_snapshot!(layout_of(maybe_of(pair_of(bool_ty(), bool_ty()))), @r###"
    Variant(Tagged(tag: Niche(path: ({root} as(transparent) Some).0.{tag}, values: { None => 2 }), variants:
    | None => Aggregate {}
    | Some => Aggregate { 0 => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 3..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    )), 1 => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 2..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    )) }
    ))
    "###);
}

#[test]
fn list_of_unit_layout() {
    assert_display_snapshot!(layout_of(list_of(unit_ty())), @r###"
    Recursive(Variant(Tagged(tag: Niche(path: ({root} as(transparent) Cons).tl, values: { Nil => 0 }), variants:
    | Cons => Aggregate { hd => Aggregate {}, tl => Ptr(pointee: recur#0, niches: none) }
    | Nil => Aggregate {}
    )))
    "###);
}

#[test]
fn list_of_bool_layout() {
    assert_display_snapshot!(layout_of(list_of(bool_ty())), @r###"
    Recursive(Variant(Tagged(tag: Niche(path: ({root} as(transparent) Cons).hd.{tag}, values: { Nil => 2 }), variants:
    | Cons => Aggregate { hd => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 3..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    )), tl => Ptr(pointee: recur#0, niches: 0..=0) }
    | Nil => Aggregate {}
    )))
    "###);
}

#[test]
fn list_of_maybe_unit_layout() {
    assert_display_snapshot!(layout_of(list_of(maybe_of(unit_ty()))), @r###"
    Recursive(Variant(Tagged(tag: Niche(path: ({root} as(transparent) Cons).hd.{tag}, values: { Nil => 2 }), variants:
    | Cons => Aggregate { hd => Variant(Tagged(tag: Direct(values: { None => 0, Some => 1 }, niches: 3..=18446744073709551615), variants:
    | None => Aggregate {}
    | Some => Aggregate {}
    )), tl => Ptr(pointee: recur#0, niches: 0..=0) }
    | Nil => Aggregate {}
    )))
    "###);
}

#[test]
fn list_of_maybe_bool_layout() {
    assert_display_snapshot!(layout_of(list_of(maybe_of(bool_ty()))), @r###"
    Recursive(Variant(Tagged(tag: Niche(path: (({root} as(transparent) Cons).hd as(transparent) Some).{tag}, values: { Nil => 3 }), variants:
    | Cons => Aggregate { hd => Variant(Tagged(tag: Niche(path: ({root} as(transparent) Some).{tag}, values: { None => 2 }), variants:
    | None => Aggregate {}
    | Some => Variant(Tagged(tag: Direct(values: { False => 0, True => 1 }, niches: 4..=18446744073709551615), variants:
    | False => Aggregate {}
    | True => Aggregate {}
    ))
    )), tl => Ptr(pointee: recur#0, niches: 0..=0) }
    | Nil => Aggregate {}
    )))
    "###);
}
