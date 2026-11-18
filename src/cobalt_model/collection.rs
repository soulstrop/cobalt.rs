use cobalt_config::Frontmatter;
use cobalt_config::SortOrder;
use liquid;

use crate::error::Result;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Collection {
    pub title: liquid::model::KString,
    pub slug: liquid::model::KString,
    pub description: Option<liquid::model::KString>,
    pub dir: cobalt_config::RelPath,
    pub drafts_dir: Option<cobalt_config::RelPath>,
    pub order: SortOrder,
    pub rss: Option<cobalt_config::RelPath>,
    pub jsonfeed: Option<cobalt_config::RelPath>,
    pub standard_site: bool,
    pub publish_date_in_filename: bool,
    pub default: Frontmatter,
}

impl Collection {
    pub fn from_page_config(
        config: cobalt_config::PageCollection,
        site: &cobalt_config::Site,
        common_default: &Frontmatter,
    ) -> Result<Self> {
        let mut config: cobalt_config::Collection = config.into();
        // Use `site` because the pages are effectively the site
        config.title = Some(site.title.clone().unwrap_or_else(|| "".into()));
        config.description = site.description.clone();
        Self::from_config(config, "pages", false, common_default)
    }

    pub fn from_post_config(
        config: cobalt_config::PostCollection,
        site: &cobalt_config::Site,
        include_drafts: bool,
        common_default: &Frontmatter,
    ) -> Result<Self> {
        let mut config: cobalt_config::Collection = config.into();
        // Default with `site` for people quickly bootstrapping a blog, the blog and site are
        // effectively equivalent.
        if config.title.is_none() {
            config.title = Some(site.title.clone().unwrap_or_else(|| "".into()));
        }
        if config.description.is_none() {
            config.description = site.description.clone();
        }
        Self::from_config(config, "posts", include_drafts, common_default)
    }

    fn from_config(
        config: cobalt_config::Collection,
        slug: &str,
        include_drafts: bool,
        common_default: &Frontmatter,
    ) -> Result<Self> {
        let cobalt_config::Collection {
            title,
            description,
            dir,
            drafts_dir,
            order,
            rss,
            jsonfeed,
            standard_site,
            default,
            publish_date_in_filename,
        } = config;

        let title = title.ok_or_else(|| anyhow::format_err!("Collection is missing a `title`"))?;
        let slug = liquid::model::KString::from_ref(slug);

        let dir = dir.unwrap_or_else(|| cobalt_config::RelPath::from_unchecked(slug.as_str()));
        let drafts_dir = if include_drafts { drafts_dir } else { None };

        let default = default.merge(common_default).merge(&Frontmatter {
            collection: Some(slug.clone()),
            ..Default::default()
        });

        let new = Collection {
            title,
            slug,
            description,
            dir,
            drafts_dir,
            order,
            rss,
            jsonfeed,
            standard_site,
            publish_date_in_filename,
            default,
        };
        Ok(new)
    }

    pub fn attributes(&self) -> liquid::Object {
        let mut attributes: liquid::Object = vec![
            (
                "title".into(),
                liquid::model::Value::scalar(self.title.clone()),
            ),
            (
                "slug".into(),
                liquid::model::Value::scalar(self.slug.clone()),
            ),
            (
                "description".into(),
                liquid::model::Value::scalar(self.description.clone().unwrap_or_default()),
            ),
        ]
        .into_iter()
        .collect();
        if let Some(rss) = self.rss.as_ref() {
            attributes.insert(
                "rss".into(),
                liquid::model::Value::scalar(rss.as_str().to_owned()),
            );
        }
        if let Some(jsonfeed) = self.jsonfeed.as_ref() {
            attributes.insert(
                "jsonfeed".into(),
                liquid::model::Value::scalar(jsonfeed.as_str().to_owned()),
            );
        }
        attributes.insert(
            "standard_site".into(),
            liquid::model::Value::scalar(self.standard_site),
        );
        attributes
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn common_default() -> Frontmatter {
        Frontmatter {
            slug: Some("common-slug".into()),
            title: Some("Common Title".into()),
            description: Some("Common Description".into()),
            ..Default::default()
        }
    }

    #[test]
    fn from_page_config_uses_site_metadata() {
        let site = cobalt_config::Site {
            title: Some("Site Title".into()),
            description: Some("Site Description".into()),
            ..Default::default()
        };

        let actual = Collection::from_page_config(
            cobalt_config::PageCollection::default(),
            &site,
            &common_default(),
        )
        .unwrap();

        assert_eq!(actual.title, "Site Title");
        assert_eq!(actual.slug, "pages");
        assert_eq!(actual.description, Some("Site Description".into()));
        assert_eq!(actual.dir.as_str(), "");
        assert_eq!(actual.order, SortOrder::None);
        assert_eq!(actual.default.collection, Some("pages".into()));
        assert_eq!(actual.default.excerpt_separator, Some("".into()));
    }

    #[test]
    fn from_post_config_uses_site_defaults_and_ignores_drafts() {
        let site = cobalt_config::Site {
            title: Some("Blog".into()),
            description: Some("Blog Description".into()),
            ..Default::default()
        };
        let config = cobalt_config::PostCollection {
            drafts_dir: Some(cobalt_config::RelPath::from_unchecked("drafts")),
            ..Default::default()
        };

        let actual = Collection::from_post_config(config, &site, false, &common_default()).unwrap();

        assert_eq!(actual.title, "Blog");
        assert_eq!(actual.description, Some("Blog Description".into()));
        assert_eq!(actual.slug, "posts");
        assert_eq!(actual.dir.as_str(), "posts");
        assert_eq!(actual.drafts_dir, None);
        assert_eq!(actual.default.collection, Some("posts".into()));
    }

    #[test]
    fn from_post_config_preserves_drafts_when_enabled() {
        let config = cobalt_config::PostCollection {
            title: Some("Posts".into()),
            drafts_dir: Some(cobalt_config::RelPath::from_unchecked("drafts")),
            ..Default::default()
        };

        let actual = Collection::from_post_config(
            config,
            &cobalt_config::Site::default(),
            true,
            &common_default(),
        )
        .unwrap();

        assert_eq!(
            actual
                .drafts_dir
                .as_ref()
                .map(cobalt_config::RelPath::as_str),
            Some("drafts")
        );
    }

    #[test]
    fn attributes_include_optional_feeds() {
        let config = cobalt_config::PostCollection {
            title: Some("Posts".into()),
            description: Some("Latest posts".into()),
            rss: Some(cobalt_config::RelPath::from_unchecked("feed.xml")),
            jsonfeed: Some(cobalt_config::RelPath::from_unchecked("feed.json")),
            ..Default::default()
        };
        let collection = Collection::from_post_config(
            config,
            &cobalt_config::Site::default(),
            false,
            &common_default(),
        )
        .unwrap();

        let actual = serde_json::to_value(collection.attributes()).unwrap();

        assert_eq!(actual.get("description"), Some(&json!("Latest posts")));
        assert_eq!(actual.get("jsonfeed"), Some(&json!("feed.json")));
        assert_eq!(actual.get("rss"), Some(&json!("feed.xml")));
        assert_eq!(actual.get("slug"), Some(&json!("posts")));
        assert_eq!(actual.get("title"), Some(&json!("Posts")));
    }
}
