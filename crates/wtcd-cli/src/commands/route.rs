use std::path::Path;

/// Execute a route query against the routing index (D-13, D-15, D-16).
pub fn run_route(repo_root: &Path, query: &str, top_k: usize) -> wtcd_core::error::Result<()> {
    // 1. Load config to get mirror output_dir
    let config = wtcd_scope::load_config(repo_root)?;
    let mirror_config = config.mirror.unwrap_or_default();

    // 2. Construct routing_index.json path
    //    mirror_config.output_dir is typically "mirror/file"
    //    Index is at mirror/routing_index.json (sibling of mirror/file/)
    let mirror_dir = repo_root.join(&mirror_config.output_dir);
    let index_path = mirror_dir
        .parent()
        .unwrap_or_else(|| Path::new("mirror"))
        .join("routing_index.json");

    // 3. Read index from disk (D-13)
    let index = wtcd_core::index::read_routing_index(&index_path).map_err(|_| {
        wtcd_core::error::WtcdError::ConfigError(
            "Routing index not found. Run 'wtcd run' first to generate the index.".to_string(),
        )
    })?; // D-15

    // 4. Execute query
    let output = wtcd_core::index::route_query(query, &index, top_k);

    // 5. Output JSON (D-16)
    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("{{\"error\": \"Failed to serialize route output: {}\"}}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
