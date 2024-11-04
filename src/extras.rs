use lol_html::{element, RewriteStrSettings};
use serde::Serialize;

use crate::{builder::SiteBuilder, resource::ResourceTemplateData};

#[derive(Debug)]
pub enum Extra {
	Basic(&'static str),
	HtmlModification(fn(page: String, builder: &SiteBuilder) -> eyre::Result<String>),
}

impl Extra {
	/// runs the handler for the extra
	pub fn handle(&self, page: String, builder: &SiteBuilder) -> eyre::Result<String> {
		match self {
			Self::Basic(template) => {
				let content = builder.reg.render(template, &())?;
				append_to(&page, &content, "main.page")
			}
			Self::HtmlModification(f) => (f)(page, builder),
		}
	}
}

/// Gets the extra for the given value.
pub fn get_extra(extra: &str) -> Option<Extra> {
	match extra {
		"index" => Some(Extra::HtmlModification(index)),
		"click" => Some(Extra::Basic("extras/click")),
		_ => None,
	}
}

/// Extra to append a tempalte to the page.
fn append_to(page: &str, content: &str, selector: &str) -> eyre::Result<String> {
	Ok(lol_html::rewrite_str(
		page,
		RewriteStrSettings {
			element_content_handlers: vec![element!(selector, move |el| {
				el.append(content, lol_html::html_content::ContentType::Html);
				Ok(())
			})],
			..Default::default()
		},
	)?)
}

/// Extra to add a sidebar to the index page with recent blog posts on it.
fn index(page: String, builder: &SiteBuilder) -> eyre::Result<String> {
	#[derive(Debug, Serialize)]
	struct SidebarTemplateData<'r> {
		resources: Vec<ResourceTemplateData<'r>>,
	}

	let sidebar = builder.reg.render(
		"extras/index-injection",
		&SidebarTemplateData {
			resources: builder
				.resource_builders
				.get("blog")
				.expect("missing blog builder")
				.loaded_metadata
				.iter()
				.take(3)
				.map(|(id, v)| ResourceTemplateData {
					resource: v,
					id: id.clone(),
					timestamp: v.timestamp,
				})
				.collect(),
		},
	)?;

	append_to(&page, &sidebar, "#content")
}
