use std::path::PathBuf;

use anyhow::Result;
use image_hasher::Hasher;
use rayon::{iter::IntoParallelRefIterator, prelude::ParallelIterator};

use crate::cache::{HashCache, ALREADY_CHECKED_CACHE};

pub fn compare_imgs(
    img_path: &PathBuf,
    other: &Vec<PathBuf>,
    threshold: u32,
    hasher: &Hasher,
    hash_cache: &HashCache,
) -> Result<()> {
    if other.len() == 0 {
        return Ok(());
    }

    let hash1 = hash_cache.try_get(&img_path, hasher)?;

    other
        .iter()
        .map(|other_path| {
            if !already_checked(img_path.to_owned(), other_path.to_owned()) {
                let hash2 = hash_cache.try_get(other_path, hasher)?;

                let dist = hash1.dist(&hash2);
                if dist <= threshold {
                    println!(
                        "\n'{}'\n'{}'\n  dist: {}",
                        img_path.display(),
                        other_path.display(),
                        dist
                    );
                }
            }

            Ok(())
        })
        .collect()
}

fn already_checked(path1: PathBuf, path2: PathBuf) -> bool {
    if path1 == path2 {
        return true;
    }

    let mut comp_cache = ALREADY_CHECKED_CACHE.lock().unwrap();
    return if comp_cache.contains(&(path1.clone(), path2.clone()))
        || comp_cache.contains(&(path2.clone(), path1.clone()))
    {
        true
    } else {
        comp_cache.insert((path1, path2));
        false
    };
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
