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

pub fn glob(glob: impl AsRef<str>) -> Result<Glob, Error> {
    let paths = glob::glob(glob.as_ref())?
        .into_iter()
        .collect::<Result<Vec<PathBuf>, glob::GlobError>>()?;
    Ok(Glob { paths })
}

pub fn write(contents: impl Into<String>, to: impl AsRef<Path>) -> Result<(), Error> {
    // Create directory tree
    if let Some(parent) = to.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    dbg!(to.as_ref());
    fs::write(to.as_ref(), contents.into())?;
    Ok(())
}

#[derive(Debug)]
pub struct Glob {
    paths: Vec<PathBuf>,
}

impl Glob {
    pub fn parse<T: Send + Sync>(
        self,
        parse_fn: impl Fn(PathBuf) -> Result<T, Error>,
    ) -> Result<Parsed<T>, Error> {
        let inner = self
            .paths
            .into_iter()
            .map(|path| parse_fn(path))
            .collect::<Result<Vec<T>, Error>>()?;
        Ok(Parsed { items: inner })
    }
}

#[derive(Debug, Clone)]
pub struct Parsed<T: Send + Sync> {
    items: Vec<T>,
}

impl<T: Send + Sync> Parsed<T> {
    pub fn sort_by_key<K, F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.items.sort_by_key(f);
        self
    }

    pub fn sort_by_key_reverse<K, F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) -> K,
        K: Ord,
    {
        self.items.sort_by_key(f);
        self.items.reverse();
        self
    }

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

    pub fn render_all<S: Into<String>, E: Into<Box<dyn std::error::Error + Send + Sync>>>(
        self,
        render_fn: impl Fn(&Vec<T>) -> Result<S, E>,
        dest_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let content = render_fn(&self.items).map_err(|e| Error::RenderFn(e.into()))?;
        write(content.into(), dest_path)?;
        Ok(self)
    }

    pub fn into_vec(self) -> Vec<T> {
        self.items
    }

    pub fn first(&self) -> Option<&T> {
        self.items.first()
    }
}
