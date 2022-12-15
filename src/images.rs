use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
};

use anyhow::Context;
use itertools::Itertools;
use rss::{validation::Validate, ChannelBuilder, ItemBuilder};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};
use url::Url;

use crate::{builder::SiteBuilder, link_list::Link, PageMetadata, SiteConfig};

pub(crate) const IMAGES_PATH: &str = "images";
pub(crate) const IMAGES_OUT_PATH: &str = "i";

/// Definition for a remote image.
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageMetadata {
	/// The image's title.
	pub title: String,
	/// The image's timestamp.
	#[serde(with = "time::serde::rfc3339")]
	pub timestamp: OffsetDateTime,
	/// The image's alt text.
	pub alt: String,
	/// The image's extra description, if any.
	pub desc: Option<String>,
	/// The image's file path.
	pub file: String,
	/// The image's tags.
	pub tags: Vec<String>,
}

impl ImageMetadata {
	/// Gets an image's ID from its path.
	pub fn get_id(path: &Path) -> String {
		path.with_extension("")
			.file_name()
			.expect("Should never fail")
			.to_string_lossy()
			.into_owned()
	}

	/// Loads an image's ID and metadata.
	pub fn load(path: &Path) -> anyhow::Result<(String, Self)> {
		let id = Self::get_id(path);
		let metadata: ImageMetadata = serde_yaml::from_str(&std::fs::read_to_string(path)?)?;
		Ok((id, metadata))
	}

