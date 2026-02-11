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
        let module = Module::from_file(&self.engine, path)?;
        let name = "unknown".to_string(); // Extract from manifest?

        let mut store = Store::new(&self.engine, PluginState { name: name.clone() });

        let instance = self
            .linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate plugin")?;

        Ok(Plugin {
            instance,
            name,
            store,
        })
    }
}
