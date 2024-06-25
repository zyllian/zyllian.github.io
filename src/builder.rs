//! Module containing the site builder.

use std::path::PathBuf;

use eyre::Context;
use gray_matter::{engine::YAML, Matter};
use handlebars::Handlebars;
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use pulldown_cmark::{Options, Parser};
use serde::Serialize;
use url::Url;

use crate::{
	extras::Extra, resource::ResourceBuilder, util, PageMetadata, Site, ROOT_PATH, SASS_PATH,
};

/// Struct containing data to be sent to templates when rendering them.
#[derive(Debug, Serialize)]
struct TemplateData<'a, T> {
	/// The rendered page.
	pub page: &'a str,
	/// The page's title.
	pub title: &'a str,
	/// Custom head data for the page.
	pub head: Option<String>,
	/// The page's custom scripts.
	pub scripts: &'a [String],
	/// the page's custom styles.
	pub styles: &'a [String],
	/// Custom template data.
	#[serde(flatten)]
	pub extra_data: T,
}

/// Struct used to build the site.
pub struct SiteBuilder<'a> {
	/// The matter instance used to extract front matter.
	pub(crate) matter: Matter<YAML>,
	/// The Handlebars registry used to render templates.
	pub(crate) reg: Handlebars<'a>,
	/// The site info used to build the site.
	pub site: Site,
	/// The path to the build directory.
	pub build_path: PathBuf,
	/// Whether the site is going to be served locally with the dev server.
	serving: bool,

	/// Resource builder for the site's images section.
	pub images_builder:
		ResourceBuilder<crate::images::ImageMetadata, crate::images::ImageTemplateData>,
	/// Resource builder for the site's blog section.
	pub blog_builder:
		ResourceBuilder<crate::blog::BlogPostMetadata, crate::blog::BlogPostTemplateData>,
}

impl<'a> SiteBuilder<'a> {
	/// Creates a new site builder.
	pub fn new(site: Site, serving: bool) -> Self {
		let mut build_path = match &site.config.build {
			Some(build) => site.site_path.join(build),
			_ => site.site_path.join("build"),
		};
		if serving {
			build_path = site.site_path.join("build");
		}

		Self {
			matter: Matter::new(),
			reg: Handlebars::new(),
			images_builder: ResourceBuilder::new(crate::images::get_images_resource_config(&site)),
			blog_builder: ResourceBuilder::new(crate::blog::get_blog_resource_config(&site)),
			site,
			build_path,
			serving,
		}
	}

	/// Prepares the site builder for use and sets up the build directory.
	pub fn prepare(mut self) -> eyre::Result<Self> {
		if self.build_path.exists() {
			for entry in self.build_path.read_dir()? {
				let path = &entry?.path();
				if path.is_dir() {
					std::fs::remove_dir_all(path).with_context(|| {
						format!("Failed to remove directory at {}", path.display())
					})?;
				} else {
					std::fs::remove_file(path)
						.with_context(|| format!("Failed to remove file at {}", path.display()))?;
				}
			}
		} else {
			std::fs::create_dir(&self.build_path).wrap_err("Failed to create build directory")?;
		}

		for (template_name, template_path) in &self.site.template_index {
			self.reg
				.register_template_file(template_name, template_path)
				.wrap_err("Failed to register template file")?;
		}

		let root_path = self.site.site_path.join(ROOT_PATH);
		if root_path.exists() {
			for entry in walkdir::WalkDir::new(&root_path) {
				let entry = entry?;
				let path = entry.path();
				if path.is_dir() {
					continue;
				}
				let output_path = self.build_path.join(path.strip_prefix(&root_path)?);
				let parent_path = output_path.parent().expect("should never fail");
				std::fs::create_dir_all(parent_path)?;
				std::fs::copy(path, output_path)?;
			}
		}

		let images_path = self.build_path.join(crate::images::IMAGES_OUT_PATH);
		if !images_path.exists() {
			std::fs::create_dir(images_path).wrap_err("Failed to create images path")?;
		}

		self.images_builder
			.load_all(&self)
			.wrap_err("Failed to load images metadata")?;
		self.blog_builder
			.load_all(&self)
			.wrap_err("Failed to load blog metadata")?;

		Ok(self)
	}

