//! Writes the documentation site's color tokens to
//! `website/src/styles/tokens.css` (`mise run doc:tokens`). Fails when the
//! light theme misses WCAG AA.

use std::path::PathBuf;

fn main() {
    let failures = tp_ui::web::light_failures();
    if !failures.is_empty() {
        eprintln!("light theme contrast failures:\n{}", failures.join("\n"));
        std::process::exit(1);
    }
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../website/src/styles/tokens.css");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, tp_ui::web::tokens_css()).unwrap();
    println!("wrote {}", path.display());
}
