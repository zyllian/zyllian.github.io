use std::path::Path;

use zoey::Site;

fn main() -> anyhow::Result<()> {
	let builder = Site::new(Path::new("site"))?;
	builder.build_once()?;

	println!("Build complete!");

	Ok(())
}
