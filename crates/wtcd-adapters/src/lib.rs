pub mod c;
pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod go;
pub mod java;
pub mod kotlin;
pub mod py;
pub mod rust;
pub mod swift;
pub mod ts;
pub mod zig;

use wtcd_core::adapter::AdapterRegistry;

/// Register all built-in language adapters
pub fn register_all_adapters() -> anyhow::Result<AdapterRegistry> {
    let mut registry = AdapterRegistry::new();
    let ts_adapter = ts::TsAdapter::new()?;
    let py_adapter = py::PyAdapter::new()?;
    let go_adapter = go::GoAdapter::new()?;
    let c_adapter = c::CAdapter::new()?;
    let cpp_adapter = cpp::CppAdapter::new()?;
    let csharp_adapter = csharp::CSharpAdapter::new()?;
    let dart_adapter = dart::DartAdapter::new()?;
    let java_adapter = java::JavaAdapter::new()?;
    let kotlin_adapter = kotlin::KotlinAdapter::new()?;
    let rust_adapter = rust::RustAdapter::new()?;
    let swift_adapter = swift::SwiftAdapter::new()?;
    let zig_adapter = zig::ZigAdapter::new()?;
    registry.register(Box::new(ts_adapter));
    registry.register(Box::new(py_adapter));
    registry.register(Box::new(go_adapter));
    registry.register(Box::new(c_adapter));
    registry.register(Box::new(cpp_adapter));
    registry.register(Box::new(csharp_adapter));
    registry.register(Box::new(dart_adapter));
    registry.register(Box::new(java_adapter));
    registry.register(Box::new(kotlin_adapter));
    registry.register(Box::new(rust_adapter));
    registry.register(Box::new(swift_adapter));
    registry.register(Box::new(zig_adapter));
    Ok(registry)
}
