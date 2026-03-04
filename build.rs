use std::path::Path;

const STORAGE_UTILS: &str = include_str!("src/storage-utils.js");

fn main() {
    let out = std::env::var("OUT_DIR").unwrap();
    let out = Path::new(&out);

    println!("cargo:rerun-if-changed=src/storage-utils.js");
    println!("cargo:rerun-if-changed=src/decrypt.html");
    minify_html(
        &inline_storage_utils("src/decrypt.html"),
        &out.join("decrypt.minify.html"),
    );

    println!("cargo:rerun-if-changed=src/decrypt.js");
    minify_js(
        &inline_storage_utils("src/decrypt.js"),
        &out.join("decrypt.minify.js"),
    );
}

fn minify_html(content: &[u8], output_path: &Path) {
    let mut cfg = minify_html::Cfg::new();
    cfg.preserve_brace_template_syntax = true;
    cfg.minify_js = true;

    let html = minify_html::minify(content, &cfg);
    std::fs::write(output_path, html).expect("Failed to write minified HTML template");
}

fn minify_js(content: &[u8], output_path: &Path) {
    let session = minify_js::Session::new();
    let mut code_minified = Vec::new();
    minify_js::minify(
        &session,
        minify_js::TopLevelMode::Global,
        content,
        &mut code_minified,
    )
    .expect("Failed to minify JS template");
    std::fs::write(output_path, code_minified).expect("Failed to write minified JS template");
}

fn inline_storage_utils(input_path: impl AsRef<Path>) -> Vec<u8> {
    let content = std::fs::read_to_string(input_path).expect("Failed to read template");
    let inlined = content.replace("{{ storage-utils }}", STORAGE_UTILS);
    inlined.into_bytes()
}
