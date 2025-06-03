//! Pichu is the static site generator designed to evolve with your needs.
//!
//! # Example
//!
//! ```
//! fn main() -> Box<dyn std::error::Error> {
//!     pichu::glob("content/blog/*.md")?
//!         .parse_markdown::<BlogFrontmatter>()?
//!         .render_each(render_blog_post, |post| format!("dist/blog/{}/index.html", post.basename))?
//!         .render_all(render_blog, "dist/blog/index.html")?;
//!     Ok(())
//! }
//! ```

use rayon::prelude::*;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[cfg(feature = "markdown")]
mod markdown;
#[cfg(feature = "markdown")]
pub use markdown::Markdown;

#[cfg(feature = "sass")]
mod sass;
#[cfg(feature = "sass")]
pub use sass::render_sass;

/// The error type returned in this crate.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IO(#[from] io::Error),
    #[error("{0}")]
    GlobPatternError(#[from] glob::PatternError),
    #[error("{0}")]
    GlobError(#[from] glob::GlobError),
    #[error("render fn error: {0}")]
    RenderFn(#[from] Box<dyn std::error::Error + Send + Sync>),
    // markdown
    #[cfg(feature = "markdown")]
    #[error("missing frontmatter in {0}")]
    MissingFrontmatter(PathBuf),
    #[cfg(feature = "markdown")]
    #[error("failed to deserialize frontmatter for {0}: {1}")]
    DeserializeFrontmatter(PathBuf, serde_json::error::Error),
    #[cfg(feature = "markdown")]
    #[error("no file stem for: {0}")]
    NoFileStem(PathBuf),
    // sass
    #[cfg(feature = "sass")]
    #[error("failed to compile sass: {0}")]
    SassCompile(#[from] Box<grass::Error>),
}

/// Write contents to a file. The biggest difference to [`fs::write`] is that this
/// creates subdirectories as necessary.
pub fn write(contents: impl Into<String>, to: impl AsRef<Path>) -> Result<(), Error> {
    // Create directory tree
    if let Some(parent) = to.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(to.as_ref(), contents.into())?;
    Ok(())
}

/// Get a list of paths that match the given glob.
///
/// # Examples
///
/// ```
/// let glob = pichu::glob("content/blog/*.md")?;
/// ```
pub fn glob(glob: impl AsRef<str>) -> Result<Glob, Error> {
    let paths = glob::glob(glob.as_ref())?
        .into_iter()
        .collect::<Result<Vec<PathBuf>, glob::GlobError>>()?;
    Ok(Glob { paths })
}

/// A list of paths, probably created by [`glob`].
#[derive(Debug)]
pub struct Glob {
    paths: Vec<PathBuf>,
}

impl Glob {
    /// Parse the files in parallel using the provided parse_fn.
    pub fn parse<T: Send + Sync>(
        self,
        parse_fn: impl Fn(PathBuf) -> Result<T, Error> + Send + Sync,
    ) -> Result<Parsed<T>, Error> {
        let inner = self
            .paths
            .into_par_iter()
            .map(|path| parse_fn(path))
            .collect::<Result<Vec<T>, Error>>()?;
        Ok(Parsed { items: inner })
    }
}

/// Parsed is a list of parsed items, ready to be sorted and rendered.
#[derive(Debug, Clone)]
pub struct Parsed<T: Send + Sync> {
    items: Vec<T>,
}

impl<T: Send + Sync> Parsed<T> {
    /// Sort the items by the key provided, ascending.
    pub fn sort_by_key<K, F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.items.sort_by_key(f);
        self
    }

    /// Sort the items by the key provided, descending.
    pub fn sort_by_key_reverse<K, F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.items.sort_by_key(f);
        self.items.reverse();
        self
    }

    /// Render individual items in parallel using the provided render function.
    pub fn render_each<
        P: AsRef<Path>,
        S: Into<String> + Send,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    >(
        self,
        render_fn: impl Fn(&T) -> Result<S, E> + Send + Sync,
        build_path_fn: impl Fn(&T) -> P + Send + Sync,
    ) -> Result<Self, Error> {
        self.items
            .par_iter()
            .map(|item| {
                let content = render_fn(&item)?;
                Ok((item, content))
            })
            .collect::<Result<Vec<_>, E>>()
            .map_err(|e| Error::RenderFn(e.into()))?
            .into_par_iter()
            .map(|(item, content)| write(content.into(), build_path_fn(&item)))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(self)
    }

    /// Render all items into a single destination.
    pub fn render_all<S: Into<String>, E: Into<Box<dyn std::error::Error + Send + Sync>>>(
        self,
        render_fn: impl Fn(&Vec<T>) -> Result<S, E>,
        dest_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let content = render_fn(&self.items).map_err(|e| Error::RenderFn(e.into()))?;
        write(content.into(), dest_path)?;
        Ok(self)
    }

    /// Extract the underlying `Vec<T>` for further processing.
    pub fn into_vec(self) -> Vec<T> {
        self.items
    }

    /// Return a reference to the first item, or `None` if empty.
    pub fn first(&self) -> Option<&T> {
        self.items.first()
    }
}
