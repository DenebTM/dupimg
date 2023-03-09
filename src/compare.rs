use std::{path::PathBuf, sync::Arc};

use dssim_core::{Dssim, DssimImage};
use image::imageops::FilterType;
use imgref::Img;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use rgb::RGB;

use crate::cache::{ALREADY_CHECKED_CACHE, SCALED_IMG_CACHE};

static SCALED_SIZE: u32 = 100;

pub fn compare_imgs(
    img_path: &PathBuf,
    other: &Vec<PathBuf>,
    threshold: f64,
    dssim: &Dssim,
) -> Result<(), Arc<String>> {
    if other.len() == 0 {
        return Ok(());
    }

    let img1 = get_cached_img(img_path, &dssim, other, false)?;
    other
        .iter()
        .map(|other_path| {
            if !already_checked(img_path.to_owned(), other_path.to_owned()) {
                let img2 = get_cached_img(other_path, &dssim, other, false)?;

                let (diff, _) = dssim.compare(&img1, img2);
                if diff <= threshold {
                    println!(
                        "\n'{}'\n'{}'\n  SSIM: {}",
                        img_path.display(),
                        other_path.display(),
                        diff
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

pub fn prescale<'a>(paths: &'a Vec<PathBuf>, dssim: &'a Dssim) -> Vec<&'a PathBuf> {
    paths
        .into_par_iter()
        .map(|path| match get_cached_img(path, dssim, &paths, true) {
            Ok(_) => None,
            Err(err) => {
                eprintln!("{err}");
                Some(path)
            }
        })
        .flatten()
        .collect()
}

fn get_cached_img(
    path: &PathBuf,
    dssim: &Dssim,
    other: &Vec<PathBuf>,
    precache: bool,
) -> Result<Arc<DssimImage<f32>>, Arc<String>> {
    SCALED_IMG_CACHE.try_get_with(path.to_owned(), || match dssim_from_path(path, dssim) {
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

fn dssim_from_path(path: &PathBuf, dssim: &Dssim) -> Result<DssimImage<f32>, String> {
    use image::io::Reader as ImageReader;

    let img = ImageReader::open(path)
        .map_err(|err| format!("Could not read '{}' - {err}", path.display()))?
        .decode()
        .map_err(|err| format!("Could not process '{}' - {err}", path.display()))?
        .adjust_contrast(30.0)
        .resize_exact(SCALED_SIZE, SCALED_SIZE, FilterType::Nearest)
        .into_rgb32f();

    // convert image::Rgb to rgb::RGB (why)
    let pixels: Vec<RGB<f32>> = img
        .pixels()
        .map(|p| RGB {
            r: p.0[0],
            g: p.0[1],
            b: p.0[2],
        })
        .collect();

    // Dssim/imgref wrapper struct for the second image to be compared
    let src_img = Img::new(pixels, SCALED_SIZE as usize, SCALED_SIZE as usize);

    match dssim.create_image(&src_img) {
        Some(img) => Ok(img),
        None => Err(format!(
            "dssim.create_image returned None for {}",
            path.display()
        )),
    }
}
