use lol_html::{element, RewriteStrSettings};
use serde::Serialize;

use crate::{blog::BlogPostMetadata, builder::SiteBuilder, resource::ResourceTemplateData};

#[derive(Debug)]
pub enum Extra {
	HtmlModification(fn(page: String, builder: &SiteBuilder) -> eyre::Result<String>),
}

/// Gets the extra for the given value.
pub fn get_extra(extra: &str) -> Option<Extra> {
	match extra {
		"index" => Some(Extra::HtmlModification(index)),
		_ => None,
	}
}

/// Extra to add a sidebar to the index page with recent blog posts on it.
fn index(page: String, builder: &SiteBuilder) -> eyre::Result<String> {
	#[derive(Debug, Serialize)]
	struct SidebarTemplateData<'r> {
		// resources: Vec<&'r ResourceMetadata<BlogPostMetadata>>,
		resources: Vec<ResourceTemplateData<'r, BlogPostMetadata, ()>>,
	}

	let lmd = builder.blog_builder.loaded_metadata.borrow();

	let sidebar = builder.reg.render(
		"extras/index-injection",
		&SidebarTemplateData {
			resources: lmd
				.iter()
				.take(3)
				.map(|(id, v)| ResourceTemplateData {
					resource: v,
					id: id.clone(),
					extra: (),
					timestamp: v.timestamp,
				})
				.collect(),
		},
	)?;

	Ok(lol_html::rewrite_str(
		&page,
		RewriteStrSettings {
			element_content_handlers: vec![element!("#content", move |el| {
				el.append(&sidebar, lol_html::html_content::ContentType::Html);
				Ok(())
			})],
			..Default::default()
		},
	)?)
}
