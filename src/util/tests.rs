use super::*;

#[test]
fn test_range_values_count() {
    fn t(r: RangeInclusive<u64>, expect: Option<u64>) {
        assert_eq!(range_values_count(r), expect);
    }

    t(1..=0, Some(0));
    t(2..=0, Some(0));

    t(0..=0, Some(1));
    t(1..=1, Some(1));
    t(0..=2, Some(3));
    t(0..=(u64::MAX - 1), Some(u64::MAX));

    t(0..=u64::MAX, None);
}
