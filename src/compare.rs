use std::{path::PathBuf, sync::Arc};

use image::DynamicImage;
use image_hasher::{Hasher, ImageHash};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::cache::{ALREADY_CHECKED_CACHE, HASH_CACHE};

pub fn compare_imgs(
    img_path: &PathBuf,
    other: &Vec<PathBuf>,
    threshold: u32,
    hasher: &Hasher,
) -> Result<(), Arc<String>> {
    if other.len() == 0 {
        return Ok(());
    }

    let hash1 = get_cached_hash(img_path, &hasher, other, false)?;

    other
        .iter()
        .map(|other_path| {
            if !already_checked(img_path.to_owned(), other_path.to_owned()) {
                let hash2 = get_cached_hash(other_path, &hasher, other, false)?;

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

pub fn prescale<'a>(paths: &'a Vec<PathBuf>, hasher: &'a Hasher) -> Vec<&'a PathBuf> {
    paths
        .into_par_iter()
        .map(|path| match get_cached_hash(path, hasher, &paths, true) {
            Ok(_) => None,
            Err(err) => {
                eprintln!("{err}");
                Some(path)
            }
        })
        .flatten()
        .collect()
}

fn get_cached_hash(
    path: &PathBuf,
    hasher: &Hasher,
    other: &Vec<PathBuf>,
    precache: bool,
) -> Result<Arc<ImageHash>, Arc<String>> {
    HASH_CACHE.try_get_with(path.to_owned(), || match hash_path(path, hasher) {
        Ok(img) => Ok(Arc::new(img)),
        Err(err) => {
            // mark this image as "already checked" to prevent a million errors
            if !precache {
                let mut comp_cache = ALREADY_CHECKED_CACHE.lock().unwrap();
                for other_path in other {
                    comp_cache.insert((path.to_owned(), other_path.to_owned()));
                }
            }
            Err(err)
        }
    })
}

fn hash_path(path: &PathBuf, hasher: &Hasher) -> Result<ImageHash, String> {
    let img = image::open(path)
        .map_err(|err| format!("Could not process '{}' - {err}", path.display()))?
        // .adjust_contrast(30.0)
        .into_rgba32f();

    let img = DynamicImage::ImageRgba32F(img);

    Ok(hasher.hash_image(&img))
}
