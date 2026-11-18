use std::vec::Vec;

use cobalt_config::SortOrder;

pub use cobalt_config::DateIndex;
pub use cobalt_config::Include;

#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct PaginationConfig {
    pub include: Include,
    pub per_page: i32,
    pub front_permalink: cobalt_config::Permalink,
    pub permalink_suffix: liquid::model::KString,
    pub order: SortOrder,
    pub sort_by: Vec<liquid::model::KString>,
    pub date_index: Vec<DateIndex>,
}

impl PaginationConfig {
    pub fn from_config(
        config: cobalt_config::Pagination,
        permalink: &cobalt_config::Permalink,
    ) -> Option<Self> {
        let config = config.merge(&cobalt_config::Pagination::with_defaults());
        let cobalt_config::Pagination {
            include,
            per_page,
            permalink_suffix,
            order,
            sort_by,
            date_index,
        } = config;
        let include = include.expect("default applied");
        let per_page = per_page.expect("default applied");
        let permalink_suffix = permalink_suffix.expect("default applied");
        let order = order.expect("default applied");
        let sort_by = sort_by.expect("default applied");
        let date_index = date_index.expect("default applied");

        if include == Include::None {
            return None;
        }
        Some(Self {
            include,
            per_page,
            front_permalink: permalink.to_owned(),
            permalink_suffix,
            order,
            sort_by,
            date_index,
        })
    }
}

// TODO to be replaced by a call to `is_sorted()` once it's stabilized
pub fn is_date_index_sorted(v: &[DateIndex]) -> bool {
    let mut copy = v.to_owned();
    copy.sort_unstable();
    copy.eq(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_config_returns_none_when_disabled() {
        let actual = PaginationConfig::from_config(
            cobalt_config::Pagination::default(),
            &cobalt_config::Permalink::default(),
        );

        assert_eq!(actual, None);
    }

    #[test]
    fn from_config_applies_defaults() {
        let actual = PaginationConfig::from_config(
            cobalt_config::Pagination {
                include: Some(Include::Tags),
                ..Default::default()
            },
            &cobalt_config::Permalink::default(),
        )
        .unwrap();

        assert_eq!(actual.include, Include::Tags);
        assert_eq!(actual.per_page, 10);
        assert_eq!(actual.permalink_suffix, "{{num}}/");
        assert_eq!(actual.order, SortOrder::Desc);
        assert_eq!(actual.sort_by, vec!["published_date"]);
        assert_eq!(actual.date_index, vec![DateIndex::Year, DateIndex::Month]);
    }

    #[test]
    fn is_date_index_sorted_detects_unsorted_values() {
        assert!(is_date_index_sorted(&[]));
        assert!(is_date_index_sorted(&[
            DateIndex::Year,
            DateIndex::Month,
            DateIndex::Day
        ]));
        assert!(!is_date_index_sorted(&[DateIndex::Month, DateIndex::Year]));
    }
}
