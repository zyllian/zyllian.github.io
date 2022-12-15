//! Module containing code to serve the dev site.

use std::{
	collections::HashMap,
	net::SocketAddr,
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};

use anyhow::Context;
use futures::SinkExt;
use hotwatch::{Event, Hotwatch};
use warp::{
	hyper::StatusCode,
	path::FullPath,
	reply::Response,
	ws::{Message, WebSocket},
	Filter,
};

use crate::{
	images::ImageMetadata, Site, SiteBuilder, PAGES_PATH, ROOT_PATH, SASS_PATH, STATIC_PATH,
	TEMPLATES_PATH,
};

fn with_build_path(
	build_path: PathBuf,
) -> impl Filter<Extract = (PathBuf,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || build_path.clone())
}

/// Helper to get the "name" of a path.
fn get_name(path: &Path) -> (PathBuf, String) {
	let name = path.with_extension("");
	let name_str = name.display().to_string();
	(name, name_str)
}

/// Helper to make a path relative.
fn rel(path: &Path, prefix: &Path) -> Result<PathBuf, std::path::StripPrefixError> {
	Ok(path.strip_prefix(prefix)?.to_owned())
}

/// Creates or updates a resource.
fn create(
	builder: &mut SiteBuilder,
	path: &Path,
	relative_path: &Path,
	build: bool,
) -> anyhow::Result<()> {
	if path.is_dir() {
		return Ok(());
	}
	if let Ok(page_path) = relative_path.strip_prefix(PAGES_PATH) {
		let (_page_name, page_name_str) = get_name(page_path);

		builder
			.site
			.page_index
			.insert(page_name_str.clone(), path.to_owned());
		if build {
			builder.build_page(&page_name_str)?;
		}
	} else if let Ok(template_path) = relative_path.strip_prefix(TEMPLATES_PATH) {
		let (_template_name, template_name_str) = get_name(template_path);
		builder.refresh_template(&template_name_str, path)?;
		if build {
			builder.site.build_all_pages(builder)?;
			builder.build_images()?;
		}
	} else if let Ok(_static_path) = relative_path.strip_prefix(STATIC_PATH) {
		std::fs::copy(path, builder.build_path.join(relative_path))?;
	} else if relative_path.display().to_string() == "config.yaml" {
		let new_config = serde_yaml::from_str(&std::fs::read_to_string(path)?)?;
		builder.site.config = new_config;
		builder.site.build_all_pages(builder)?;
	} else if let Ok(_sass_path) = relative_path.strip_prefix(SASS_PATH) {
		if build {
			builder.build_sass().context("Failed to rebuild Sass")?;
		}
	} else if let Ok(root_path) = relative_path.strip_prefix(ROOT_PATH) {
		std::fs::copy(path, builder.build_path.join(root_path))?;
	} else if let Ok(_image_path) = relative_path.strip_prefix(crate::images::IMAGES_PATH) {
		// HACK: this could get very inefficient with a larger number of images. should definitely optimize
		builder.build_images()?;
	}

	Ok(())
}

/// Removes an existing resource.
fn remove(builder: &mut SiteBuilder, path: &Path, relative_path: &Path) -> anyhow::Result<()> {
	if path.is_dir() {
		return Ok(());
	}
	if let Ok(page_path) = relative_path.strip_prefix(PAGES_PATH) {
		let (page_name, page_name_str) = get_name(page_path);

		builder.site.page_index.remove(&page_name_str);
		std::fs::remove_file(builder.build_path.join(page_name.with_extension("html")))
			.with_context(|| format!("Failed to remove page at {:?}", path))?;
	} else if let Ok(template_path) = relative_path.strip_prefix(TEMPLATES_PATH) {
		let (_template_name, template_name_str) = get_name(template_path);
		builder.site.template_index.remove(&template_name_str);
		builder.reg.unregister_template(&template_name_str);
		builder
			.site
			.build_all_pages(builder)
			.context("Failed to rebuild pages")?;
	} else if let Ok(_static_path) = relative_path.strip_prefix(STATIC_PATH) {
		let to_remove = builder.build_path.join(relative_path);
		std::fs::remove_file(&to_remove)
			.with_context(|| format!("Failed to remove file at {:?}", to_remove))?;
	} else if let Ok(_sass_path) = relative_path.strip_prefix(SASS_PATH) {
		builder.build_sass().context("Failed to rebuild Sass")?;
	} else if let Ok(root_path) = relative_path.strip_prefix(ROOT_PATH) {
		std::fs::remove_file(builder.build_path.join(root_path))?;
	} else if let Ok(_image_path) = relative_path.strip_prefix(crate::images::IMAGES_PATH) {
		let p = ImageMetadata::build_path(&builder.build_path, &ImageMetadata::get_id(path));
		std::fs::remove_file(p)?;
		// HACK: same as in `create`
		builder.build_images()?;
	}

	Ok(())
}

/// Decides whether to skip a path in the watcher.
fn skip_path(builder: &SiteBuilder, path: &Path) -> bool {
	path.strip_prefix(&builder.build_path).is_ok()
}

