use std::path::Path;

use crate::{write, Error};

/// Render a SASS/SCSS file to the destination.
/// Other SASS/SCSS files next to the provided one will be available for
/// inclusion.
pub fn render_sass(source: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<String, Error> {
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
    write(path, css)?;
    Ok(hash)
}
