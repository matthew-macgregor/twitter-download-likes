use std::fs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

/// Helper function to serialize serde Serializable types to disk.
pub fn write<T>(path: &Path, obj: &T) -> Result<(), Box<dyn Error>>
where
    T: Serialize,
{
    let json_str = serde_json::to_string_pretty(obj)?;
    let path_str = match path.to_str() {
        Some(s) => s,
        None => "unknown path",
    };
    fs::write(path, json_str).expect(&format!("Failed to write file {}", path_str));
    Ok(())
}

/// Helper function to deserialize serde Deserialize types into memory.
pub fn read<T>(path: &Path) -> Result<T, Box<dyn Error>>
where
    T: for<'a> Deserialize<'a>,
{
    let json_str = fs::read_to_string(path)?;
    let result = serde_json::from_str::<T>(&json_str)?;
    Ok(result)
}