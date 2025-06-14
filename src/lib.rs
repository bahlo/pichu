//! Pichu is the static site generator designed to evolve with your needs.
//!
//! # Example
//!
//! ```
//! use serde::Deserialize;
//! use pichu::Markdown;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     pichu::glob("content/blog/*.md")?
//!         .parse_markdown::<Blogpost>()?
//!         .render_each(render_blog_post, |post| format!("dist/blog/{}/index.html", post.basename))?
//!         .render_all(render_blog, "dist/blog/index.html")?;
//!     Ok(())
//! }
//!
//! #[derive(Debug, Deserialize)]
//! struct Blogpost {
//!     title: String,
//! }
//!
//! fn render_blog_post(post: &Markdown<Blogpost>) -> String {
//!     format!("<h1>{}</h1>{}", post.frontmatter.title, post.html)
//! }
//!
//! fn render_blog(posts: &Vec<Markdown<Blogpost>>) -> String {
//!     format!("{} posts", posts.len())
//! }
//! ```

#![deny(warnings)]
#![deny(clippy::pedantic, clippy::unwrap_used)]
#![deny(missing_docs)]

use rayon::prelude::*;
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[cfg(feature = "markdown")]
mod markdown;
#[cfg(feature = "markdown")]
pub use markdown::{parse_markdown, Markdown, MarkdownError};

#[cfg(feature = "sass")]
mod sass;
#[cfg(feature = "sass")]
pub use sass::{render_sass, SassError};

#[cfg(feature = "watch")]
mod watch;
#[cfg(feature = "watch")]
pub use watch::watch;

/// The error type returned in this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O error.
    #[error("io error: {0}")]
    IO(#[from] io::Error),
    /// Invalid glob pattern.
    #[error("{0}")]
    GlobPatternError(#[from] glob::PatternError),
    /// Failed to process glob result.
    #[error("{0}")]
    GlobError(#[from] glob::GlobError),
    /// Error occurred while parsing.
    #[error("parse error: {0:?}")]
    Parse(Box<dyn fmt::Debug + Send + Sync>),
    /// Error occurred during render.
    #[error("render error: {0:?}")]
    Render(Box<dyn fmt::Debug + Send + Sync>),
    /// File already exists at the destination path.
    #[error("file exists: {0}")]
    FileExists(PathBuf),
}

/// Like [`fs::write`], but creates directories as necessary.
///
/// # Errors
///
/// Returns an error if the parent directory cannot be created or if the file cannot be written.
pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<(), io::Error> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

/// Copy the contents of a directory into another, recursively.
/// Skips files starting with a `.`, except `.well-known`.
///
/// # Errors
///
/// Returns an error if directories cannot be created, files cannot be copied, or if a file already exists at the destination.
pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), Error> {
    fs::create_dir_all(to.as_ref())?;
    fs::read_dir(from.as_ref())?
        .map(|entry| {
            let entry = entry?;
            let file_name = entry.file_name();

            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with('.') && file_name_str != ".well-known" {
                return Ok(());
            }

            let new_path = to.as_ref().join(file_name);
            if entry.path().is_dir() {
                fs::create_dir(&new_path)?;
                copy_dir(entry.path(), &new_path)?;
            } else {
                if new_path.exists() {
                    return Err(Error::FileExists(new_path));
                }

                let path = entry.path();
                fs::copy(path, new_path)?;
            }

            Ok(())
        })
        .collect::<Result<Vec<()>, Error>>()?;
    Ok(())
}

/// Get a list of paths that match the given glob.
///
/// # Errors
///
/// Returns an error if the glob pattern is invalid or if there are filesystem access issues.
pub fn glob(glob: impl AsRef<str>) -> Result<Glob, Error> {
    let paths = glob::glob(glob.as_ref())?.collect::<Result<Vec<PathBuf>, glob::GlobError>>()?;
    Ok(Glob { paths })
}

/// A list of paths, probably created by [`glob`].
#[derive(Debug)]
pub struct Glob {
    paths: Vec<PathBuf>,
}

impl Glob {
    /// Parse the files using the provided parse function.
    pub fn parse<T: Send + Sync>(self, parse_fn: impl Fn(PathBuf) -> T + Send + Sync) -> Parsed<T> {
        let items = self.paths.into_par_iter().map(parse_fn).collect::<Vec<T>>();
        Parsed { items }
    }

    /// Parse the files in parallel using the provided `parse_fn`.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the files fail to parse.
    pub fn try_parse<T: Send + Sync, E: fmt::Debug + Send + Sync + 'static>(
        self,
        parse_fn: impl Fn(&PathBuf) -> Result<T, E> + Send + Sync,
    ) -> Result<Parsed<T>, Error> {
        let items = self
            .paths
            .par_iter()
            .map(parse_fn)
            .collect::<Result<Vec<T>, E>>()
            .map_err(|e| Error::Parse(Box::new(e)))?;
        Ok(Parsed { items })
    }
}

