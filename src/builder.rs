//! Module containing the site builder.

use std::path::PathBuf;

use anyhow::Context;
use gray_matter::{engine::YAML, Matter};
use handlebars::Handlebars;
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use pulldown_cmark::{Options, Parser};
use serde::Serialize;
use url::Url;
use walkdir::WalkDir;

use crate::{images::ImageMetadata, util, PageMetadata, Site, ROOT_PATH, SASS_PATH, STATIC_PATH};

/// Struct containing data to be sent to templates when rendering them.
#[derive(Debug, Serialize)]
struct TemplateData<'a> {
	/// The rendered page.
	pub page: &'a str,
}

/// Struct used to build the site.
pub struct SiteBuilder<'a> {
	/// The matter instance used to extract front matter.
	matter: Matter<YAML>,
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

	/// Prepares the site builder for use.
	pub fn prepare(mut self) -> anyhow::Result<Self> {
		let build_static_path = self.build_path.join(STATIC_PATH);
		if self.build_path.exists() {
			if build_static_path.exists() {
				std::fs::remove_dir_all(&build_static_path)
					.context("Failed to remove old static directory")?;
			}
			for entry in WalkDir::new(&self.build_path) {
				let entry = entry?;
				let path = entry.path();
				if let Some(ext) = path.extension() {
					if ext == "html" {
						std::fs::remove_file(path).with_context(|| {
							format!("Failed to remove file at {}", path.display())
						})?;
					}
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

		let static_path = self.site.site_path.join(STATIC_PATH);
		if static_path.exists() {
			fs_extra::copy_items(
				&[static_path],
				&self.build_path,
				&fs_extra::dir::CopyOptions::default(),
			)
			.context("Failed to copy static directory")?;
		}

		let images_path = self.build_path.join(crate::images::IMAGES_OUT_PATH);
		if !images_path.exists() {
			std::fs::create_dir(images_path).context("Failed to create images path")?;
		}

		Ok(self)
	}

	/// Helper to build a page without writing it to disk.
	pub fn build_page_raw(
		&self,
		page_metadata: PageMetadata,
		page_html: &str,
	) -> anyhow::Result<String> {
		let out = self.reg.render(
			&page_metadata.template.unwrap_or_else(|| "base".to_string()),
			&TemplateData { page: page_html },
		)?;

		let title = match &page_metadata.title {
			Some(page_title) => format!("{} / {}", self.site.config.title, page_title),
			_ => self.site.config.title.clone(),
		};

		let mut output = Vec::new();
		let mut rewriter = HtmlRewriter::new(
			Settings {
				element_content_handlers: vec![
					element!("head", |el| {
						el.prepend(r#"<meta charset="utf-8">"#, ContentType::Html);
						el.append(&format!("<title>{}</title>", title), ContentType::Html);
						if self.serving {
							el.append(
								&format!(r#"<script src="/{}/_dev.js"></script>"#, STATIC_PATH),
								ContentType::Html,
							);
						}

						Ok(())
					}),
					element!("a", |el| {
						if let Some(href) = el.get_attribute("href") {
							let me = href == "https://mas.to/@zyl";
							if let Ok(href) = Url::parse(&href) {
								if href.host().is_some() {
									let mut rel = String::from("noopener noreferrer");
									if me {
										rel.push_str(" me");
									}
									el.set_attribute("rel", &rel)?;
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

	/// Builds a page.
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
				"Failed to create page file at {} for page {}",
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
		let images = ImageMetadata::load_all(&self.site.site_path)?;
		ImageMetadata::build_lists(self, &images)?;
		for (id, image) in images {
			image.build(self, id)?;
		}
		Ok(())
	}
}
