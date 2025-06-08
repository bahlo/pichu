use comrak::{markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter};
use gray_matter::{engine::YAML, Matter};
use serde::de::DeserializeOwned;
use std::{
    cell::LazyCell,
    fmt,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use crate::{Error, Glob, Parsed};

#[derive(thiserror::Error, Debug)]
pub enum MarkdownError {
    #[error("io error: {0}")]
    IO(#[from] io::Error),
    #[error("missing frontmatter in {0}")]
    MissingFrontmatter(PathBuf),
    #[cfg(feature = "markdown")]
    #[error("failed to deserialize frontmatter for {0}: {1}")]
    DeserializeFrontmatter(PathBuf, serde_json::error::Error),
    #[cfg(feature = "markdown")]
    #[error("no file stem for: {0}")]
    NoFileStem(PathBuf),
}

impl From<MarkdownError> for Box<dyn std::error::Error + Send> {
    fn from(err: MarkdownError) -> Self {
        Box::new(err)
    }
}

/// SyntectAdapter::new loads a few binary files from disk, better to do this only once.
const SYNTECT_ADAPTER: LazyCell<SyntectAdapter> = LazyCell::new(|| SyntectAdapter::new(None));

/// A parsed markdown file.
#[derive(Debug, Clone)]
pub struct Markdown<T> {
    pub frontmatter: T,
    pub basename: String,
    pub markdown: String,
    pub html: String,
}

impl Glob {
    /// Parse the paths as Markdown files.
    /// You are encouraged to copy-paste this function into your codebase to
    /// adapt it to your needs, if required.
    #[cfg(feature = "markdown")]
    pub fn parse_markdown<T: DeserializeOwned + fmt::Debug + Send + Sync>(
        self,
    ) -> Result<Parsed<Markdown<T>>, Error> {
        self.try_parse::<Markdown<T>, MarkdownError>(parse_markdown)
    }
}

pub fn parse_markdown<T: DeserializeOwned>(path: PathBuf) -> Result<Markdown<T>, MarkdownError> {
    let mut file = File::open(&path).map_err(MarkdownError::IO)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(MarkdownError::IO)?;

    let matter = Matter::<YAML>::new();
    let markdown = matter.parse(&contents);
    let frontmatter: T = markdown
        .data
        .ok_or(MarkdownError::MissingFrontmatter(path.clone()))?
        .deserialize()
        .map_err(|e| MarkdownError::DeserializeFrontmatter(path.clone(), e))?;

    let syntect_adapter = &*SYNTECT_ADAPTER;
    let markdown_context = MarkdownContext::new(syntect_adapter);
    let html = markdown_to_html_with_plugins(
        &markdown.content,
        &markdown_context.options,
        &markdown_context.plugins,
    );

    let basename = path
        .file_stem()
        .ok_or_else(|| MarkdownError::NoFileStem(path.clone()))?
        .to_string_lossy()
        .to_string();

    Ok(Markdown {
        frontmatter,
        basename,
        markdown: markdown.content,
        html,
    })
}

pub struct MarkdownContext<'a> {
    plugins: comrak::Plugins<'a>,
    options: comrak::Options<'a>,
}

impl<'a> MarkdownContext<'a> {
    fn new(syntect_adapter: &'a SyntectAdapter) -> Self {
        let render = comrak::RenderOptions {
            unsafe_: true,
            ..Default::default()
        };
        let extension = comrak::ExtensionOptions {
            strikethrough: true,
            tagfilter: true,
            table: true,
            superscript: true,
            header_ids: Some("".to_string()),
            footnotes: true,
            description_lists: true,
            ..Default::default()
        };
        let parse = comrak::ParseOptions {
            smart: true,
            ..Default::default()
        };
        let options = comrak::Options {
            render,
            extension,
            parse,
        };
        let render_plugins = comrak::RenderPlugins {
            codefence_syntax_highlighter: Some(syntect_adapter),
            ..Default::default()
        };
        let plugins = comrak::Plugins {
            render: render_plugins,
        };

        Self { plugins, options }
    }
}
