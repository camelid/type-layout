use super::*;

#[test]
fn test_construct_niche_nullary_variant() {
    fn c(p: TagPath, t: u64) -> lir::Expr {
        construct_niche_nullary_variant(p, t)
    }

    assert_eq!(c(TagPath::empty(), 123), lir::Expr::U64(123));
    assert_eq!(
        c(TagPath::singleton(ValueProj::Tag), 123),
        lir::Expr::Record(map! { "tag" => lir::Expr::U64(123) })
    );
    assert_eq!(
        c(
            TagPath::singleton(ValueProj::Tag).with_outer_path(TagPath::singleton(
                ValueProj::Variant { repr: VariantRepr::Transparent, name: "Some".into() }
            )),
            123
        ),
        lir::Expr::Record(map! { "tag" => lir::Expr::U64(123) })
    );
    assert_eq!(
        c(
            TagPath::singleton(ValueProj::Tag).with_outer_path(TagPath::singleton(
                ValueProj::Variant { repr: VariantRepr::Wrapper, name: "Some".into() }
            )),
            123
        ),
        lir::Expr::Record(
            map! { "data" => lir::Expr::Record(map! { "tag" => lir::Expr::U64(123) }) }
        )
    );
}
