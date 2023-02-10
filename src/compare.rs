use std::path::PathBuf;

use dssim_core::{Dssim, DssimImage};
use image::{imageops::FilterType, ImageError};
use imgref::Img;
use rgb::RGB;

use crate::cache::{ALREADY_CHECKED_CACHE, SCALED_IMG_CACHE};

static SCALED_SIZE: u32 = 200;

pub fn compare_imgs(
    img_path: &PathBuf,
    other: &Vec<PathBuf>,
    threshold: Option<f64>,
) -> Result<(), ImageError> {
    if other.len() == 0 {
        return Ok(());
    }

    let d = Dssim::new();
    let img1 = dssim_from_path(img_path, &d).unwrap();
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
    threshold: Option<f64>,
) -> Result<(), ImageError> {
    if !already_checked(path1.to_owned(), path2.to_owned()) {
        let img2 = dssim_from_path(path2, &dssim)?;
        let (diff, _) = dssim.compare(img1, &img2);
        if threshold.is_none() || diff <= threshold.unwrap() {
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

fn dssim_from_path(path: &PathBuf, dssim: &Dssim) -> Result<DssimImage<f32>, ImageError> {
    use image::io::Reader as ImageReader;

    let mut cache = SCALED_IMG_CACHE.lock().unwrap();
    if !cache.contains_key(path) {
        // release mutex on cache so as not to block other threads
        drop(cache);

        let img = ImageReader::open(path)?
            .decode()?
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

        cache = SCALED_IMG_CACHE.lock().unwrap();
        cache.insert(path.to_owned(), dssim.create_image(&src_img).unwrap());
    }

    return Ok(cache.get(path).unwrap().to_owned());
}
