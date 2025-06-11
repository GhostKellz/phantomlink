use std::fs;
use std::path::PathBuf;

pub fn find_vst_plugins() -> Vec<PathBuf> {
    let dirs = vec![
        dirs::home_dir().map(|h| h.join(".vst")),
        Some(PathBuf::from("/usr/lib/vst")),
        Some(PathBuf::from("/usr/local/lib/vst")),
    ];
    let mut plugins = Vec::new();
    for dir in dirs.into_iter().flatten() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "so") {
                    plugins.push(path);
                }
            }
        }
    }
    plugins
}
