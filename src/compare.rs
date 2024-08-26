use std::path::PathBuf;

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
) -> Result<()> {
    let hash1 = hash_cache.try_get(path1, hasher)?;
    let hash2 = hash_cache.try_get(path2, hasher)?;

    let dist = hash1.dist(&hash2);
    if dist <= threshold {
        println!(
            "\n'{}'\n'{}'\n  dist: {}",
            path1.display(),
            path2.display(),
            dist
        );
    }

    Ok(())
}

pub fn compare_all(
    path1: &PathBuf,
    other: &Vec<PathBuf>,
    threshold: u32,
    hasher: &Hasher,
    hash_cache: &HashCache,
) -> Result<()> {
    if other.len() == 0 {
        return Ok(());
    }

    other
        .iter()
        .map(|path2| compare(path1, path2, threshold, hasher, hash_cache))
        .collect()
}

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
