use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Mutex,
};

use dssim_core::DssimImage;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCALED_IMG_CACHE: Mutex<HashMap<PathBuf, DssimImage<f32>>> =
        Mutex::new(HashMap::new());
    pub static ref ALREADY_CHECKED_CACHE: Mutex<HashSet<(PathBuf, PathBuf)>> =
        Mutex::new(HashSet::new());
}
