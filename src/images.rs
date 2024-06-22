use serde::{Deserialize, Serialize};

use crate::{
	resource::{EmbedMetadata, ResourceBuilderConfig, ResourceMetadata, ResourceMethods},
	Site, SiteConfig,
};

pub(crate) const IMAGES_PATH: &str = "images";
pub(crate) const IMAGES_OUT_PATH: &str = "i";

/// Gets the resource configuration for images.
pub fn get_images_resource_config(site: &Site) -> ResourceBuilderConfig {
	ResourceBuilderConfig {
		source_path: IMAGES_PATH.to_string(),
		output_path_short: IMAGES_OUT_PATH.to_string(),
		output_path_long: "images".to_string(),
		resource_template: "image".to_string(),
		resource_list_template: "images".to_string(),
		rss_template: "rss/image".to_string(),
		rss_title: "zyl's images".to_string(),
		rss_description: "feed of newly uploaded images from zyl's website.".to_string(),
		list_title: "images".to_string(),
		tag_list_title: "image tags".to_string(),
		resource_name_plural: "images".to_string(),
		resources_per_page: site.config.images_per_page,
	}
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

impl ImageMetadata {
	fn get_image_url(&self, site_config: &SiteConfig) -> eyre::Result<String> {
		Ok(site_config.cdn_url(&self.file)?.to_string())
	}
}

/// Template data for a specific image.
#[derive(Debug, Serialize)]
pub struct ImageTemplateData {
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
	) -> eyre::Result<ImageTemplateData> {
		Ok(ImageTemplateData {
			src: self.inner.get_image_url(site_config)?,
		})
	}

	fn get_head_data(&self, site_config: &SiteConfig) -> eyre::Result<String> {
		Ok(EmbedMetadata {
			title: self.title.clone(),
			site_name: site_config.title.clone(),
			description: self.inner.desc.clone(),
			image: Some(self.inner.get_image_url(site_config)?),
			url: None,
			theme_color: "rgb(255, 196, 252)".to_string(),
			large_image: true,
		}
		.build())
	}
}
