use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use image_hasher::Hasher;
use rayon::{iter::IntoParallelRefIterator, prelude::ParallelIterator};

use crate::cache::HashCache;

pub fn compare(
    path1: &PathBuf,
    path2: &PathBuf,
    threshold: u32,
    hasher: &Hasher,
    hash_cache: &HashCache,
    close_map: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<HashMap<PathBuf, u32>>>>>>,
) -> Result<()> {
    let hash1 = hash_cache.try_get(path1, hasher)?;
    let hash2 = hash_cache.try_get(path2, hasher)?;

    let dist = hash1.dist(&hash2);
    if dist <= threshold {
        let close_list = {
            let mut locked_map = close_map.lock().unwrap();
            if !locked_map.contains_key(path1) {
                locked_map.insert(path1.clone(), Arc::new(Mutex::new(HashMap::new())));
            }

            locked_map.get(path1).unwrap().clone()
        };
        close_list.lock().unwrap().insert(path2.clone(), dist);
    }

    Ok(())
}

pub fn compare_all(
    path1: &PathBuf,
    other: &Vec<PathBuf>,
    threshold: u32,
    hasher: &Hasher,
    hash_cache: &HashCache,
    close_map: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<HashMap<PathBuf, u32>>>>>>,
) -> Result<()> {
    if other.len() == 0 {
        return Ok(());
    }

    other
        .par_iter()
        .map(|path2| {
            compare(
                path1,
                path2,
                threshold,
                hasher,
                hash_cache,
                close_map.clone(),
            )
        })
        .collect()
}

/// return value: list of paths that could not be hashed
///
/// TODO: this semantically sucks, improve it
pub fn hash_paths<'a>(
    paths: &'a Vec<PathBuf>,
    hasher: &'a Hasher,
    hash_cache: &HashCache,
) -> Vec<&'a PathBuf> {
    paths
        .par_iter()
        .map(|path| match hash_cache.try_get(path, hasher) {
            Ok(_) => None,
            Err(err) => {
                eprintln!("{err}");
                Some(path)
            }
        })
        .flatten()
        .collect()
}
