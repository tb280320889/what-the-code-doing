/// Internal mirror settings (D-16: wtcd-mirror doesn't depend on wtcd-scope)
#[derive(Debug, Clone)]
pub struct MirrorSettings {
    pub output_dir: String,
    pub fp_version: String,
    pub include_manual_appendix: bool,
}

impl Default for MirrorSettings {
    fn default() -> Self {
        Self {
            output_dir: "mirror/file".to_string(),
            fp_version: "1".to_string(),
            include_manual_appendix: false,
        }
    }
}

impl From<wtcd_core::config::MirrorConfig> for MirrorSettings {
    fn from(config: wtcd_core::config::MirrorConfig) -> Self {
        Self {
            output_dir: config.output_dir,
            fp_version: config.fp_version,
            include_manual_appendix: config.include_manual_appendix,
        }
    }
}
