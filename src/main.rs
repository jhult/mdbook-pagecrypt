const OUTPUT_PAGECRYPT: &str = "output.pagecrypt";

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use mdbook_html::HtmlHandlebars;
use mdbook_pagecrypt::PageCrypt;
use mdbook_renderer::book::BookItem;
use mdbook_renderer::{RenderContext, Renderer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PageCryptConfig {
    pub password: String,
    #[serde(default = "default_rounds")]
    pub rounds: u32,
}

fn default_rounds() -> u32 {
    600_000
}

fn main() -> Result<()> {
    let mut stdin = io::stdin();
    let mut ctx =
        RenderContext::from_json(&mut stdin).with_context(|| "Failed to read JSON from stdin")?;

    let cfg_value: Value = ctx
        .config
        .get(OUTPUT_PAGECRYPT)
        .with_context(|| format!("Failed to parse config entry '{}'", OUTPUT_PAGECRYPT))?
        .context(format!("password is required in [{}]", OUTPUT_PAGECRYPT))?;

    let cfg: PageCryptConfig = serde_json::from_value(cfg_value.clone())
        .with_context(|| format!("Failed to deserialize pagecrypt config: {:?}", cfg_value))?;

    if cfg.password.is_empty() {
        anyhow::bail!("password is required in [{}]", OUTPUT_PAGECRYPT);
    }

    // Check for conflicting HTML config: if pagecrypt has HTML options,
    // [output.html] should not exist in book.toml
    let pagecrypt_has_html_config = if let Value::Object(ref map) = cfg_value {
        map.keys().any(|k| k != "password" && k != "rounds")
    } else {
        false
    };

    if pagecrypt_has_html_config {
        let book_toml_path = ctx.root.join("book.toml");
        if let Ok(content) = fs::read_to_string(&book_toml_path) {
            // Check for [output.html] section in book.toml
            if content.contains("[output.html]") {
                anyhow::bail!(
                    "[output.html] section found in book.toml. \
                     Remove it and place HTML options under [output.pagecrypt] instead."
                );
            }
        }
    }

    copy_html_config(&mut ctx)?;

    let no_html_extension: bool = ctx
        .config
        .get("output.pagecrypt.no-html-extension")?
        .unwrap_or(false);

    let pagecrypt = PageCrypt::builder()
        .password(cfg.password)
        .rounds(cfg.rounds)
        .build()?;

    let dest = ctx.destination.clone();
    fs::create_dir_all(&dest).with_context(|| format!("Failed to create {:?}", dest))?;

    // Render HTML with default template
    HtmlHandlebars::new().render(&ctx)?;

    // Patch up the HTML with password protection
    for item in ctx.book.iter() {
        if let BookItem::Chapter(ref ch) = *item {
            if ch.is_draft_chapter() {
                continue;
            }

            // Get the path to the HTML file
            let path = ch.path.as_ref().unwrap();
            let ctx_path = path
                .to_str()
                .with_context(|| "Could not convert path to str")?;
            let logical_path = Path::new(ctx_path).with_extension("html");
            let filepath = if no_html_extension {
                // Transform: chapter.html -> chapter/index.html
                // Index pages stay the same: index.html -> index.html
                dest.join(clean_url_output_path(&logical_path))
            } else {
                dest.join(logical_path)
            };

            // Encrypt the HTML
            let file = fs::read(&filepath).with_context(|| "Failed to read HTML file")?;
            let file = pagecrypt.encrypt_html(&file)?;
            fs::write(&filepath, file).with_context(|| "Failed to write HTML file")?;
        }
    }

    // Encrypt the index.html
    let index_path = dest.join("index.html");
    if index_path.exists() {
        let index = fs::read(&index_path)?;
        let index = pagecrypt.encrypt_html(&index)?;
        fs::write(&index_path, index).with_context(|| "Failed to write index.html")?;
    }

    // Encrypt the print.html (or _print/index.html with clean URLs)
    let print_path = if no_html_extension {
        dest.join("_print/index.html")
    } else {
        dest.join("print.html")
    };
    if print_path.exists() {
        let print_content = fs::read(&print_path)?;
        let print_content = pagecrypt.encrypt_html(&print_content)?;
        fs::write(&print_path, print_content).with_context(|| "Failed to write print.html")?;
    }

    // Encrypt the toc.html (or _toc/index.html with clean URLs)
    let toc_path = if no_html_extension {
        dest.join("_toc/index.html")
    } else {
        dest.join("toc.html")
    };
    if toc_path.exists() {
        let toc_content = fs::read(&toc_path)?;
        let toc_content = pagecrypt.encrypt_html(&toc_content)?;
        fs::write(&toc_path, toc_content).with_context(|| "Failed to write toc.html")?;
    }

    // Remove searchindex.json (legacy format, no longer used by mdBook)
    let searchindex_json = dest.join("searchindex.json");
    if searchindex_json.exists() {
        fs::remove_file(&searchindex_json).with_context(|| "Failed to remove search index")?;
    }

    // Encrypt the search index
    let searchindex_js = dest.join("searchindex.js");
    if searchindex_js.exists() {
        let index = fs::read(&searchindex_js)?;
        let index = pagecrypt.encrypt_js(&index)?;
        fs::write(&searchindex_js, index).with_context(|| "Failed to write search index")?;
    }

    Ok(())
}

fn copy_html_config(ctx: &mut RenderContext) -> Result<()> {
    let pagecrypt_config: Option<Value> = ctx
        .config
        .get(OUTPUT_PAGECRYPT)
        .with_context(|| "Failed to read pagecrypt config")?;

    if let Some(Value::Object(map)) = pagecrypt_config {
        for (key, value) in map {
            if key != "password" && key != "rounds" {
                ctx.config
                    .set(&format!("output.html.{}", key), value)
                    .with_context(|| format!("Failed to set output.html.{}", key))?;
            }
        }
    }
    Ok(())
}

/// Converts a logical HTML path to the physical output path in clean URL mode.
///
/// Index pages are left unchanged. Non-index pages are moved into a
/// subdirectory so the URL has no extension.
///
/// # Examples
///
/// - `foo/bar.html` -> `foo/bar/index.html`
/// - `foo/index.html` -> `foo/index.html` (unchanged)
/// - `index.html` -> `index.html` (unchanged)
fn clean_url_output_path(logical_html_path: &Path) -> std::path::PathBuf {
    let is_index = logical_html_path.file_stem() == Some(OsStr::new("index"));

    if is_index {
        logical_html_path.to_path_buf()
    } else {
        logical_html_path.with_extension("").join("index.html")
    }
}
