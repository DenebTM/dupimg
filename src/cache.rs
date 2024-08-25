use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use image_hasher::ImageHash;
use lazy_static::lazy_static;
use moka::sync::Cache;

lazy_static! {
    pub static ref HASH_CACHE: Cache<PathBuf, Arc<ImageHash>> = Cache::new(10_000);
    pub static ref ALREADY_CHECKED_CACHE: Mutex<HashSet<(PathBuf, PathBuf)>> =
        Mutex::new(HashSet::new());
}
