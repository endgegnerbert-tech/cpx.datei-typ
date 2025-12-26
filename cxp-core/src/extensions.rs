//! Extension System for CXP files
//!
//! Allows apps to store custom data in CXP files under namespaced directories.
//!
//! Structure:
//! ```text
//! extensions/
//! ├── contextai/           # namespace
//! │   ├── manifest.msgpack # extension metadata
//! │   ├── data1.msgpack    # custom data
//! │   └── data2.msgpack
//! ├── another_ext/
//! │   └── ...
//! ```

use crate::{CxpError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extension metadata stored in manifest.msgpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Extension namespace (e.g., "contextai")
    pub namespace: String,
    /// Extension version (e.g., "1.0.0")
    pub version: String,
    /// Optional description
    pub description: Option<String>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl ExtensionManifest {
    /// Create a new extension manifest
    pub fn new(namespace: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            version: version.into(),
            description: None,
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add metadata key-value pair
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Serialize to MessagePack
    pub fn to_msgpack(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(|e| CxpError::Serialization(e.to_string()))
    }

    /// Deserialize from MessagePack
    pub fn from_msgpack(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data).map_err(|e| CxpError::Serialization(e.to_string()))
    }
}

/// Extension trait for implementing custom extensions
pub trait Extension {
    /// Get the extension namespace (e.g., "contextai")
    fn namespace(&self) -> &str;

    /// Get the extension version (e.g., "1.0.0")
    fn version(&self) -> &str;
}

/// Manages extensions in a CXP file
#[derive(Debug, Default)]
pub struct ExtensionManager {
    /// Registered extensions
    extensions: HashMap<String, ExtensionManifest>,
    /// Extension data cache (namespace -> key -> data)
    data_cache: HashMap<String, HashMap<String, Vec<u8>>>,
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
            data_cache: HashMap::new(),
        }
    }

    /// Register an extension
    pub fn register<E: Extension>(&mut self, ext: E) {
        let manifest = ExtensionManifest::new(ext.namespace(), ext.version());
        self.extensions.insert(ext.namespace().to_string(), manifest);
    }

    /// Register an extension with custom manifest
    pub fn register_manifest(&mut self, manifest: ExtensionManifest) {
        self.extensions.insert(manifest.namespace.clone(), manifest);
    }

    /// Write extension data to cache
    pub fn write_data(&mut self, namespace: &str, key: &str, data: &[u8]) -> Result<()> {
        // Validate namespace is registered
        if !self.extensions.contains_key(namespace) {
            return Err(CxpError::InvalidFormat(
                format!("Extension namespace '{}' not registered", namespace)
            ));
        }

        self.data_cache
            .entry(namespace.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), data.to_vec());

        Ok(())
    }

    /// Read extension data from cache
    pub fn read_data(&self, namespace: &str, key: &str) -> Result<Vec<u8>> {
        self.data_cache
            .get(namespace)
            .and_then(|ns_data| ns_data.get(key))
            .cloned()
            .ok_or_else(|| CxpError::FileNotFound(
                format!("Extension data not found: {}/{}", namespace, key)
            ))
    }

    /// List all registered extension namespaces
    pub fn list_extensions(&self) -> Vec<&str> {
        self.extensions.keys().map(|s| s.as_str()).collect()
    }

    /// Get extension manifest
    pub fn get_manifest(&self, namespace: &str) -> Option<&ExtensionManifest> {
        self.extensions.get(namespace)
    }

    /// List all data keys for a namespace
    pub fn list_data_keys(&self, namespace: &str) -> Vec<&str> {
        self.data_cache
            .get(namespace)
            .map(|ns_data| ns_data.keys().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get all extension data (for writing to ZIP)
    pub fn all_data(&self) -> &HashMap<String, HashMap<String, Vec<u8>>> {
        &self.data_cache
    }

    /// Get extension manifests (for writing to ZIP)
    pub fn manifests(&self) -> &HashMap<String, ExtensionManifest> {
        &self.extensions
    }

    /// Load extension data (for reading from ZIP)
    pub fn load_data(&mut self, namespace: String, key: String, data: Vec<u8>) {
        self.data_cache
            .entry(namespace)
            .or_insert_with(HashMap::new)
            .insert(key, data);
    }

    /// Load extension manifest (for reading from ZIP)
    pub fn load_manifest(&mut self, manifest: ExtensionManifest) {
        self.extensions.insert(manifest.namespace.clone(), manifest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExtension {
        namespace: String,
        version: String,
    }

    impl Extension for TestExtension {
        fn namespace(&self) -> &str {
            &self.namespace
        }

        fn version(&self) -> &str {
            &self.version
        }
    }

    #[test]
    fn test_extension_manifest_creation() {
        let manifest = ExtensionManifest::new("contextai", "1.0.0")
            .with_description("Context AI extension")
            .with_metadata("author", "Example");

        assert_eq!(manifest.namespace, "contextai");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.description, Some("Context AI extension".to_string()));
        assert_eq!(manifest.metadata.get("author"), Some(&"Example".to_string()));
    }

    #[test]
    fn test_extension_manifest_serialization() {
        let manifest = ExtensionManifest::new("contextai", "1.0.0");
        let data = manifest.to_msgpack().unwrap();
        let restored = ExtensionManifest::from_msgpack(&data).unwrap();

        assert_eq!(restored.namespace, manifest.namespace);
        assert_eq!(restored.version, manifest.version);
    }

    #[test]
    fn test_extension_manager_register() {
        let mut manager = ExtensionManager::new();
        let ext = TestExtension {
            namespace: "contextai".to_string(),
            version: "1.0.0".to_string(),
        };

        manager.register(ext);

        let extensions = manager.list_extensions();
        assert_eq!(extensions.len(), 1);
        assert!(extensions.contains(&"contextai"));
    }

    #[test]
    fn test_extension_manager_write_read() {
        let mut manager = ExtensionManager::new();
        let ext = TestExtension {
            namespace: "contextai".to_string(),
            version: "1.0.0".to_string(),
        };

        manager.register(ext);

        let data = b"test data";
        manager.write_data("contextai", "key1", data).unwrap();

        let read_data = manager.read_data("contextai", "key1").unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_extension_manager_unregistered_namespace() {
        let mut manager = ExtensionManager::new();
        let result = manager.write_data("unknown", "key1", b"data");

        assert!(result.is_err());
        assert!(matches!(result, Err(CxpError::InvalidFormat(_))));
    }

    #[test]
    fn test_extension_manager_list_data_keys() {
        let mut manager = ExtensionManager::new();
        let ext = TestExtension {
            namespace: "contextai".to_string(),
            version: "1.0.0".to_string(),
        };

        manager.register(ext);
        manager.write_data("contextai", "key1", b"data1").unwrap();
        manager.write_data("contextai", "key2", b"data2").unwrap();

        let keys = manager.list_data_keys("contextai");
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1"));
        assert!(keys.contains(&"key2"));
    }

    #[test]
    fn test_extension_manager_get_manifest() {
        let mut manager = ExtensionManager::new();
        let manifest = ExtensionManifest::new("contextai", "1.0.0")
            .with_description("Test extension");

        manager.register_manifest(manifest.clone());

        let retrieved = manager.get_manifest("contextai").unwrap();
        assert_eq!(retrieved.namespace, "contextai");
        assert_eq!(retrieved.version, "1.0.0");
        assert_eq!(retrieved.description, Some("Test extension".to_string()));
    }

    #[test]
    fn test_extension_manager_load_data() {
        let mut manager = ExtensionManager::new();

        manager.load_data("contextai".to_string(), "key1".to_string(), b"data1".to_vec());
        manager.load_data("contextai".to_string(), "key2".to_string(), b"data2".to_vec());

        let keys = manager.list_data_keys("contextai");
        assert_eq!(keys.len(), 2);
    }
}
