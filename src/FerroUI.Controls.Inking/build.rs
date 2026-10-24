//! Compiles the theme documents of the crate (`Themes/WinUI.xaml` and the resources and
//! control themes of the toolbar under `Themes/WinUI/`) and lists them as the assets a build
//! with the feature `document-assets` embeds (docs/porting/xaml.md, 9.5.13, 9.5.20 and 9.6).
//!
//! The documents are compiled as one group to `$OUT_DIR/xaml/compiled_xaml.rs` against the
//! type models: the one of this crate, scanned from its sources, and the ones of the crates it
//! is built on (`DEP_<CRATE>_XAML_XAMLMETA`, the WinUI theme among them, whose high-contrast
//! variant the resources name). The build also writes the `.xamlmeta` of the crate, which
//! Cargo hands to the build scripts of the crates that depend on this one
//! (`DEP_FERROUI_CONTROLS_INKING_XAML_XAMLMETA`): an application whose documents name the
//! types of the crate and include its styles.
//!
//! As upstream's compiler removes a compiled resource, a compiled document is not an asset;
//! with the feature `document-assets`, and in the tests of the crate, the documents are
//! embedded as assets too (`$OUT_DIR/assets.rs`), for a document the run-time loader loads
//! that includes them.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use ferroui_build::{Build, TypeSystem, XamlGroup};

fn collect(root: &Path, directory: &Path, found: &mut Vec<(String, PathBuf)>) {
    println!("cargo::rerun-if-changed={}", directory.display());
    let mut entries: Vec<PathBuf> = fs::read_dir(directory)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", directory.display()))
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    entries.sort();

    for path in entries {
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        if path.is_dir() {
            if !name.starts_with('.') {
                collect(root, &path, found);
            }
            continue;
        }
        if path.extension().is_some_and(|extension| extension == "xaml") {
            let relative = path.strip_prefix(root).expect("a path under the crate directory");
            let parts: Vec<String> =
                relative.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
            found.push((format!("/{}", parts.join("/")), path));
        }
    }
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("assets.rs");

    let mut assets = Vec::new();
    collect(&root, &root.join("Themes"), &mut assets);
    assets.sort();

    let mut text = String::from("pub(crate) static ASSETS: &[(&str, &[u8])] = &[\n");
    let mut documents: Vec<(String, String)> = Vec::with_capacity(assets.len());
    for (asset_path, path) in &assets {
        let path = path.canonicalize().unwrap_or_else(|e| panic!("cannot resolve {}: {e}", path.display()));
        println!("cargo::rerun-if-changed={}", path.display());
        writeln!(text, "    ({asset_path:?}, include_bytes!({:?})),", path.display().to_string()).expect("write");
        let document = fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        documents.push((asset_path.trim_start_matches('/').to_string(), document));
    }
    text.push_str("];\n");

    // The compiled markup of the crate: every theme document, as one group, against the type
    // models (the scan of the sources of this crate and the models of the crates it is built on).
    let documents: Vec<(&str, &str)> = documents.iter().map(|(name, text)| (name.as_str(), text.as_str())).collect();
    Build::from_env()
        .type_system(TypeSystem::Model)
        .compile_group(XamlGroup::new("compiled_xaml").documents(&documents))
        .run();

    // Written only when it changed, so that the crate is not rebuilt for nothing.
    if fs::read_to_string(&out).ok().as_deref() != Some(text.as_str()) {
        fs::write(&out, text).unwrap_or_else(|e| panic!("cannot write {}: {e}", out.display()));
    }
}
