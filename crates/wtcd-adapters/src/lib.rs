pub mod go;
pub mod py;
pub mod ts;

use wtcd_core::adapter::AdapterRegistry;

/// Register all built-in language adapters
pub fn register_all_adapters() -> anyhow::Result<AdapterRegistry> {
    let mut registry = AdapterRegistry::new();
    let ts_adapter = ts::TsAdapter::new()?;
    let py_adapter = py::PyAdapter::new()?;
    let go_adapter = go::GoAdapter::new()?;
    registry.register(Box::new(ts_adapter));
    registry.register(Box::new(py_adapter));
    registry.register(Box::new(go_adapter));
    Ok(registry)
}