	/// Loads all available images.
	pub fn load_all(site_path: &Path) -> anyhow::Result<Vec<(String, Self)>> {
		let images_path = site_path.join(IMAGES_PATH);
		let mut images = Vec::new();
		for e in images_path.read_dir()? {
			let p = e?.path();
			if let Some(ext) = p.extension() {
				if ext == "yml" {
					let (id, metadata) = Self::load(&p)?;
					images.push((id, metadata));
				}
			}
		}
		images.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp).reverse());
		Ok(images)
	}

	/// Builds an image's page.
	pub fn build(self, builder: &SiteBuilder, id: String) -> anyhow::Result<()> {
		let out = {
			let data = ImageTemplateData {
				image: &self,
				src: self.cdn_url(&builder.site.config)?.to_string(),
				id: &id,
				timestamp: self.timestamp,
			};
			builder.reg.render("image", &data)?
		};
		let out = builder.build_page_raw(
			PageMetadata {
				title: Some(self.title),
				..Default::default()
			},
			&out,
		)?;
		let out_path = Self::build_path(&builder.build_path, &id);
		std::fs::write(out_path, out)?;
		Ok(())
	}

	/// Gets an image's CDN url.
	pub fn cdn_url(&self, config: &SiteConfig) -> anyhow::Result<Url> {
		Ok(config.cdn_url.join(&config.s3_prefix)?.join(&self.file)?)
	}

	/// Gets an image's build path.
	pub fn build_path(build_path: &Path, id: &str) -> PathBuf {
		build_path
			.join(IMAGES_OUT_PATH)
			.join(id)
			.with_extension("html")
	}

	/// Builds the various list pages for images.
	pub fn build_lists(builder: &SiteBuilder, metadata: &[(String, Self)]) -> anyhow::Result<()> {
		let mut data = Vec::with_capacity(metadata.len());
		for (id, metadata) in metadata {
			data.push(ImageTemplateData {
				image: metadata,
				src: metadata.cdn_url(&builder.site.config)?.to_string(),
				id,
				timestamp: metadata.timestamp,
			});
		}

		fn build_list(
			builder: &SiteBuilder,
			list: Vec<&ImageTemplateData>,
			title: &str,
			tag: Option<&str>,
			out_path: &Path,
			items_per_page: usize,
		) -> anyhow::Result<()> {
			if !out_path.exists() {
				std::fs::create_dir_all(out_path)?;
			}

			let page_max = list.len() / items_per_page
				+ if list.len() % items_per_page == 0 {
					0
				} else {
					1
				};
			let mut previous = None;
			let mut next;
			for (page, iter) in list.iter().chunks(items_per_page).into_iter().enumerate() {
				next = (page + 1 != page_max).then_some(page + 2);
				let data = ImageListTemplateData {
					images: iter.copied().collect(),
					tag,
					page: page + 1,
					page_max,
					previous,
					next,
				};
				let out = builder.reg.render("images", &data)?;
				let out = builder.build_page_raw(
					PageMetadata {
						title: Some(title.to_owned()),
						..Default::default()
					},
					&out,
				)?;
				if page == 0 {
					std::fs::write(out_path.join("index.html"), &out)?;
				}
				std::fs::write(
					out_path.join(out_path.join((page + 1).to_string()).with_extension("html")),
					out,
				)?;
				previous = Some(page + 1);
			}

			Ok(())
		}

		build_list(
			builder,
			data.iter().collect(),
			"Images",
			None,
			&builder.build_path.join("images"),
			builder.site.config.images_per_page,
		)?;

		let mut tags: BTreeMap<String, Vec<&ImageTemplateData>> = BTreeMap::new();
		for image in &data {
			for tag in image.image.tags.iter().cloned() {
				tags.entry(tag).or_default().push(image);
			}
		}

		{
			let links = tags
				.iter()
				.map(|(tag, data)| {
					let count = data.len();
					(
						Link::new(
							format!("/{IMAGES_OUT_PATH}/tag/{tag}/"),
							format!("{tag} ({count})"),
						),
						count,
					)
				})
				.sorted_by(|(_, a), (_, b)| a.cmp(b).reverse())
				.map(|(l, _)| l)
				.collect();
			let out = crate::link_list::render_basic_link_list(builder, links, "Image Tags")?;
			std::fs::write(
				builder.build_path.join(IMAGES_OUT_PATH).join("tags.html"),
				out,
			)?;
		}

		for (tag, data) in tags {
			build_list(
				builder,
				data,
				&format!("Images tagged {tag}"),
				Some(tag.as_str()),
				&builder
					.build_path
					.join(IMAGES_OUT_PATH)
					.join("tag")
					.join(&tag),
				builder.site.config.images_per_page,
			)?;
		}

		let mut items = Vec::with_capacity(data.len());
		for image in data {
			items.push(
				ItemBuilder::default()
					.title(Some(image.image.title.to_owned()))
					.link(Some(
						builder
							.site
							.config
							.base_url
							.join(&format!("{IMAGES_OUT_PATH}/{}", image.id))?
							.to_string(),
					))
					.description(image.image.desc.to_owned())
					.pub_date(Some(image.image.timestamp.format(&Rfc2822)?))
					.content(Some(builder.reg.render("rss/image", &image)?))
					.build(),
			);
		}

		// Build RSS feed
		let channel = ChannelBuilder::default()
			.title("Zyllian's images".to_string())
			.link(
				builder
					.site
					.config
					.base_url
					.join("images/")
					.expect("Should never fail"),
			)
			.description("Feed of newly uploaded images from Zyllian's website.".to_string())
			.last_build_date(Some(OffsetDateTime::now_utc().format(&Rfc2822)?))
			.items(items)
			.build();
		channel.validate().context("Failed to validate RSS feed")?;
		let out = channel.to_string();
		std::fs::write(builder.build_path.join("images").join("rss.xml"), out)?;

		Ok(())
	}
}

/// Template data for a specific image.
#[derive(Debug, Serialize)]
struct ImageTemplateData<'i> {
	/// The image's regular metadata.
	#[serde(flatten)]
	image: &'i ImageMetadata,
	/// Direct URL to the image's CDN location.
	/// TODO: link to smaller versions on list pages
	src: String,
	/// The image's ID.
	id: &'i str,
	/// The image's timestamp. (Duplicated to change the serialization method.)
	#[serde(serialize_with = "time::serde::rfc2822::serialize")]
	timestamp: OffsetDateTime,
}

/// Template data for image lists.
#[derive(Debug, Serialize)]
struct ImageListTemplateData<'i> {
	/// The list of images to display.
	images: Vec<&'i ImageTemplateData<'i>>,
	/// The current tag, if any.
	tag: Option<&'i str>,
	/// The current page.
	page: usize,
	/// The total number of pages.
	page_max: usize,
	/// The previous page, if any.
	previous: Option<usize>,
	/// The next page, if any.
	next: Option<usize>,
}
