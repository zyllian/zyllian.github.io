use std::path::Path;

use zoey::Site;

#[cfg(feature = "serve")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mode {
	Build,
	Serve,
}

#[cfg(feature = "serve")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let site = Site::new(Path::new("site"))?;

	let mut mode = Mode::Build;
	for arg in std::env::args() {
		if arg == "serve" {
			mode = Mode::Serve;
			break;
		}
	}

	match mode {
		Mode::Build => site.build_once()?,
		Mode::Serve => site.serve().await?,
	}

	println!("Build complete!");

	Ok(())
}

#[cfg(not(feature = "serve"))]
fn main() -> anyhow::Result<()> {
	let site = Site::new(Path::new("site"))?;
	site.build_once()?;

	println!("Build complete!");

	Ok(())
}
