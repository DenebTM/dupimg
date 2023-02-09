use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use dssim_core::DssimImage;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SCALED_IMG_CACHE: Mutex<HashMap<PathBuf, DssimImage<f32>>> =
        Mutex::new(HashMap::new());
}
