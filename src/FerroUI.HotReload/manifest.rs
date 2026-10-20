use crate::{HotReloadSession, ReloadError};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
/// Development manifest emitted by `ferroui_build::Build::hot_reload_manifest`.
pub struct ReloadManifest {
    pub files: BTreeMap<String, PathBuf>,
    pub sources: BTreeMap<String, String>,
    pub dependencies: BTreeMap<String, Vec<String>>,
}
impl ReloadManifest {
    pub fn read(path: &Path) -> Result<Self, ReloadError> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| ReloadError::Failed(error.to_string()))?;
        Self::parse(&text)
    }
    pub fn parse(text: &str) -> Result<Self, ReloadError> {
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|error| ReloadError::Failed(error.to_string()))?;
        if value["version"].as_u64() != Some(1) {
            return Err(ReloadError::Failed(
                "Unsupported reload manifest version".into(),
            ));
        }
        let documents = value["documents"]
            .as_array()
            .ok_or_else(|| ReloadError::Failed("Missing documents".into()))?;
        let mut manifest = Self {
            files: BTreeMap::new(),
            sources: BTreeMap::new(),
            dependencies: BTreeMap::new(),
        };
        for document in documents {
            let string = |key: &str| {
                document[key]
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| ReloadError::Failed(format!("Missing document {key}")))
            };
            let uri = crate::document_key(&string("uri")?);
            let base = ferroui_base::utilities::Uri::try_create(
                &uri,
                ferroui_base::utilities::UriKind::Absolute,
            )
            .ok_or_else(|| ReloadError::Failed(format!("Invalid document URI: {uri}")))?;
            if manifest
                .files
                .insert(uri.clone(), PathBuf::from(string("source_path")?))
                .is_some()
            {
                return Err(ReloadError::Failed(format!(
                    "Duplicate document URI: {uri}"
                )));
            }
            manifest.sources.insert(uri.clone(), string("source")?);
            let mut includes = Vec::new();
            if let Some(values) = document["includes"].as_array() {
                for value in values {
                    let text = value
                        .as_str()
                        .ok_or_else(|| ReloadError::Failed("Include must be a URI".into()))?;
                    let relative = ferroui_base::utilities::Uri::try_create(
                        text,
                        ferroui_base::utilities::UriKind::RelativeOrAbsolute,
                    )
                    .ok_or_else(|| ReloadError::Failed(format!("Invalid include: {text}")))?;
                    let resolved = if relative.is_absolute_uri() {
                        relative
                    } else {
                        ferroui_base::utilities::Uri::combine(&base, &relative)
                    };
                    includes.push(resolved.absolute_uri().to_string());
                }
            }
            manifest.dependencies.insert(uri, includes);
        }
        Ok(manifest)
    }
    pub fn session(&self) -> Result<HotReloadSession, ReloadError> {
        let mut session = HotReloadSession::new();
        for (uri, source) in &self.sources {
            session.register_document(uri.clone(), source.clone())?;
        }
        Ok(session)
    }
    /// Watch manifest document directories recursively from the build's source
    /// baseline. Newly discovered documents are reported for conservative admission.
    pub fn watcher(&self, interval: std::time::Duration) -> crate::FileWatcher {
        let mut directories = std::collections::BTreeSet::new();
        for (uri, path) in &self.files {
            if let (Some(parent), Some((prefix, _))) = (path.parent(), uri.rsplit_once('/')) {
                directories.insert((parent.to_path_buf(), format!("{prefix}/")));
            }
        }
        crate::FileWatcher::with_directories(
            self.files.clone(),
            directories
                .into_iter()
                .map(|(path, uri_prefix)| crate::WatchDirectory { path, uri_prefix })
                .collect(),
            interval,
            self.sources.clone(),
        )
    }
    pub fn transitive_dependencies(&self, uri: &str) -> Vec<String> {
        let root = crate::document_key(uri);
        let mut seen = std::collections::BTreeSet::from([root.clone()]);
        let mut pending = vec![root.clone()];
        while let Some(uri) = pending.pop() {
            for included in self.dependencies.get(&uri).into_iter().flatten() {
                if seen.insert(included.clone()) {
                    pending.push(included.clone());
                }
            }
        }
        seen.remove(&root);
        seen.into_iter().collect()
    }
}
