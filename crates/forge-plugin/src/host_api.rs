use crate::runtime::PluginState;
use anyhow::Result;
use wasmtime::{Caller, Linker};

pub fn register_host_api(linker: &mut Linker<PluginState>) -> Result<()> {
    linker.func_wrap(
        "env",
        "forge_read_buffer",
        |mut _caller: Caller<'_, PluginState>| -> i32 {
            // Placeholder: Return length or pointer?
            // Real implementation requires accessing WASM memory and copying string
            tracing::info!("Plugin called forge_read_buffer");
            0
        },
    )?;

    linker.func_wrap(
        "env",
        "forge_insert_text",
        |mut _caller: Caller<'_, PluginState>, ptr: i32, len: i32| {
            // Placeholder: Read string from memory at ptr/len and insert
            tracing::info!("Plugin called forge_insert_text(ptr={}, len={})", ptr, len);
        },
    )?;

    linker.func_wrap(
        "env",
        "forge_show_notification",
        |mut _caller: Caller<'_, PluginState>, ptr: i32, len: i32, level: i32| {
             tracing::info!("Plugin called forge_show_notification(ptr={}, len={}, level={})", ptr, len, level);
        },
    )?;

    Ok(())
}
