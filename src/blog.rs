use serde::{Deserialize, Serialize};

use crate::{
	resource::{ResourceBuilderConfig, ResourceMetadata, ResourceMethods},
	Site, SiteConfig,
};

pub const BLOG_PATH: &str = "blog";

/// Gets the blog's resource configuration.
pub fn get_blog_resource_config(site: &Site) -> ResourceBuilderConfig {
	ResourceBuilderConfig {
		source_path: BLOG_PATH.to_string(),
		output_path_short: BLOG_PATH.to_string(),
		output_path_long: BLOG_PATH.to_string(),
		resource_template: "blog-post".to_string(),
		resource_list_template: "blog-list".to_string(),
		rss_template: "rss/blog-post".to_string(),
		rss_title: "zyl's blog".to_string(),
		rss_description: "Feed of recent blog posts on zyl's website.".to_string(),
		list_title: "blog".to_string(),
		tag_list_title: "blog tags".to_string(),
		resource_name_plural: "blog posts".to_string(),
		resources_per_page: site.config.blog_posts_per_page,
	}
}

/// Metadata for a blog post.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPostMetadata {
	/// A short description about the post.
	pub desc: String,
	/// Path to the post's header image.
	pub header_image_file: String,
	/// Alt text for the post's header image.
	pub header_image_alt: String,
	/// Optional custom object fit value.
	pub image_fit: Option<String>,
	/// Optional custom object position value.
	pub image_center: Option<String>,
}

impl BlogPostMetadata {
	/// Helper to get the CDN URL to the blog post's header image.
	fn get_header_image(&self, site_config: &SiteConfig) -> anyhow::Result<String> {
		Ok(site_config.cdn_url(&self.header_image_file)?.to_string())
	}
}

/// Template data for a blog post.
#[derive(Debug, Serialize)]
pub struct BlogPostTemplateData {
	/// CDN path to the post's header image.
	pub header_image: String,
	/// Custom object fit value.
	pub object_fit: String,
	/// Custom object position value.
	pub object_position: String,
}

impl ResourceMethods<BlogPostTemplateData> for ResourceMetadata<BlogPostMetadata> {
	fn get_short_desc(&self) -> String {
		self.inner.desc.clone()
	}

	fn get_extra_resource_template_data(
		&self,
		site_config: &SiteConfig,
	) -> anyhow::Result<BlogPostTemplateData> {
		// TODO: render markdown
		Ok(BlogPostTemplateData {
			header_image: self.inner.get_header_image(site_config)?,
			object_fit: self
				.inner
				.image_fit
				.clone()
				.unwrap_or_else(|| "cover".to_string()),
			object_position: self
				.inner
				.image_center
				.clone()
				.unwrap_or_else(|| "50% 50%".to_string()),
		})
	}

	fn get_head_data(&self, site_config: &SiteConfig) -> anyhow::Result<String> {
		// TODO: update this so we're not just doing raw html injection lmao
		Ok(format!(
			r#"
		<meta property="og:site_name" content="{}">
		<meta name="twitter:card" content="summary_large_image">
		<meta name="twitter:title" content="{}">
		<meta name="twitter:image" content="{}">
		<meta name="og:description" content="{}">
		"#,
			site_config.title,
			self.title,
			self.inner.get_header_image(site_config)?,
			self.inner.desc,
		))
	}
}
