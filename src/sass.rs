use std::{io, path::Path};

use crate::write;

/// Error type for SASS/SCSS compilation operations.
#[derive(thiserror::Error, Debug)]
pub enum SassError {
    /// I/O error.
    #[error("io error: {0}")]
    IO(#[from] io::Error),
    /// SASS/SCSS compilation failed.
    #[error("failed to compile sass: {0}")]
    SassCompile(#[from] Box<grass::Error>),
}

/// Render a SASS/SCSS file to the destination.
/// Other SASS/SCSS files next to the provided one will be available for
/// inclusion.
///
/// # Errors
///
/// Returns an error if the SASS file cannot be compiled or if the output cannot be written.
pub fn render_sass(source: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<String, SassError> {
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