impl Site {
	/// Serves the site for development. Don't use this in production.
	pub async fn serve(self) -> anyhow::Result<()> {
		let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

		let mut builder = SiteBuilder::new(self, true).prepare()?;
		let site = &builder.site;
		let build_path = builder.build_path.clone();

		// Perform initial build
		for page_name in site.page_index.keys() {
			if let Err(e) = builder.build_page(page_name) {
				eprintln!("Failed to build page {}: {}", page_name, e);
			}
		}
		builder.build_sass().context("Failed to build Sass")?;
		builder
			.build_images()
			.context("Failed to build image pages")?;

		// Map of websocket connections
		let peers: Arc<Mutex<HashMap<SocketAddr, WebSocket>>> =
			Arc::new(Mutex::new(HashMap::new()));

		// Watch for changes to the site
		let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize");
		let hw_peers = peers.clone();
		hotwatch
			.watch(site.site_path.clone(), move |event| {
				let peers = hw_peers.clone();

				match (|| match event {
					Event::Write(path) => {
						if skip_path(&builder, &path) {
							Ok(false)
						} else {
							let rel = rel(&path, &builder.site.site_path)?;
							println!("CHANGED - {:?}", rel);
							create(&mut builder, &path, &rel, true)?;
							Ok::<_, anyhow::Error>(true)
						}
					}
					Event::Create(path) => {
						if skip_path(&builder, &path) {
							Ok(false)
						} else {
							let rel = rel(&path, &builder.site.site_path)?;
							println!("CREATED - {:?}", rel);
							create(&mut builder, &path, &rel, true)?;
							Ok(true)
						}
					}
					Event::Remove(path) => {
						if skip_path(&builder, &path) {
							Ok(false)
						} else {
							let rel = rel(&path, &builder.site.site_path)?;
							println!("REMOVED - {:?}", rel);
							remove(&mut builder, &path, &rel)?;
							Ok(true)
						}
					}
					Event::Rename(old, new) => {
						if skip_path(&builder, &old) && skip_path(&builder, &new) {
							Ok(false)
						} else {
							let old_rel = rel(&old, &builder.site.site_path)?;
							let new_rel = rel(&new, &builder.site.site_path)?;
							println!("RENAMED - {:?} -> {:?}", old_rel, new_rel);
							create(&mut builder, &new, &new_rel, false)?;
							remove(&mut builder, &old, &old_rel)?;
							Ok(true)
						}
					}
					_ => Ok(false),
				})() {
					Ok(reload) => {
						if reload {
							// Send reload event to connected websockets
							let mut peers = peers.lock().unwrap();
							let mut to_remove = Vec::new();
							for (addr, peer) in peers.iter_mut() {
								let task = async {
									peer.send(Message::text("reload".to_string())).await?;
									Ok::<_, anyhow::Error>(())
								};
								to_remove.push(*addr);
								if let Err(e) = futures::executor::block_on(task) {
									eprintln!("{}", e);
								}
							}
							for addr in &to_remove {
								peers.remove(addr);
							}
						}
					}
					Err(e) => eprintln!("Failed to update: {}", e),
				}
			})
			.expect("Failed to watch file");

		let routes = warp::any()
			.and(warp::ws())
			.and(warp::filters::addr::remote())
			.and_then(move |ws: warp::ws::Ws, addr| {
				let peers = peers.clone();
				async move {
					// Add websocket connection to peers list
					if let Some(addr) = addr {
						let peers = peers.clone();
						return Ok(ws.on_upgrade(move |websocket| async move {
							peers.lock().unwrap().insert(addr, websocket);
						}));
					}
					Err(warp::reject())
				}
			})
			.or(warp::any().and(warp::get()).and(
				warp::path::full()
					.and(with_build_path(build_path.clone()))
					.and_then(move |path: FullPath, build_path: PathBuf| async move {
						// Serve static files
						let p = &path.as_str()[1..];
						let p = percent_encoding::percent_decode_str(p)
							.decode_utf8()
							.expect("Failed to decode URL");

						if p == "static/_dev.js" {
							let res = Response::new(include_str!("./refresh_websocket.js").into());
							return Ok(res);
						}

						let mut p = build_path.join(p.as_ref());

						if !p.exists() {
							p = p.with_extension("html");
						}
						if p.is_dir() {
							p = p.join("index.html");
						}

						if p.exists() {
							let mut res = Response::new("".into());
							match std::fs::read_to_string(&p) {
								Ok(body) => {
									*res.body_mut() = body.into();
								}
								Err(e) => {
									eprintln!("{}", e);
									*res.body_mut() = format!("Failed to load: {}", e).into();
									*res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
								}
							}
							return Ok(res);
						}
						Err(warp::reject())
					}),
			))
			.or(warp::any()
				.and(warp::path::full())
				.and_then(move |path: FullPath| {
					let build_path = build_path.clone();
					async move {
						// Handle missing files
						println!("404 - {}", path.as_str());
						let body = match std::fs::read_to_string(build_path.join("404.html")) {
							Ok(body) => body,
							_ => "404 Not Found".to_string(),
						};
						let mut res = Response::new(body.into());
						*res.status_mut() = StatusCode::NOT_FOUND;
						Ok::<_, std::convert::Infallible>(res)
					}
				}));

		println!("Starting server at http://{}", addr);
		warp::serve(routes).run(addr).await;

		Ok(())
	}
}

impl<'a> SiteBuilder<'a> {
	/// Refreshes a template to ensure it's up to date.
	pub fn refresh_template(
		&mut self,
		template_name: &str,
		template_path: &Path,
	) -> anyhow::Result<()> {
		self.reg
			.register_template_file(template_name, template_path)?;

		Ok(())
	}
}