/// Parsed is a list of parsed items, ready to be sorted and rendered.
#[derive(Debug, Clone)]
pub struct Parsed<T: Send + Sync> {
    items: Vec<T>,
}

impl<T: Send + Sync> Parsed<T> {
    /// Sort the items by the key provided, ascending.
    #[must_use]
    pub fn sort_by_key<K, F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> K + Sync,
        K: Ord,
    {
        self.items.par_sort_by_key(f);
        self
    }

    /// Sort the items by the key provided, descending.
    #[must_use]
    pub fn sort_by_key_reverse<K, F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> K + Sync,
        K: Ord,
    {
        self.items.par_sort_by_key(f);
        self.items.reverse();
        self
    }

    /// Render individual items in parallel using the provided render function.
    ///
    /// # Errors
    ///
    /// Returns an error if any file cannot be written to the filesystem.
    pub fn render_each<P: AsRef<Path>, S: Into<String> + Send>(
        self,
        render_fn: impl Fn(&T) -> S + Send + Sync,
        build_path_fn: impl Fn(&T) -> P + Send + Sync,
    ) -> Result<Self, Error> {
        self.items
            .par_iter()
            .map(|item| {
                let content = render_fn(item);
                (item, content)
            })
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(item, content)| write(build_path_fn(item), content.into()).map_err(Error::IO))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(self)
    }

    /// Render individual items in parallel using the provided render function.
    ///
    /// # Errors
    ///
    /// Returns an error if the render function fails for any item or if any file cannot be written.
    pub fn try_render_each<
        P: AsRef<Path>,
        S: Into<String> + Send,
        E: fmt::Debug + Send + Sync + 'static,
    >(
        self,
        render_fn: impl Fn(&T) -> Result<S, E> + Send + Sync,
        build_path_fn: impl Fn(&T) -> P + Send + Sync,
    ) -> Result<Self, Error> {
        self.items
            .par_iter()
            .map(|item| {
                let content = render_fn(item)?;
                Ok((item, content))
            })
            .collect::<Result<Vec<_>, E>>()
            .map_err(|e| Error::Render(Box::new(e)))?
            .into_par_iter()
            .map(|(item, content)| write(build_path_fn(item), content.into()).map_err(Error::IO))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(self)
    }

    /// Render all items into a single destination.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written to the filesystem.
    pub fn render_all<S: Into<String>>(
        self,
        render_fn: impl Fn(&Vec<T>) -> S,
        dest_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let content = render_fn(&self.items);
        write(dest_path, content.into())?;
        Ok(self)
    }

    /// Render all items into a single destination.
    ///
    /// # Errors
    ///
    /// Returns an error if the render function fails or if the file cannot be written.
    pub fn try_render_all<S: Into<String>, E: fmt::Debug + Send + Sync + 'static>(
        self,
        render_fn: impl Fn(&Vec<T>) -> Result<S, E>,
        dest_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let content = render_fn(&self.items).map_err(|e| Error::Render(Box::new(e)))?;
        write(dest_path, content.into())?;
        Ok(self)
    }

    /// Extract the underlying `Vec<T>` for further processing.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.items
    }

    /// Return a reference to the first item, or `None` if empty.
    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.items.first()
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    use std::{env, fs};

    #[derive(Deserialize, Debug)]
    struct Blog {
        title: String,
    }

    #[test]
    fn test_write() -> Result<(), Box<dyn std::error::Error>> {
        let dir = env::temp_dir().join("pichu_test_write");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }

        // Write file to a non-existing subdirectory
        let content = "foo bar";
        let filepath = dir.join("foo/bar/baz.txt");
        write(&filepath, content)?;

        // Ensure the file exists and directories have been created
        assert_eq!(fs::read_to_string(filepath)?, content);

        fs::remove_dir_all(&dir)?;

        Ok(())
    }

    #[test]
    fn test_copy_dir() -> Result<(), Box<dyn std::error::Error>> {
        let dir = env::temp_dir().join("pichu_test_copy_dir");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }

        copy_dir("examples", &dir)?;

        assert!(dir.join("maud.rs").exists());
        assert!(dir.join("content/about.md").exists());
        assert!(dir.join("content/blog/hello-world.md").exists());

        fs::remove_dir_all(&dir)?;

        Ok(())
    }

    #[test]
    fn test_glob() -> Result<(), Box<dyn std::error::Error>> {
        let paths = glob("examples/content/**/*.md")?.paths;
        assert_eq!(paths.len(), 2);
        assert_eq!(
            paths[0].to_str().expect("Path should be valid UTF-8"),
            "examples/content/about.md"
        );
        assert_eq!(
            paths[1].to_str().expect("Path should be valid UTF-8"),
            "examples/content/blog/hello-world.md"
        );

        let paths = glob("examples/content/*.md")?.paths;
        assert_eq!(paths.len(), 1);
        assert_eq!(
            paths[0].to_str().expect("Path should be valid UTF-8"),
            "examples/content/about.md"
        );

        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = glob("examples/content/blog/*.md")?
            .parse::<Blog>(|_path| Blog {
                title: "foo".to_string(),
            })
            .into_vec();
        assert_eq!(&parsed[0].title, "foo");

        Ok(())
    }

    #[test]
    fn test_try_parse() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct File {
            contents: String,
        }

        let parsed = glob("examples/content/blog/*.md")?
            .try_parse::<File, io::Error>(|path| {
                let contents = fs::read_to_string(path)?;
                Ok(File { contents })
            })?
            .into_vec();
        assert!(!parsed[0].contents.is_empty());

        Ok(())
    }

    #[test]
    fn test_render_each_all() -> Result<(), Box<dyn std::error::Error>> {
        let dir = env::temp_dir().join("pichu_test_render_each_all");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }

        glob("examples/content/blog/*.md")?
            .parse_markdown::<Blog>()?
            .render_each(
                |blog| format!("<h1>{}</h1>", blog.frontmatter.title),
                |blog| dir.join(format!("blog/{}.html", blog.basename)),
            )?
            .render_all(
                |posts| format!("<p>Number of posts: {}</p>", posts.len()),
                dir.join("blog.html"),
            )?;

        assert_eq!(
            fs::read_to_string(dir.join("blog/hello-world.html"))?,
            "<h1>Hello, world</h1>".to_string()
        );
        assert_eq!(
            fs::read_to_string(dir.join("blog.html"))?,
            "<p>Number of posts: 1</p>".to_string()
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn test_try_render_each_all() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(thiserror::Error, Debug)]
        enum RenderError {
            #[error("Missing frontmatter")]
            MissingFrontmatter,
            #[error("No posts")]
            NoPosts,
        }

        let dir = env::temp_dir().join("pichu_test_try_render_each_all");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }

        glob("examples/content/blog/*.md")?
            .parse_markdown::<Blog>()?
            .try_render_each(
                |blog| {
                    if blog.frontmatter.title.is_empty() {
                        return Err(RenderError::MissingFrontmatter);
                    }
                    Ok(format!("<h1>{}</h1>", blog.frontmatter.title))
                },
                |blog| dir.join(format!("blog/{}.html", blog.basename)),
            )?
            .try_render_all(
                |posts| {
                    if posts.is_empty() {
                        return Err(RenderError::NoPosts);
                    }
                    Ok(format!("<p>Number of posts: {}</p>", posts.len()))
                },
                dir.join("blog.html"),
            )?;

        assert_eq!(
            fs::read_to_string(dir.join("blog/hello-world.html"))?,
            "<h1>Hello, world</h1>".to_string()
        );
        assert_eq!(
            fs::read_to_string(dir.join("blog.html"))?,
            "<p>Number of posts: 1</p>".to_string()
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }
}
