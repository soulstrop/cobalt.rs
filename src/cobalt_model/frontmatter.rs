use std::fmt;

use cobalt_config::DateTime;
use cobalt_config::SourceFormat;
use liquid;
use serde::Serialize;

use super::pagination;
use crate::error::Result;

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Frontmatter {
    pub permalink: cobalt_config::Permalink,
    pub slug: liquid::model::KString,
    pub title: liquid::model::KString,
    pub description: Option<liquid::model::KString>,
    pub excerpt: Option<liquid::model::KString>,
    pub categories: Vec<liquid::model::KString>,
    pub tags: Vec<liquid::model::KString>,
    pub excerpt_separator: liquid::model::KString,
    pub published_date: Option<DateTime>,
    pub format: SourceFormat,
    pub templated: bool,
    pub layout: Option<liquid::model::KString>,
    pub is_draft: bool,
    pub at_uri: Option<liquid::model::KString>,
    pub weight: i32,
    pub collection: liquid::model::KString,
    pub data: liquid::Object,
    pub pagination: Option<pagination::PaginationConfig>,
}

impl Frontmatter {
    pub fn from_config(config: cobalt_config::Frontmatter) -> Result<Frontmatter> {
        let cobalt_config::Frontmatter {
            permalink,
            slug,
            title,
            description,
            excerpt,
            categories,
            tags,
            excerpt_separator,
            published_date,
            format,
            templated,
            layout,
            is_draft,
            at_uri,
            weight,
            collection,
            data,
            pagination,
        } = config;

        let collection = collection.unwrap_or_default();

        let permalink = permalink.unwrap_or_default();

        if let Some(tags) = &tags {
            if tags.iter().any(|x| x.trim().is_empty()) {
                anyhow::bail!("Empty strings are not allowed in tags");
            }
        }
        let fm = Frontmatter {
            pagination: pagination
                .and_then(|p| pagination::PaginationConfig::from_config(p, &permalink)),
            permalink,
            slug: slug.ok_or_else(|| anyhow::format_err!("No slug"))?,
            title: title.ok_or_else(|| anyhow::format_err!("No title"))?,
            description,
            excerpt,
            categories: categories.unwrap_or_default(),
            tags: tags.unwrap_or_default(),
            excerpt_separator: excerpt_separator.unwrap_or_else(|| "\n\n".into()),
            published_date,
            format: format.unwrap_or_default(),
            #[cfg(feature = "preview_unstable")]
            templated: templated.unwrap_or(false),
            #[cfg(not(feature = "preview_unstable"))]
            templated: templated.unwrap_or(true),
            layout,
            is_draft: is_draft.unwrap_or(false),
            at_uri,
            weight: weight.unwrap_or(0),
            collection,
            data,
        };

        if let Some(pagination) = &fm.pagination {
            if !pagination::is_date_index_sorted(&pagination.date_index) {
                anyhow::bail!("date_index is not correctly sorted: Year > Month > Day...");
            }
        }
        Ok(fm)
    }
}

impl fmt::Display for Frontmatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let converted = serde_yaml::to_string(self).expect("should always be valid");
        let subset = converted
            .strip_prefix("---")
            .unwrap_or(converted.as_str())
            .trim();
        let converted = if subset == "{}" { "" } else { subset };
        if converted.is_empty() {
            Ok(())
        } else {
            write!(f, "{converted}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> cobalt_config::Frontmatter {
        cobalt_config::Frontmatter {
            slug: Some("example-post".into()),
            title: Some("Example Post".into()),
            ..Default::default()
        }
    }

    #[test]
    fn from_config_applies_defaults_and_pagination() {
        let mut config = valid_config();
        config.pagination = Some(cobalt_config::Pagination {
            include: Some(cobalt_config::Include::Tags),
            ..Default::default()
        });

        let actual = Frontmatter::from_config(config).unwrap();

        assert_eq!(actual.excerpt_separator, "\n\n");
        assert_eq!(actual.weight, 0);
        assert!(!actual.is_draft);
        assert_eq!(
            actual.pagination.as_ref().unwrap().include,
            cobalt_config::Include::Tags
        );
        assert_eq!(actual.pagination.as_ref().unwrap().per_page, 10);
        assert_eq!(
            actual.pagination.as_ref().unwrap().date_index,
            vec![
                cobalt_config::DateIndex::Year,
                cobalt_config::DateIndex::Month
            ]
        );
    }

    #[test]
    fn from_config_rejects_blank_tags() {
        let mut config = valid_config();
        config.tags = Some(vec!["valid".into(), "   ".into()]);

        let err = Frontmatter::from_config(config).unwrap_err();

        assert_eq!(err.to_string(), "Empty strings are not allowed in tags");
    }

    #[test]
    fn from_config_requires_slug() {
        let mut config = valid_config();
        config.slug = None;

        let err = Frontmatter::from_config(config).unwrap_err();

        assert_eq!(err.to_string(), "No slug");
    }

    #[test]
    fn from_config_requires_title() {
        let mut config = valid_config();
        config.title = None;

        let err = Frontmatter::from_config(config).unwrap_err();

        assert_eq!(err.to_string(), "No title");
    }

    #[test]
    fn from_config_rejects_unsorted_date_index() {
        let mut config = valid_config();
        config.pagination = Some(cobalt_config::Pagination {
            include: Some(cobalt_config::Include::Dates),
            date_index: Some(vec![
                cobalt_config::DateIndex::Month,
                cobalt_config::DateIndex::Year,
            ]),
            ..Default::default()
        });

        let err = Frontmatter::from_config(config).unwrap_err();

        assert_eq!(
            err.to_string(),
            "date_index is not correctly sorted: Year > Month > Day..."
        );
    }
}
