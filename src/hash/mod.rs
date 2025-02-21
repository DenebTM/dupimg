pub mod cache;

use anyhow::{Context, Result};
use cache::HashCache;
use image::DynamicImage;
use image_hasher::{Hasher, ImageHash};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::path::PathBuf;

pub fn hash(path: &PathBuf, hasher: &Hasher) -> Result<ImageHash> {
    let img = image::open(path.clone())
        .with_context(|| format!("Could not process '{}'", path.display()))?
        // .adjust_contrast(30.0)
        .into_rgba32f();

    let img = DynamicImage::ImageRgba32F(img);
    let img_hash = hasher.hash_image(&img);

    Ok(img_hash)
}

pub struct HashAllResult {
    /// paths for which the hash has been loaded from cache
    pub cached: Vec<PathBuf>,

    /// paths for which a hash has been newly created
    pub new: Vec<PathBuf>,

    /// paths for which hashing failed
    pub failed: Vec<PathBuf>,

    /// paths that could not be found
    pub notfound: Vec<PathBuf>,
}

pub fn hash_all<'a, I>(
    paths: &'a I,
    hasher: &'a Hasher,
    hash_cache: &HashCache,
) -> Result<HashAllResult>
where
    &'a I: IntoIterator<Item = &'a PathBuf> + IntoParallelIterator<Item = &'a PathBuf>,
{
    let notfound: Vec<PathBuf> = paths
        .into_iter()
        .filter(|path| !path.exists())
        .cloned()
        .collect();

    let cached = paths
        .into_iter()
        .filter(|path| hash_cache.contains(path))
        .cloned()
        .collect();

    let (new, failed): (Vec<PathBuf>, Vec<PathBuf>) = paths
        .into_par_iter()
        .map(
            |path| match hash_cache.try_get_with(&path, || hash(&path, hasher)) {
                Ok(_) => (Some(path.clone()), None),
                Err(err) => {
                    eprintln!("{err}");
                    (None, Some(path.clone()))
                }
            },
        )
        .flatten()
        .collect();

    let failed = failed
        .into_iter()
        .filter(|path| !notfound.contains(path))
        .collect();

    hash_cache.flush_writes()?;

    Ok(HashAllResult {
        cached,
        new,
        failed,
        notfound,
    })
}
