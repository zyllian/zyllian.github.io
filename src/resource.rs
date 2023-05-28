use std::{
	collections::BTreeMap,
	marker::PhantomData,
	path::{Path, PathBuf},
};

use anyhow::Context;
use itertools::Itertools;
use rss::{validation::Validate, ChannelBuilder, ItemBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::{builder::SiteBuilder, link_list::Link, PageMetadata, SiteConfig};

/// Metadata for resources.
#[derive(Debug, Deserialize, Serialize)]
pub struct ResourceMetadata<T> {
	/// The resource's title.
	pub title: String,
	/// The resource's timestamp.
	#[serde(with = "time::serde::rfc3339")]
	pub timestamp: OffsetDateTime,
	/// The resource's tags.
	pub tags: Vec<String>,
	/// Extra resource data not included.
	#[serde(flatten)]
	pub inner: T,
}

#[derive(Debug, Serialize)]
pub struct ResourceTemplateData<'r, M, E> {
	/// The resource's metadata.
	#[serde(flatten)]
	resource: &'r ResourceMetadata<M>,
	/// The resource's ID.
	id: String,
	/// Extra data to be passed to the template.
	#[serde(flatten)]
	extra: E,
	/// The resource's timestamp. Duplicated to change serialization method.
	#[serde(serialize_with = "ResourceTemplateData::<M, E>::timestamp_formatter")]
	timestamp: OffsetDateTime,
}

impl<'r, M, E> ResourceTemplateData<'r, M, E> {
	fn timestamp_formatter<S>(timestamp: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let out = timestamp
			.format(
				&time::format_description::parse("[weekday], [month repr:long] [day], [year]")
					.expect("Should never fail"),
			)
			.expect("Should never fail");
		serializer.serialize_str(&out)
	}
}

/// Trait for getting extra template data from resource metadata.
pub trait ResourceMethods<E>
where
	E: Serialize,
{
	fn get_short_desc(&self) -> String;

	fn get_extra_resource_template_data(&self, site_config: &SiteConfig) -> anyhow::Result<E>;
}

#[derive(Debug, Serialize)]
struct ResourceListTemplateData<'r, M, E> {
	resources: Vec<&'r ResourceTemplateData<'r, M, E>>,
	tag: Option<&'r str>,
	page: usize,
	page_max: usize,
	previous: Option<usize>,
	next: Option<usize>,
}

/// Config for the resource builder.
#[derive(Debug)]
pub struct ResourceBuilderConfig {
	/// Path to where the resources should be loaded from.
	pub source_path: String,
	/// Path to where the resource pages should be written to.
	pub output_path_short: String,
	/// Path to where the main list should be written to.
	pub output_path_long: String,
	/// The template used to render a single resource.
	pub resource_template: String,
	/// The template used to render a list of resources.
	pub resource_list_template: String,
	/// Template used when rendering the RSS feed.
	pub rss_template: String,
	/// The RSS feed's title.
	pub rss_title: String,
	/// The description for the RSS feed.
	pub rss_description: String,
	/// Title for the main list of resources.
	pub list_title: String,
	/// Title for the page containing a list of tags.
	pub tag_list_title: String,
	/// Name for the resource type in plural.
	pub resource_name_plural: String,
	/// The number of resources to display on a single page.
	pub resources_per_page: usize,
}

/// Helper to genericize resource building.
#[derive(Debug)]
pub struct ResourceBuilder<M, E> {
	/// The builder's config.
	config: ResourceBuilderConfig,
	/// The currently loaded resource metadata.
	loaded_metadata: Vec<(String, ResourceMetadata<M>)>,
	_extra: PhantomData<E>,
}

