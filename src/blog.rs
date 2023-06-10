use serde::{Deserialize, Serialize};

use crate::{
	builder::SiteBuilder,
	resource::{ResourceBuilder, ResourceBuilderConfig, ResourceMetadata, ResourceMethods},
};

pub const BLOG_PATH: &str = "blog";

/// Builds the blog.
pub fn build_blog(site_builder: &SiteBuilder) -> anyhow::Result<()> {
	let config = ResourceBuilderConfig {
		source_path: BLOG_PATH.to_string(),
		output_path_short: BLOG_PATH.to_string(),
		output_path_long: BLOG_PATH.to_string(),
		resource_template: "blog-post".to_string(),
		resource_list_template: "blog-list".to_string(),
		rss_template: "rss/blog-post".to_string(),
		rss_title: "Zyllian's blog".to_string(),
		rss_description: "Feed of recent blog posts on Zyllian's website.".to_string(),
		list_title: "Blog".to_string(),
		tag_list_title: "Blog Tags".to_string(),
		resource_name_plural: "Blog posts".to_string(),
		resources_per_page: site_builder.site.config.blog_posts_per_page,
	};

	let mut builder = ResourceBuilder::<BlogPostMetadata, BlogPostTemplateData>::new(config);
	builder.load_all(site_builder)?;
	builder.build_all(site_builder)?;

	Ok(())
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
		site_config: &crate::SiteConfig,
	) -> anyhow::Result<BlogPostTemplateData> {
		// TODO: render markdown
		Ok(BlogPostTemplateData {
			header_image: site_config
				.cdn_url(&self.inner.header_image_file)?
				.to_string(),
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
}
