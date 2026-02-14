use anyhow::{Context, Result};
use std::path::Path;
use wasmtime::{Engine, Instance, Linker, Module, Store};

pub struct PluginState {
    pub name: String,
    // Add other state as needed (e.g., access to editor)
}

pub struct Plugin {
    pub instance: Instance,
    pub name: String,
    pub store: Store<PluginState>,
}

pub struct PluginRuntime {
    engine: Engine,
    linker: Linker<PluginState>,
}

impl PluginRuntime {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        crate::host_api::register_host_api(&mut linker)?;
        Ok(Self { engine, linker })
    }

    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<Plugin> {
        let path_ref = path.as_ref();

        // Attempt to read manifest if adjacent
        let name = if let Some(parent) = path_ref.parent() {
            let manifest_path = parent.join("forge-ext.toml");
            if let Ok(content) = std::fs::read_to_string(manifest_path) {
                if let Ok(manifest) = toml::from_str::<serde_json::Value>(&content) {
                    manifest
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                path_ref
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or("unknown".to_string())
            }
        } else {
            "unknown".to_string()
        };

        let module = Module::from_file(&self.engine, path_ref)
            .context(format!("Failed to load WASM module from {:?}", path_ref))?;

        let mut store = Store::new(&self.engine, PluginState { name: name.clone() });

        let instance = self
            .linker
            .instantiate(&mut store, &module)
            .context(format!("Failed to instantiate plugin '{}'", name))?;

        Ok(Plugin {
            instance,
            name,
            store,
        })
    }
}
