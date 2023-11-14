use lol_html::{element, RewriteStrSettings};
use serde::Serialize;

use crate::{blog::BlogPostMetadata, builder::SiteBuilder, resource::ResourceMetadata};

#[derive(Debug)]
pub enum Extra {
	HtmlModification(fn(page: String, builder: &SiteBuilder) -> anyhow::Result<String>),
}

/// Gets the extra for the given value.
pub fn get_extra(extra: &str) -> Option<Extra> {
	match extra {
		"index" => Some(Extra::HtmlModification(index)),
		_ => None,
	}
}

/// Extra to add a sidebar to the index page with recent blog posts on it.
fn index(page: String, builder: &SiteBuilder) -> anyhow::Result<String> {
	#[derive(Debug, Serialize)]
	struct SidebarTemplateData<'r> {
		resources: Vec<&'r ResourceMetadata<BlogPostMetadata>>,
	}

	let lmd = builder.blog_builder.loaded_metadata.borrow();

	let sidebar = builder.reg.render(
		"extras/index-injection",
		&SidebarTemplateData {
			resources: lmd.iter().take(3).map(|(_, v)| v).collect(),
		},
	)?;

	Ok(lol_html::rewrite_str(
		&page,
		RewriteStrSettings {
			element_content_handlers: vec![element!("#content", |el| {
				el.append(&sidebar, lol_html::html_content::ContentType::Html);
				Ok(())
			})],
			..Default::default()
		},
	)?)
}
