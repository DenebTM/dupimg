use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use dssim_core::DssimImage;
use lazy_static::lazy_static;
use moka::sync::Cache;

lazy_static! {
    pub static ref SCALED_IMG_CACHE: Cache<PathBuf, Arc<DssimImage<f32>>> = Cache::new(10_000);
    pub static ref ALREADY_CHECKED_CACHE: Mutex<HashSet<(PathBuf, PathBuf)>> =
        Mutex::new(HashSet::new());
}
