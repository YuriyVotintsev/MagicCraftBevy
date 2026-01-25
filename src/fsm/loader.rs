use std::fs;
use std::path::Path;

use super::registry::MobRegistry;
use super::types::MobDef;

pub fn load_mobs(mobs_dir: &str) -> MobRegistry {
    let mut registry = MobRegistry::new();

    let path = Path::new(mobs_dir);
    if !path.exists() {
        return registry;
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read mobs directory: {}", e);
            return registry;
        }
    };

    for entry in entries.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |ext| ext == "ron") {
            match fs::read_to_string(&file_path) {
                Ok(content) => match ron::from_str::<MobDef>(&content) {
                    Ok(mob_def) => {
                        println!("Loaded mob: {}", mob_def.name);
                        registry.insert(mob_def);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse mob file {:?}: {}", file_path, e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read mob file {:?}: {}", file_path, e);
                }
            }
        }
    }

    registry
}