impl<M, E> ResourceBuilder<M, E>
where
	M: Serialize + DeserializeOwned,
	E: Serialize,
	ResourceMetadata<M>: ResourceMethods<E>,
{
	/// Creates a new resource builder.
	pub fn new(config: ResourceBuilderConfig) -> Self {
		Self {
			config,
			loaded_metadata: Default::default(),
			_extra: Default::default(),
		}
	}

	/// Gets a resource's ID from its path.
	fn get_id(path: &Path) -> String {
		path.with_extension("")
			.file_name()
			.expect("Should never fail")
			.to_string_lossy()
			.into_owned()
	}

	/// Loads resource metadata from the given path.
	fn load(path: &Path) -> anyhow::Result<(String, ResourceMetadata<M>)> {
		let id = Self::get_id(path);
		let metadata = serde_yaml::from_str(&std::fs::read_to_string(path)?)?;
		Ok((id, metadata))
	}

	/// Loads all resource metadata from the given config.
	pub fn load_all(&mut self, site_path: &Path) -> anyhow::Result<()> {
		self.loaded_metadata.clear();
		for e in site_path.join(&self.config.source_path).read_dir()? {
			let p = e?.path();
			if let Some("yml") = p.extension().and_then(|e| e.to_str()) {
				let (id, metadata) = Self::load(&p)?;
				self.loaded_metadata.push((id, metadata));
			}
		}
		self.loaded_metadata
			.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
		Ok(())
	}

	/// Gets a resource's build path.
	fn build_path(&self, base_path: &Path, id: &str) -> PathBuf {
		base_path
			.join(&self.config.output_path_short)
			.join(id)
			.with_extension("html")
	}

	/// Builds a single resource page.
	fn build(
		&self,
		builder: &SiteBuilder,
		id: String,
		resource: &ResourceMetadata<M>,
	) -> anyhow::Result<()> {
		let out_path = self.build_path(&builder.build_path, &id);
		let out = {
			let data = ResourceTemplateData {
				resource,
				extra: resource.get_extra_resource_template_data(&builder.site.config)?,
				id,
				timestamp: resource.timestamp,
			};
			builder.reg.render(&self.config.resource_template, &data)?
		};

		let out = builder.build_page_raw(
			PageMetadata {
				title: Some(resource.title.clone()),
				..Default::default()
			},
			&out,
		)?;
		std::fs::write(out_path, out)?;

		Ok(())
	}

	pub fn build_all(&self, builder: &SiteBuilder) -> anyhow::Result<()> {
		for (id, resource) in &self.loaded_metadata {
			self.build(builder, id.clone(), resource)?;
		}

		let mut data = Vec::with_capacity(self.loaded_metadata.len());
		for (id, resource) in &self.loaded_metadata {
			let extra = resource.get_extra_resource_template_data(&builder.site.config)?;
			data.push(ResourceTemplateData {
				resource,
				extra,
				id: id.clone(),
				timestamp: resource.timestamp,
			});
		}

		fn build_list<M, E>(
			builder: &SiteBuilder,
			config: &ResourceBuilderConfig,
			list: Vec<&ResourceTemplateData<M, E>>,
			title: &str,
			tag: Option<&str>,
			out_path: &Path,
			items_per_page: usize,
		) -> anyhow::Result<()>
		where
			M: Serialize,
			E: Serialize,
		{
			if !out_path.exists() {
				std::fs::create_dir_all(out_path)?;
			}

			let page_max = list.len() / items_per_page + (list.len() % items_per_page).min(1);
			let mut previous = None;
			let mut next;
			for (page, iter) in list.iter().chunks(items_per_page).into_iter().enumerate() {
				next = (page + 1 != page_max).then_some(page + 2);
				let data = ResourceListTemplateData {
					resources: iter.copied().collect(),
					tag,
					page: page + 1,
					page_max,
					previous,
					next,
				};
				let out = builder.reg.render(&config.resource_list_template, &data)?;
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
					out_path.join((page + 1).to_string()).with_extension("html"),
					out,
				)?;
				previous = Some(page + 1);
			}

			Ok(())
		}

		let out_path = builder.build_path.join(&self.config.output_path_short);

		// Build main list of resources
		build_list(
			builder,
			&self.config,
			data.iter().collect(),
			&self.config.list_title,
			None,
			&builder.build_path.join(&self.config.output_path_long),
			self.config.resources_per_page,
		)?;

		// Build resource lists by tag
		let mut tags: BTreeMap<String, Vec<&ResourceTemplateData<M, E>>> = BTreeMap::new();
		for resource in &data {
			for tag in resource.resource.tags.iter().cloned() {
				tags.entry(tag).or_default().push(resource);
			}
		}

		// Build list of tags
		{
			let links = tags
				.iter()
				.map(|(tag, data)| {
					let count = data.len();
					(
						Link::new(
							format!("/{}/tag/{tag}/", self.config.output_path_short),
							format!("{tag} ({count})"),
						),
						count,
					)
				})
				.sorted_by(|(_, a), (_, b)| b.cmp(a))
				.map(|(l, _)| l)
				.collect();
			let out = crate::link_list::render_basic_link_list(
				builder,
				links,
				&self.config.tag_list_title,
			)?;
			std::fs::write(out_path.join("tags.html"), out)?;
		}

		for (tag, data) in tags {
			build_list(
				builder,
				&self.config,
				data,
				&format!("{} tagged {tag}", self.config.resource_name_plural),
				Some(tag.as_str()),
				&out_path.join("tag").join(&tag),
				self.config.resources_per_page,
			)?;
		}

		// Build RSS feed
		let mut items = Vec::with_capacity(data.len());
		for resource in data {
			items.push(
				ItemBuilder::default()
					.title(Some(resource.resource.title.to_owned()))
					.link(Some(
						builder
							.site
							.config
							.base_url
							.join(&format!(
								"{}/{}",
								self.config.output_path_short, resource.id
							))?
							.to_string(),
					))
					.description(Some(resource.resource.get_short_desc()))
					.pub_date(Some(resource.timestamp.format(&Rfc2822)?))
					.content(Some(
						builder.reg.render(&self.config.rss_template, &resource)?,
					))
					.build(),
			)
		}

		let channel = ChannelBuilder::default()
			.title(self.config.rss_title.clone())
			.link(
				builder
					.site
					.config
					.base_url
					.join(&format!("{}/", self.config.output_path_long))
					.expect("Should never fail"),
			)
			.description(self.config.rss_description.clone())
			.last_build_date(Some(OffsetDateTime::now_utc().format(&Rfc2822)?))
			.items(items)
			.build();
		channel.validate().context("Failed to validate RSS feed")?;
		let out = channel.to_string();
		std::fs::write(
			builder
				.build_path
				.join(&self.config.output_path_long)
				.join("rss.xml"),
			out,
		)?;

		Ok(())
	}
}
