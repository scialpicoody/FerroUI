use serde_json::json;
use std::path::{Path, PathBuf};
pub(super) fn write(
    manifest_dir: &Path,
    root: &Path,
    out: &Path,
    assembly: &str,
) -> Result<PathBuf, String> {
    let mut files = Vec::new();
    super::collect(manifest_dir, root, &mut files)?;
    let mut documents = Vec::new();
    for (relative, path) in files {
        if path.extension().and_then(|value| value.to_str()) != Some("xaml") {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let xml = roxmltree::Document::parse(&source)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let mut names = Vec::new();
        let mut data_types = Vec::new();
        let mut attributes = Vec::new();
        let mut includes = Vec::new();
        for node in xml.descendants().filter(|node| node.is_element()) {
            for attr in node.attributes() {
                if attr.name() == "Name" {
                    names.push(json!({"name": attr.value(), "type": node.tag_name().name(), "namespace": node.tag_name().namespace()}));
                }
                if attr.name() == "DataType" {
                    data_types.push(attr.value());
                }
                // Preserve every member for metadata-based handler classification by consumers.
                attributes.push(json!({"type": node.tag_name().name(), "namespace": node.tag_name().namespace(), "member": attr.name(), "value": attr.value()}));
            }
            if matches!(
                node.tag_name().name(),
                "StyleInclude" | "ResourceInclude" | "MergeResourceInclude"
            ) {
                if let Some(source) = node.attribute("Source") {
                    includes.push(source);
                }
            }
        }
        documents.push(json!({"uri": format!("ferres://{assembly}/{}", relative.trim_start_matches('/')),
            "source_path": path.canonicalize().map_err(|error| error.to_string())?, "assembly": assembly,
            "class": xml.root_element().attribute(("http://schemas.microsoft.com/winfx/2006/xaml", "Class")),
            "source": source, "names": names, "data_types": data_types, "members": attributes, "includes": includes }));
    }
    let path = out.join("hot-reload.json");
    std::fs::create_dir_all(out).map_err(|error| error.to_string())?;
    let text = serde_json::to_string_pretty(&json!({"version": 1, "documents": documents}))
        .map_err(|error| error.to_string())?;
    if std::fs::read_to_string(&path).ok().as_deref() != Some(&text) {
        std::fs::write(&path, text).map_err(|error| error.to_string())?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifest_records_the_built_source_and_include_contract() {
        let root = std::env::temp_dir().join(format!("ferroui-manifest-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let source = r#"<Border xmlns="https://github.com/ferroui" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" x:Class="Demo.View" x:Name="Root"><Border.Styles><StyleInclude Source="Theme.xaml"/></Border.Styles></Border>"#;
        std::fs::write(root.join("View.xaml"), source).unwrap();
        let output = write(&root, &root, &root.join("generated"), "Demo").unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(output).unwrap()).unwrap();
        assert_eq!(value["version"], 1);
        assert_eq!(value["documents"][0]["class"], "Demo.View");
        assert_eq!(value["documents"][0]["source"], source);
        assert_eq!(value["documents"][0]["includes"][0], "Theme.xaml");
        std::fs::remove_dir_all(root).unwrap();
    }
}
