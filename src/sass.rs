use std::path::Path;

use crate::{write, Error};

pub fn render_sass<A: AsRef<Path>>(
    source: impl AsRef<Path>,
    destination: A,
) -> Result<String, Error> {
    let source = source.as_ref();
    let options = match source.parent() {
        Some(parent) => grass::Options::default().load_path(parent),
        None => grass::Options::default(),
    };
    let css = grass::from_path(source, &options)?;
    let hash: String = blake3::hash(css.as_bytes())
        .to_string()
        .chars()
        .take(16)
        .collect();
    write(css, destination)?;
    Ok(hash)
}
