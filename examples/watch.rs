use pichu::Markdown;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Blogpost {
    title: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pichu::watch("examples/content", |paths| {
        println!("Paths changed: {:?}", paths);
        if let Err(e) = build() {
            eprintln!("Build error: {}", e);
        }
    })?;
    Ok(())
}

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let _blog = pichu::glob("examples/content/blog/*.md")?
        .parse_markdown::<Blogpost>()?
        .render_each(render_blogpost, |post| {
            format!("examples/dist/maud/{}/index.html", post.basename)
        })?;

    Ok(())
}

fn render_blogpost(post: &Markdown<Blogpost>) -> String {
    // Check out other examples for more advanced templating
    format!("<h1>{}</h1>{}", post.frontmatter.title, post.html)
}
