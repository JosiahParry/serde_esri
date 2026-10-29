use parquet::{
    arrow::arrow_reader::{RowSelection, RowSelector},
    file::{
        metadata::{ParquetMetaData, RowGroupMetaData},
        page_index::column_index::{ColumnIndexMetaData, PrimitiveColumnIndex},
        statistics::{Statistics, ValueStatistics},
    },
};

pub(crate) struct Plan {
    pub(crate) row_groups: Vec<usize>,
    pub(crate) selection: RowSelection,
}

impl Plan {
    pub(crate) fn new(
        metadata: &ParquetMetaData,
        rows: impl Fn(&RowGroupMetaData, usize) -> RowSelection,
    ) -> Self {
        let mut row_groups = Vec::new();
        let mut selectors = Vec::new();
        for (row_group_index, row_group) in metadata.row_groups().iter().enumerate() {
            let selection = rows(row_group, row_group_index);
            if selection.selects_any() {
                row_groups.push(row_group_index);
                selectors.extend(selection.iter().copied());
            }
        }
        Plan {
            row_groups,
            selection: RowSelection::from(selectors),
        }
    }
}

pub(crate) trait MinMax: Copy {
    fn statistics(statistics: &Statistics) -> Option<&ValueStatistics<Self>>;
    fn column_index(index: &ColumnIndexMetaData) -> Option<&PrimitiveColumnIndex<Self>>;

    fn rows(
        metadata: &ParquetMetaData,
        row_group: &RowGroupMetaData,
        row_group_index: usize,
        leaf: usize,
        keep: impl Fn([Self; 2]) -> bool,
    ) -> RowSelection {
        let total = usize::try_from(row_group.num_rows()).unwrap_or(0);
        let chunk = row_group
            .column(leaf)
            .statistics()
            .and_then(Self::statistics)
            .and_then(|s| Some([*s.min_opt()?, *s.max_opt()?]));
        if chunk.is_some_and(|bounds| !keep(bounds)) {
            return RowSelection::from(vec![RowSelector::skip(total)]);
        }
        let pages = metadata
            .column_index()
            .and_then(|c| c.get(row_group_index))
            .and_then(|c| c.get(leaf))
            .and_then(Self::column_index);
        let locations = metadata
            .offset_index()
            .and_then(|o| o.get(row_group_index))
            .and_then(|o| o.get(leaf));
        let (Some(pages), Some(locations)) = (pages, locations) else {
            return RowSelection::from(vec![RowSelector::select(total)]);
        };
        let starts = locations
            .page_locations()
            .iter()
            .map(|page| usize::try_from(page.first_row_index).unwrap_or(0))
            .collect::<Vec<_>>();
        let bounds = pages.min_values_iter().zip(pages.max_values_iter());
        let ranges = starts
            .iter()
            .zip(bounds)
            .enumerate()
            .filter(|(_, (_, (min, max)))| {
                min.zip(*max).is_some_and(|(&min, &max)| keep([min, max]))
            })
            .map(|(page, (&start, _))| start..starts.get(page + 1).copied().unwrap_or(total));
        RowSelection::from_consecutive_ranges(ranges, total)
    }
}

impl MinMax for i64 {
    fn statistics(statistics: &Statistics) -> Option<&ValueStatistics<Self>> {
        match statistics {
            Statistics::Int64(s) => Some(s),
            _ => None,
        }
    }

    fn column_index(index: &ColumnIndexMetaData) -> Option<&PrimitiveColumnIndex<Self>> {
        match index {
            ColumnIndexMetaData::INT64(c) => Some(c),
            _ => None,
        }
    }
}

impl MinMax for f64 {
    fn statistics(statistics: &Statistics) -> Option<&ValueStatistics<Self>> {
        match statistics {
            Statistics::Double(s) => Some(s),
            _ => None,
        }
    }

    fn column_index(index: &ColumnIndexMetaData) -> Option<&PrimitiveColumnIndex<Self>> {
        match index {
            ColumnIndexMetaData::DOUBLE(c) => Some(c),
            _ => None,
        }
    }
}
