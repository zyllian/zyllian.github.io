use std::path::Path;

use zyl_site::Site;

#[cfg(feature = "serve")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mode {
	Build,
	Serve,
	Now,
}

#[cfg(feature = "serve")]
#[tokio::main]
async fn main() -> eyre::Result<()> {
	use time::{format_description::well_known::Rfc3339, OffsetDateTime};

	#[cfg(feature = "color-eyre")]
	color_eyre::install()?;

	let site = Site::new(&Path::new("site").canonicalize()?)?;

	let mut mode = Mode::Build;
	for arg in std::env::args() {
		if arg == "serve" {
			mode = Mode::Serve;
			break;
		} else if arg == "now" {
			mode = Mode::Now;
			break;
		}
	}

	match mode {
		Mode::Build => {
			println!("Building site...");
			site.build_once()?
		}
		Mode::Serve => site.serve().await?,
		Mode::Now => {
			let time = OffsetDateTime::now_utc();
			println!(
				"{}",
				time.format(&Rfc3339)
					.expect("failed to format the current time")
			);
		}
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
