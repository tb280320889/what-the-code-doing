use wtcd_core::types::RunOutput;

/// Format and print run output as JSON (CORE-06)
pub fn format_json(output: &RunOutput) {
    match serde_json::to_string_pretty(output) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("{{\"error\": \"Failed to serialize output: {}\"}}", e);
        }
    }
}
