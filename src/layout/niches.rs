use std::ops::RangeInclusive;

use crate::util::range_values_count;

#[derive(Debug, Clone, Default)]
pub struct IntNiches {
    // TODO: use Vec to support multiple disjoint niche ranges
    range: Option<Range>,
}

type Range = RangeInclusive<u64>;

impl IntNiches {
    pub fn none() -> Self {
        Self { range: None }
    }

    pub fn range(range: Range) -> Self {
        Self { range: Some(range) }
    }

    fn from_option(range: Option<Range>) -> Self {
        Self { range }
    }

    pub(crate) fn as_range(&self) -> Option<Range> {
        let Self { range } = self;
        range.clone()
    }

    pub fn remove_value(self, value: u64) -> Result<Self, Self> {
        match self {
            Self { range: None } => Err(Self::none()),
            Self { range: Some(range) } => {
                remove_value_from_range(value, range).map(Self::from_option).map_err(Self::range)
            }
        }
    }

    /// Remove `count` niche values from `self`.
    ///
    /// If that many values were available, returns `Ok((new_self, extracted_values))`.
    /// If not enough values were available, returns `Err(old_self)`.
    pub fn remove_some_values(self, count: u64) -> Result<(Self, Self), Self> {
        match self {
            Self { range: None } => Err(Self::none()),
            Self { range: Some(range) } => shrink_range_by(range, count)
                .map(|OkRangeShrink { new_range, extracted }| {
                    (Self::from_option(new_range), Self::from_option(extracted))
                })
                .map_err(Self::range),
        }
    }

    pub fn remove_some_values_mut(&mut self, count: u64) -> Result<Self, ()> {
        match self.clone().remove_some_values(count) {
            Ok((slf, extracted)) => {
                *self = slf;
                Ok(extracted)
            }
            Err(slf) => {
                *self = slf;
                Err(())
            }
        }
    }
}

struct OkRangeShrink {
    new_range: Option<Range>,
    extracted: Option<Range>,
}

fn shrink_range_by(range: Range, count: u64) -> Result<OkRangeShrink, Range> {
    let start = *range.start();
    let end = *range.end();
    let available_count = range_values_count(range.clone()).unwrap_or(u64::MAX);
    if !range.is_empty() && available_count >= count {
        let new_range = normalize_range((start + count)..=end);
        let extracted = (start + count).checked_sub(1).and_then(|end| normalize_range(start..=end));
        Ok(OkRangeShrink { new_range, extracted })
    } else {
        Err(range)
    }
}

fn remove_value_from_range(value: u64, range: Range) -> Result<Option<Range>, Range> {
    if !range.contains(&value) {
        return Err(range);
    }

    let old_start = *range.start();
    let old_end = *range.end();

    let new_range = if value == old_end {
        match value.checked_sub(1) {
            Some(new_end) => old_start..=new_end,
            // We're removing 0 from a range that ended at 0,
            // so return an empty range.
            None => return Ok(None),
        }
    } else {
        // Arbitrarily keep the upper side of the range.
        let new_start = value.checked_add(1).unwrap();
        new_start..=old_end
    };

    Ok(normalize_range(new_range))
}

fn normalize_range(range: Range) -> Option<Range> {
    if range.is_empty() {
        None
    } else {
        Some(range)
    }
}

impl std::fmt::Display for IntNiches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self { range: None } => {
                write!(f, "none")
            }
            Self { range: Some(range) } => write!(f, "{:?}", range),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shrink_range_by() {
        fn shrink(range: Range, count: u64) -> Result<(Option<Range>, Option<Range>), Range> {
            let OkRangeShrink { new_range, extracted } = shrink_range_by(range, count)?;
            Ok((new_range, extracted))
        }

        assert_eq!(shrink(0..=0, 0), Ok((Some(0..=0), None)));
        assert_eq!(shrink(0..=0, 1), Ok((None, Some(0..=0))));
        assert_eq!(shrink(0..=0, 2), Err(0..=0));

        assert_eq!(shrink(0..=3, 0), Ok((Some(0..=3), None)));
        assert_eq!(shrink(0..=3, 2), Ok((Some(2..=3), Some(0..=1))));
        assert_eq!(shrink(0..=3, 3), Ok((Some(3..=3), Some(0..=2))));
        assert_eq!(shrink(0..=3, 4), Ok((None, Some(0..=3))));
        assert_eq!(shrink(0..=3, 5), Err(0..=3));
        assert_eq!(shrink(0..=3, 6), Err(0..=3));

        assert_eq!(shrink(0..=u64::MAX, 0), Ok((Some(0..=u64::MAX), None)));
        assert_eq!(shrink(0..=u64::MAX, 1), Ok((Some(1..=u64::MAX), Some(0..=0))));
        assert_eq!(
            shrink(0..=u64::MAX, u64::MAX - 2),
            Ok((Some((u64::MAX - 2)..=u64::MAX), Some(0..=(u64::MAX - 3))))
        );
        assert_eq!(
            shrink(0..=u64::MAX, u64::MAX - 1),
            Ok((Some((u64::MAX - 1)..=u64::MAX), Some(0..=(u64::MAX - 2))))
        );
        assert_eq!(
            shrink(0..=u64::MAX, u64::MAX),
            Ok((Some(u64::MAX..=u64::MAX), Some(0..=(u64::MAX - 1))))
        );
    }

    #[test]
    fn test_remove_value_from_range() {
        assert_eq!(remove_value_from_range(7, 2..=5), Err(2..=5));
        assert_eq!(remove_value_from_range(0, 0..=0), Ok(None));
        assert_eq!(remove_value_from_range(1, 0..=3), Ok(Some(2..=3)));
        assert_eq!(remove_value_from_range(2, 1..=3), Ok(Some(3..=3)));
        assert_eq!(remove_value_from_range(1, 1..=3), Ok(Some(2..=3)));
        assert_eq!(remove_value_from_range(3, 1..=3), Ok(Some(1..=2)));
    }
}
