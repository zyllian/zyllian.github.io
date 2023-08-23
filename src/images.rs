use serde::{Deserialize, Serialize};

use crate::{
	builder::SiteBuilder,
	resource::{ResourceBuilder, ResourceBuilderConfig, ResourceMetadata, ResourceMethods},
	SiteConfig,
};

pub(crate) const IMAGES_PATH: &str = "images";
pub(crate) const IMAGES_OUT_PATH: &str = "i";

/// Builds the site's image pages.
pub fn build_images(site_builder: &SiteBuilder) -> anyhow::Result<()> {
	let config = ResourceBuilderConfig {
		source_path: IMAGES_PATH.to_string(),
		output_path_short: IMAGES_OUT_PATH.to_string(),
		output_path_long: "images".to_string(),
		resource_template: "image".to_string(),
		resource_list_template: "images".to_string(),
		rss_template: "rss/image".to_string(),
		rss_title: "zyl's images".to_string(),
		rss_description: "Feed of newly uploaded images from zyl's website.".to_string(),
		list_title: "Images".to_string(),
		tag_list_title: "Image Tags".to_string(),
		resource_name_plural: "Images".to_string(),
		resources_per_page: site_builder.site.config.images_per_page,
	};

	let mut builder = ResourceBuilder::<ImageMetadata, ImageTemplateData>::new(config);
	builder.load_all(site_builder)?;
	builder.build_all(site_builder)?;

	Ok(())
}

/// Definition for a remote image.
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageMetadata {
	/// The image's alt text.
	pub alt: String,
	/// The image's extra description, if any.
	pub desc: Option<String>,
	/// The image's file path.
	pub file: String,
}

/// Template data for a specific image.
#[derive(Debug, Serialize)]
struct ImageTemplateData {
	/// Direct URL to the image's CDN location.
	/// TODO: link to smaller versions on list pages
	src: String,
}

impl ResourceMethods<ImageTemplateData> for ResourceMetadata<ImageMetadata> {
	fn get_short_desc(&self) -> String {
		self.inner.desc.clone().unwrap_or_default()
	}

	fn get_extra_resource_template_data(
		&self,
		site_config: &SiteConfig,
	) -> anyhow::Result<ImageTemplateData> {
		Ok(ImageTemplateData {
			src: site_config.cdn_url(&self.inner.file)?.to_string(),
		})
	}
}
