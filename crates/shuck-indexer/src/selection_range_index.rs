use shuck_ast::{TextRange, TextSize};

use crate::{CommentIndex, RegionIndex};

/// Precomputed syntax spans used to expand editor selections.
///
/// The index stores parser-owned ranges in source order with equal spans removed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectionRangeIndex {
    ranges: Vec<TextRange>,
    file_range: TextRange,
}

impl SelectionRangeIndex {
    pub(crate) fn new(
        source: &str,
        comment_index: &CommentIndex,
        region_index: &RegionIndex,
    ) -> Self {
        let file_range = TextRange::new(TextSize::new(0), TextSize::from_usize(source.len()));
        let mut ranges = region_index.selection_ranges().to_vec();
        ranges.extend(comment_index.comments().iter().map(|comment| comment.range));
        ranges.push(file_range);
        ranges.retain(|range| valid_range(source, *range));
        ranges.sort_unstable_by_key(|range| (range.start(), range.end()));
        ranges.dedup();

        Self { ranges, file_range }
    }


}

fn valid_range(source: &str, range: TextRange) -> bool {
    let start = usize::from(range.start());
    let end = usize::from(range.end());
    start <= end
        && end <= source.len()
        && source.is_char_boundary(start)
        && source.is_char_boundary(end)
}
