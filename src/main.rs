use std::path::Path;

use zyl_site::Site;

#[cfg(feature = "serve")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mode {
	Build,
	Serve,
}

#[cfg(feature = "serve")]
#[tokio::main]
async fn main() -> eyre::Result<()> {
	#[cfg(feature = "color-eyre")]
	color_eyre::install()?;

	let site = Site::new(&Path::new("site").canonicalize()?)?;

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
fn main() -> eyre::Result<()> {
	let site = Site::new(&Path::new("site").canonicalize()?)?;
	site.build_once()?;

	println!("Build complete!");

	Ok(())
}
