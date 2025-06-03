use comrak::{markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter};
use gray_matter::{engine::YAML, Matter};
use serde::de::DeserializeOwned;
use std::{fmt, fs::File, io::Read};

use crate::{Error, Glob, Parsed};

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
        let syntect_adapter = SyntectAdapter::new(None);
        let markdown_context = MarkdownContext::new(&syntect_adapter);
        let matter = Matter::<YAML>::new();

        self.parse::<Markdown<T>>(|path| {
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let markdown = matter.parse(&contents);
            let frontmatter: T = markdown
                .data
                .ok_or(Error::MissingFrontmatter(path.clone()))?
                .deserialize()
                .map_err(|e| Error::DeserializeFrontmatter(path.clone(), e))?;

            let html = markdown_to_html_with_plugins(
                &markdown.content,
                &markdown_context.options,
                &markdown_context.plugins,
            );

            let basename = path
                .file_stem()
                .ok_or_else(|| Error::NoFileStem(path.clone()))?
                .to_string_lossy()
                .to_string();

            Ok(Markdown {
                frontmatter,
                basename,
                markdown: markdown.content,
                html,
            })
        })
    }
}

struct MarkdownContext<'a> {
    plugins: comrak::Plugins<'a>,
    options: comrak::Options<'a>,
}

impl<'a> MarkdownContext<'a> {
    fn new(syntect_adapter: &'a SyntectAdapter) -> Self {
        let mut render = comrak::RenderOptions::default();
        render.unsafe_ = true;
        let mut extension = comrak::ExtensionOptions::default();
        extension.strikethrough = true;
        extension.tagfilter = true;
        extension.table = true;
        extension.superscript = true;
        extension.header_ids = Some("".to_string());
        extension.footnotes = true;
        extension.description_lists = true;
        let mut parse = comrak::ParseOptions::default();
        parse.smart = true;
        let options = comrak::Options {
            render,
            extension,
            parse,
        };
        let mut render_plugins = comrak::RenderPlugins::default();
        render_plugins.codefence_syntax_highlighter = Some(syntect_adapter);
        let mut plugins = comrak::Plugins::default();
        plugins.render = render_plugins;

        Self { plugins, options }
    }
}