	/// Function to rewrite HTML wow.
	pub fn rewrite_html(&self, html: String) -> eyre::Result<String> {
		let mut output = Vec::new();
		let mut rewriter = HtmlRewriter::new(
			Settings {
				element_content_handlers: vec![
					element!("body", |el| {
						el.set_attribute("class", "debug")?;
						Ok(())
					}),
					element!("head", |el| {
						el.prepend(r#"<meta charset="utf-8">"#, ContentType::Html);
						if self.serving {
							el.append(r#"<script src="/_dev.js"></script>"#, ContentType::Html);
						}

						Ok(())
					}),
					element!("a", |el| {
						if let Some(mut href) = el.get_attribute("href") {
							if let Some((command, mut new_href)) = href.split_once('$') {
								#[allow(clippy::single_match)]
								match command {
									"me" => {
										el.set_attribute(
											"rel",
											&(el.get_attribute("rel").unwrap_or_default() + " me"),
										)?;
									}
									_ => {
										new_href = &href;
									}
								}
								href = new_href.to_string();
								el.set_attribute("href", &href)?;
							}
							if let Ok(url) = Url::parse(&href) {
								if url.host().is_some() {
									// Make external links open in new tabs without referral information
									el.set_attribute(
										"rel",
										(el.get_attribute("rel").unwrap_or_default()
											+ " noopener noreferrer")
											.trim(),
									)?;
									el.set_attribute("target", "_blank")?;
								}
							}
						}

						Ok(())
					}),
				],
				..Default::default()
			},
			|c: &[u8]| output.extend_from_slice(c),
		);

		rewriter.write(html.as_bytes())?;
		rewriter.end()?;

		Ok(String::from_utf8(output)?)
	}

	pub fn build_page_raw(
		&self,
		page_metadata: PageMetadata,
		page_html: &str,
	) -> eyre::Result<String> {
		self.build_page_raw_with_extra_data(page_metadata, page_html, ())
	}

	/// Helper to build a page without writing it to disk.
	pub fn build_page_raw_with_extra_data<T>(
		&self,
		page_metadata: PageMetadata,
		page_html: &str,
		extra_data: T,
	) -> eyre::Result<String>
	where
		T: Serialize,
	{
		let extra = page_metadata
			.extra
			.and_then(|extra| crate::extras::get_extra(&extra));

		let title = match &page_metadata.title {
			Some(page_title) => format!("{} / {}", self.site.config.title, page_title),
			_ => self.site.config.title.clone(),
		};

		let head = page_metadata.embed.map(|mut embed| {
			embed.site_name.clone_from(&self.site.config.title);
			embed.build()
		});

		let out = self.reg.render(
			&page_metadata.template.unwrap_or_else(|| "base".to_string()),
			&TemplateData {
				page: page_html,
				title: &title,
				head,
				scripts: &page_metadata.scripts,
				styles: &page_metadata.styles,
				extra_data,
			},
		)?;

		// Modify HTML output
		let mut out = self.rewrite_html(out)?;

		if let Some(Extra::HtmlModification(f)) = extra {
			out = f(out, self)?;
		}

		if !self.serving {
			out = minifier::html::minify(&out);
		}

		Ok(out)
	}

	/// Builds a standard page.
	pub fn build_page(&self, page_name: &str) -> eyre::Result<()> {
		let page_path = self.site.page_index.get(page_name).expect("Missing page");

		let input = std::fs::read_to_string(page_path)
			.with_context(|| format!("Failed to read page at {}", page_path.display()))?;
		let page = self.matter.parse(&input);

		let page_metadata = if let Some(data) = page.data {
			data.deserialize()?
		} else {
			PageMetadata::default()
		};

		let parser = Parser::new_ext(&page.content, Options::all());
		let mut page_html = String::new();
		pulldown_cmark::html::push_html(&mut page_html, parser);

		let out = self.build_page_raw(page_metadata, &page_html)?;

		let out_path = self.build_path.join(page_name).with_extension("html");
		std::fs::create_dir_all(out_path.parent().unwrap())
			.with_context(|| format!("Failed to create directory for page {}", page_name))?;
		std::fs::write(&out_path, out).with_context(|| {
			format!(
				"Failed to create HTML file at {} for page {}",
				out_path.display(),
				page_name
			)
		})?;

		Ok(())
	}

	/// Builds the Sass styles in the site.
	pub fn build_sass(&self) -> eyre::Result<()> {
		let styles_path = self.build_path.join("styles");
		if !styles_path.exists() {
			std::fs::create_dir(&styles_path)?;
		}
		if self.serving {
			util::remove_dir_contents(&styles_path)
				.wrap_err("Failed to remove old contents of styles directory")?;
		}
		let sass_path = self.site.site_path.join(SASS_PATH);
		for sheet in &self.site.config.sass_styles {
			let sheet_path = sass_path.join(sheet);
			if let Some(sheet_path) = sheet_path.to_str() {
				match grass::from_path(sheet_path, &grass::Options::default()) {
					Ok(mut css) => {
						if !self.serving {
							css = minifier::css::minify(&css)
								.map_err(|err| eyre::anyhow!(err))?
								.to_string();
						}
						std::fs::write(styles_path.join(sheet).with_extension("css"), css)
							.with_context(|| {
								format!("Failed to write new CSS file for Sass: {:?}", sheet)
							})?;
					}
					Err(e) => eprintln!(
						"Failed to compile Sass stylesheet at {:?}: {}",
						sheet_path, e
					),
				}
			} else {
				eprintln!(
					"Sass stylesheet path contains invalid UTF-8: {:?}",
					sheet_path
				);
			}
		}

		Ok(())
	}

	/// Builds the site's various image pages.
	pub fn build_images(&self) -> eyre::Result<()> {
		self.images_builder.build_all(self)
	}

	/// Builds the site's blog.
	pub fn build_blog(&self) -> eyre::Result<()> {
		self.blog_builder.build_all(self)
	}
}
