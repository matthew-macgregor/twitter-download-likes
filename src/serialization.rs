use std::error::Error;
use std::path::{Path};

/// Provides traits and helper functions for serialization and
/// deserialization of objects to/from disk.

/// Structs that implement FsCacheable are capable of serializing
/// to disk, although the details are implementation specific.
pub trait FsCacheable<T> {
    /// Cache the serialized entity to disk.
    fn cache(&self, path: &Path) -> Result<&Self, Box<dyn Error>>;
}

/// Structs that implement FsLoadable are capable of deserializing
/// themselves from a file on disk.
pub trait FsLoadable<T> {
    /// Loads the serialized entity into memory.
    fn load(path: &Path) -> Result<T, Box<dyn Error>>;
}
