use std::{path::PathBuf, sync::Arc};

use dssim_core::{Dssim, DssimImage};
use image::imageops::FilterType;
use imgref::Img;
use rgb::RGB;

use crate::cache::{ALREADY_CHECKED_CACHE, SCALED_IMG_CACHE};

static SCALED_SIZE: u32 = 200;

pub fn compare_imgs(
    img_path: &PathBuf,
    other: &Vec<PathBuf>,
    threshold: f64,
) -> Result<(), Arc<String>> {
    if other.len() == 0 {
        return Ok(());
    }

    let d = Dssim::new();
    let img1 = get_cached_img(img_path, &d)?;
    return other
        .iter()
        .map(|other_path| compare_img(&img_path, &other_path, &img1, &d, threshold))
        .collect();
}

fn compare_img(
    path1: &PathBuf,
    path2: &PathBuf,
    img1: &DssimImage<f32>,
    dssim: &Dssim,
    threshold: f64,
) -> Result<(), Arc<String>> {
    if !already_checked(path1.to_owned(), path2.to_owned()) {
        let img2 = get_cached_img(path2, &dssim)?;

        let (diff, _) = dssim.compare(img1, img2);
        if diff <= threshold {
            println!(
                "\n'{}'\n'{}'\n  SSIM: {}",
                path1.display(),
                path2.display(),
                diff
            );
        }
    }

    Ok(())
}

fn already_checked(path1: PathBuf, path2: PathBuf) -> bool {
    if path1 == path2 {
        return true;
    }

    let mut comp_cache = ALREADY_CHECKED_CACHE.lock().unwrap();
    if comp_cache.contains(&(path1.to_owned(), path2.to_owned()))
        || comp_cache.contains(&(path2.to_owned(), path1.to_owned()))
    {
        return true;
    }
    comp_cache.insert((path1.to_owned(), path2.to_owned()));
    false
}

fn get_cached_img(path: &PathBuf, dssim: &Dssim) -> Result<Arc<DssimImage<f32>>, Arc<String>> {
    SCALED_IMG_CACHE.try_get_with(path.to_owned(), || match dssim_from_path(path, dssim) {
        Ok(img) => Ok(Arc::new(img)),
        Err(err) => Err(err), // insert stuff into already_checked_cache
    })
}

fn dssim_from_path(path: &PathBuf, dssim: &Dssim) -> Result<DssimImage<f32>, String> {
    use image::io::Reader as ImageReader;

    let img = ImageReader::open(path)
        .map_err(|err| format!("Error while reading '{}': {}", path.display(), err))?
        .decode()
        .map_err(|err| format!("Error while decoding '{}': {}", path.display(), err))?
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
    let src_img = Img::new(
        pixels,
        SCALED_SIZE.try_into().unwrap(),
        SCALED_SIZE.try_into().unwrap(),
    );

    match dssim.create_image(&src_img) {
        Some(img) => Ok(img),
        None => Err(format!(
            "dssim.create_image returned None for {}",
            path.display()
        )),
    }
}
