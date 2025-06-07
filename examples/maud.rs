use maud::{html, PreEscaped};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Blogpost {
    title: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _blog = pichu::glob("examples/content/blog/*.md")?
        .parse_markdown::<Blogpost>()?
        .render_each(
            |post| {
                html! {
                    h1 { (post.frontmatter.title) }
                    article {
                        (PreEscaped(post.html.clone()))
                    }
                }
            },
            |post| format!("examples/dist/maud/{}/index.html", post.basename),
        )?;

    Ok(())
}
