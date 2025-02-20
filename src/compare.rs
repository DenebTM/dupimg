use crate::hash::cache::HashCache;
use anyhow::Result;
use rayon::{iter::IntoParallelRefIterator, prelude::ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub fn compare(
    path1: &PathBuf,
    path2: &PathBuf,
    threshold: u32,
    hash_cache: &HashCache,
    dist_matrix: Arc<Mutex<HashMap<PathBuf, HashMap<PathBuf, u32>>>>,
) -> Result<()> {
    let hash1 = hash_cache
        .get(path1)
        .ok_or(anyhow::format_err!("No hash for {}", path1.display()))?;
    let hash2 = hash_cache
        .get(path2)
        .ok_or(anyhow::format_err!("No hash for {}", path2.display()))?;

    let dist = hash1.dist(&hash2);
    if dist <= threshold {
        let mut locked_matrix = dist_matrix.lock().unwrap();
        let close_list1 = {
            if !locked_matrix.contains_key(path1) {
                locked_matrix.insert(path1.clone(), HashMap::new());
            }

            locked_matrix.get_mut(path1).unwrap()
        };
        close_list1.insert(path2.clone(), dist);

        let close_list2 = {
            if !locked_matrix.contains_key(path2) {
                locked_matrix.insert(path2.clone(), HashMap::new());
            }

            locked_matrix.get_mut(path2).unwrap()
        };
        close_list2.insert(path1.clone(), dist);
    }

    Ok(())
}

pub fn compare_all(
    path1: &PathBuf,
    other: &HashSet<PathBuf>,
    threshold: u32,
    hash_cache: &HashCache,
    dist_matrix: Arc<Mutex<HashMap<PathBuf, HashMap<PathBuf, u32>>>>,
) -> Result<()> {
    if other.len() == 0 {
        return Ok(());
    }

    other
        .par_iter()
        .map(|path2| compare(path1, path2, threshold, hash_cache, dist_matrix.clone()))
        .collect()
}
