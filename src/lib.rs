#![feature(path_try_exists)]

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use anyhow::Context;
use gray_matter::{engine::yaml::YAML, matter::Matter};
use handlebars::Handlebars;
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use pulldown_cmark::{Options, Parser};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

/// Struct for the site's configuration.
#[derive(Debug, Deserialize)]
pub struct SiteConfig {
	/// The location the site is at.
	pub base_url: String,
	/// The site's title.
	pub title: String,
	/// The site's description? Not sure if this will actually be used or not
	pub description: String,
	/// The site's build directory. Defaults to <site>/build if not specified.
	pub build: Option<String>,
}

/// Struct for the front matter in templates. (nothing here yet)
#[derive(Debug, Deserialize)]
pub struct TemplateMetadata {}

/// Struct containing data to be sent to templates when rendering them.
#[derive(Debug, Serialize)]
struct TemplateData<'a> {
	/// The rendered page.
	pub page: &'a str,
}

/// Struct for the front matter in pages.
#[derive(Debug, Deserialize)]
pub struct PageMetadata {
	/// The page's title.
	pub title: Option<String>,
	/// The template to use for the page. If not specified, it defaults to "base".
	pub template: Option<String>,
}

/// Struct containing information about the site.
#[derive(Debug)]
pub struct Site {
	/// The path to the site.
	pub site_path: PathBuf,
	/// The site's configuration.
	pub config: SiteConfig,
	/// An index of available templates.
	pub template_index: HashMap<String, PathBuf>,
	/// An index of available pages.
	pub page_index: HashMap<String, PathBuf>,
}

impl Site {
	/// Creates a new site from the given path.
	pub fn new(site_path: &Path) -> anyhow::Result<Self> {
		let config: SiteConfig = serde_yaml::from_str(
			&std::fs::read_to_string(site_path.join("config.yaml"))
				.context("Failed to read site config")?,
		)
		.context("Failed to parse site config")?;

		let mut template_index = HashMap::new();
		let templates_path = site_path.join("templates");
		for entry in WalkDir::new(&templates_path).into_iter() {
			let entry = entry.context("Failed to read template entry")?;
			let path = entry.path();

			if let Some(ext) = path.extension() {
				if ext == "hbs" && entry.file_type().is_file() {
					template_index.insert(
						path.strip_prefix(&templates_path)
							.context("This really shouldn't have happened")?
							.with_extension("")
							.to_string_lossy()
							.to_string(),
						path.to_owned(),
					);
				}
			}
		}

		let mut page_index = HashMap::new();
		let pages_path = site_path.join("pages");
		for entry in WalkDir::new(&pages_path).into_iter() {
			let entry = entry.context("Failed to read page entry")?;
			let path = entry.path();

			if let Some(ext) = path.extension() {
				if ext == "md" && entry.file_type().is_file() {
					page_index.insert(
						path.strip_prefix(&pages_path)
							.context("This really shouldn't have happened")?
							.with_extension("")
							.to_string_lossy()
							.to_string(),
						path.to_owned(),
					);
				}
			}
		}

		Ok(Self {
			site_path: site_path.to_owned(),
			config,
			template_index,
			page_index,
		})
	}

	/// Builds the site once.
	pub fn build_once(&self) -> anyhow::Result<()> {
		let builder = SiteBuilder::new(self, None).prepare()?;

		for page_name in self.page_index.keys() {
			builder.build_page(page_name)?;
		}

		Ok(())
	}
}

/// Struct used to build the site.
struct SiteBuilder<'a> {
	/// The matter instance used to extract front matter.
	matter: Matter<YAML>,
	/// The Handlebars registry used to render templates.
	reg: Handlebars<'a>,
	/// The site info used to build the site.
	site: &'a Site,
	/// The path to the build directory.
	build_path: PathBuf,
	/// Whether the site should be build for viewing locally without a server.
	local_mode: Option<String>,
}

impl<'a> SiteBuilder<'a> {
	/// Creates a new site builder.
	pub fn new(site: &'a Site, local_mode: Option<String>) -> Self {
		let build_path = match &site.config.build {
			Some(build) => site.site_path.join(build),
			_ => site.site_path.join("build"),
		};

		Self {
			matter: Matter::new(),
			reg: Handlebars::new(),
			site,
			build_path,
			local_mode,
		}
	}

	/// Prepares the site builder for use.
	pub fn prepare(mut self) -> anyhow::Result<Self> {
		if std::fs::try_exists(&self.build_path)
			.context("Failed check if build directory exists")?
		{
			std::fs::remove_dir_all(self.build_path.join("static"))
				.context("Failed to remove static directory")?;
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

		fs_extra::copy_items(
			&[self.site.site_path.join("static")],
			&self.build_path,
			&fs_extra::dir::CopyOptions::default(),
		)
		.context("Failed to copy static directory")?;

		Ok(self)
	}

	/// Builds a page.
	pub fn build_page(&self, page_name: &str) -> anyhow::Result<()> {
		let page_path = self.site.page_index.get(page_name).unwrap();

		let input = std::fs::read_to_string(page_path)
			.with_context(|| format!("Failed to read page at {}", page_path.display()))?;
		let page = self.matter.matter_struct::<PageMetadata>(input);

		let parser = Parser::new_ext(&page.content, Options::all());
		let mut page_html = String::new();
		pulldown_cmark::html::push_html(&mut page_html, parser);

		let out = self.reg.render(
			&page.data.template.unwrap_or_else(|| "base".to_string()),
			&TemplateData { page: &page_html },
		)?;

		let title = match &page.data.title {
			Some(page_title) => format!("{} / {}", self.site.config.title, page_title),
			_ => self.site.config.title.clone(),
		};

		let mut output = Vec::new();
		let mut rewriter = HtmlRewriter::new(
			Settings {
				element_content_handlers: vec![element!("head", |el| {
					el.prepend(r#"<meta charset="utf-8">"#, ContentType::Html);
					el.append(&format!("<title>{}</title>", title), ContentType::Html);
					let base = self
						.local_mode
						.as_ref()
						.unwrap_or(&self.site.config.base_url);
					el.append(&format!(r#"<base href="{}">"#, base), ContentType::Html);

					Ok(())
				})],
				..Default::default()
			},
			|c: &[u8]| output.extend_from_slice(c),
		);

		rewriter.write(out.as_bytes())?;
		rewriter.end()?;

		let out = String::from_utf8(output)?;

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
}
