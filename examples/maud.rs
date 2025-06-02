use maud::{html, Markup, PreEscaped};
use pichu::Markdown;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Blogpost {
    title: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _blog = pichu::glob("examples/content/blog/*.md")?
        .parse_markdown::<Blogpost>()?
        .render_each(render_blogpost, |post| {
            format!("examples/dist/maud/{}/index.html", post.basename)
        })?;

    Ok(())
}

fn render_blogpost(
    post: &Markdown<Blogpost>,
) -> Result<Markup, Box<dyn std::error::Error + Send + Sync>> {
    Ok(html! {
        h1 { (post.frontmatter.title) }
        article {
            (PreEscaped(post.html.clone()))
        }
    })
}
