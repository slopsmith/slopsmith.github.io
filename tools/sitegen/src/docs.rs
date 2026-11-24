use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Context, bail};
use regex::{Captures, Regex};

const SECTION: &str = r#"+++
title = "Docs"
template = "ps5slop/docs.html"
page_template = "ps5slop/doc.html"
sort_by = "title"
+++
"#;

static LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(([^)\s]+)\)").unwrap());

/// Writes each markdown file in `src` to `out` as a Zola page, replacing `out`.
pub fn run(src: &Path, out: &Path) -> anyhow::Result<()> {
    let names: BTreeSet<String> = fs::read_dir(src)
        .with_context(|| format!("reading {}", src.display()))?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.ends_with(".md"))
        .collect();
    if names.is_empty() {
        bail!("no docs in {}", src.display());
    }

    if out.exists() {
        fs::remove_dir_all(out)?;
    }
    fs::create_dir_all(out)?;
    fs::write(out.join("_index.md"), SECTION)?;

    for name in &names {
        let text = fs::read_to_string(src.join(name))?;
        let (title, body) = split_title(&text).with_context(|| format!("{name} has no `# ` title"))?;
        let body = rewrite_links(body, &names, name);
        let page = format!("+++\ntitle = {}\n+++\n\n{}", toml_string(title), body.trim_start());
        fs::write(out.join(name), page)?;
    }
    eprintln!("docs: {} pages from {}", names.len(), src.display());
    Ok(())
}

fn split_title(text: &str) -> Option<(&str, &str)> {
    let start = text.find("# ")?;
    if start != 0 && !text[..start].ends_with('\n') {
        return None;
    }
    let rest = &text[start + 2..];
    let end = rest.find('\n').unwrap_or(rest.len());
    Some((rest[..end].trim(), &rest[end..]))
}

/// Points links between docs at Zola pages. Links that leave the user docs
/// are turned into plain text with a warning, since their targets are not published.
fn rewrite_links(body: &str, names: &BTreeSet<String>, file: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut fenced = false;
    for line in body.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        if fenced {
            out.push_str(line);
            continue;
        }
        let line = LINK.replace_all(line, |c: &Captures| {
            let (text, target) = (&c[1], &c[2]);
            match resolve(target, names) {
                Some(url) => format!("[{text}]({url})"),
                None => {
                    eprintln!("docs: warning: {file}: dropped link to {target}");
                    text.to_string()
                }
            }
        });
        out.push_str(&line);
    }
    out
}

fn resolve(target: &str, names: &BTreeSet<String>) -> Option<String> {
    if target.starts_with('#') || target.contains("://") || target.starts_with("mailto:") {
        return Some(target.to_string());
    }
    let (path, anchor) = match target.split_once('#') {
        Some((p, a)) => (p, format!("#{a}")),
        None => (target, String::new()),
    };
    if names.contains(path) {
        return Some(format!("@/ps5slop/docs/{path}{anchor}"));
    }
    if path == "../../COMPATIBILITY.md" {
        return Some("@/ps5slop/compat/_index.md".to_string());
    }
    None
}

fn toml_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> BTreeSet<String> {
        ["running.md", "app.md"].map(String::from).into()
    }

    #[test]
    fn links_between_docs_point_at_zola_pages() {
        let out = rewrite_links("See [app](app.md#views).\n", &names(), "x.md");
        assert_eq!(out, "See [app](@/ps5slop/docs/app.md#views).\n");
    }

    #[test]
    fn links_to_unpublished_docs_become_text() {
        let out = rewrite_links("See [gpu.md](../gpu.md).\n", &names(), "x.md");
        assert_eq!(out, "See gpu.md.\n");
    }

    #[test]
    fn code_blocks_are_left_alone() {
        let src = "```\n[a](../gpu.md)\n```\n";
        assert_eq!(rewrite_links(src, &names(), "x.md"), src);
    }

    #[test]
    fn title_is_taken_from_the_first_heading() {
        assert_eq!(split_title("# Running games\n\nBody"), Some(("Running games", "\n\nBody")));
    }
}
