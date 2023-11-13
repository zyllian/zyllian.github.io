//! Module containing the site builder.

use std::path::PathBuf;

use anyhow::Context;
use gray_matter::{engine::YAML, Matter};
use handlebars::Handlebars;
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use pulldown_cmark::{Options, Parser};
use serde::Serialize;
use url::Url;

use crate::{util, PageMetadata, Site, ROOT_PATH, SASS_PATH};

/// Struct containing data to be sent to templates when rendering them.
#[derive(Debug, Serialize)]
struct TemplateData<'a, T> {
	/// The rendered page.
	pub page: &'a str,
	/// Custom template data.
	#[serde(flatten)]
	pub extra: T,
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
			site,
			build_path,
			serving,
		}
	}

	/// Prepares the site builder for use and sets up the build directory.
	pub fn prepare(mut self) -> anyhow::Result<Self> {
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
			std::fs::create_dir(&self.build_path).context("Failed to create build directory")?;
		}

		for (template_name, template_path) in &self.site.template_index {
			self.reg
				.register_template_file(template_name, template_path)
				.context("Failed to register template file")?;
		}

		let root_path = self.site.site_path.join(ROOT_PATH);
		if root_path.exists() {
			for entry in root_path.read_dir()? {
				let entry = entry?;
				let path = entry.path();
				std::fs::copy(&path, self.build_path.join(path.strip_prefix(&root_path)?))?;
			}
		}

		let images_path = self.build_path.join(crate::images::IMAGES_OUT_PATH);
		if !images_path.exists() {
			std::fs::create_dir(images_path).context("Failed to create images path")?;
		}

		Ok(self)
	}

	pub fn build_page_raw(
		&self,
		page_metadata: PageMetadata,
		page_html: &str,
	) -> anyhow::Result<String> {
		self.build_page_raw_extra(page_metadata, page_html, ())
	}

	/// Helper to build a page without writing it to disk.
	pub fn build_page_raw_extra<T>(
		&self,
		page_metadata: PageMetadata,
		page_html: &str,
		extra: T,
	) -> anyhow::Result<String>
	where
		T: Serialize,
	{
		let out = self.reg.render(
			&page_metadata.template.unwrap_or_else(|| "base".to_string()),
			&TemplateData {
				page: page_html,
				extra,
			},
		)?;

		let title = match &page_metadata.title {
			Some(page_title) => format!("{} / {}", self.site.config.title, page_title),
			_ => self.site.config.title.clone(),
		};

		// Modify HTML output
		let mut output = Vec::new();
		let mut rewriter = HtmlRewriter::new(
			Settings {
				element_content_handlers: vec![
					element!("head", |el| {
						el.prepend(r#"<meta charset="utf-8">"#, ContentType::Html);
						el.append(&format!("<title>{}</title>", title), ContentType::Html);
						if self.serving {
							el.append(r#"<script src="/_dev.js"></script>"#, ContentType::Html);
						}

						Ok(())
					}),
					element!("a", |el| {
						if let Some(mut href) = el.get_attribute("href") {
							if let Some((command, new_href)) = href.split_once('$') {
								#[allow(clippy::single_match)]
								match command {
									"me" => {
										el.set_attribute(
											"rel",
											&(el.get_attribute("rel").unwrap_or_default() + " me"),
										)?;
									}
									_ => {}
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

		rewriter.write(out.as_bytes())?;
		rewriter.end()?;

		let mut out = String::from_utf8(output)?;
		if !self.serving {
			out = minifier::html::minify(&out);
		}

		Ok(out)
	}

	/// Builds a standard page.
	pub fn build_page(&self, page_name: &str) -> anyhow::Result<()> {
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
	pub fn build_sass(&self) -> anyhow::Result<()> {
		let styles_path = self.build_path.join("styles");
		if !styles_path.exists() {
			std::fs::create_dir(&styles_path)?;
		}
		if self.serving {
			util::remove_dir_contents(&styles_path)
				.context("Failed to remove old contents of styles directory")?;
		}
		let sass_path = self.site.site_path.join(SASS_PATH);
		for sheet in &self.site.config.sass_styles {
			let sheet_path = sass_path.join(sheet);
			if let Some(sheet_path) = sheet_path.to_str() {
				match grass::from_path(sheet_path, &grass::Options::default()) {
					Ok(mut css) => {
						if !self.serving {
							css = minifier::css::minify(&css)
								.map_err(|err| anyhow::anyhow!(err))?
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
	pub fn build_images(&self) -> anyhow::Result<()> {
		crate::images::build_images(self)
	}

	/// Builds the site's blog.
	pub fn build_blog(&self) -> anyhow::Result<()> {
		crate::blog::build_blog(self)
	}
}
