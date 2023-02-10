use std::path::PathBuf;

use dssim_core::{Dssim, DssimImage};
use image::{imageops::FilterType, ImageError};
use imgref::Img;

use crate::cache::{PathPair, ALREADY_CHECKED_CACHE, SCALED_IMG_CACHE};

static SCALED_SIZE: u32 = 200;

pub fn compare_img(img_path: &PathBuf, other: &Vec<PathBuf>) -> Result<(), ImageError> {
    if other.len() == 0 {
        return Ok(());
    }

    let d = Dssim::new();
    let img_a = dssim_from_path(img_path, &d).unwrap();

    for other_path in other.iter() {
        if other_path == img_path {
            continue;
        }

        // check if the combination of img_path/other_path or other_path/img_path has already been checked
        let mut comp_cache = ALREADY_CHECKED_CACHE.lock().unwrap();
        if comp_cache.contains(&PathPair {
            0: img_path.to_owned(),
            1: other_path.to_owned(),
        }) || comp_cache.contains(&PathPair {
            0: other_path.to_owned(),
            1: img_path.to_owned(),
        }) {
            continue;
        }
        comp_cache.insert(PathPair {
            0: img_path.to_owned(),
            1: other_path.to_owned(),
        });
        drop(comp_cache);

        let img_b = dssim_from_path(other_path, &d).unwrap();

        let (diff, _) = d.compare(&img_a, &img_b);
        println!(
            "\n'{}'\n'{}'\n  SSIM: {}",
            img_path.display(),
            other_path.display(),
            diff
        );
    }

    Ok(())
}

fn dssim_from_path(path: &PathBuf, dssim: &Dssim) -> Result<DssimImage<f32>, ImageError> {
    use image::io::Reader as ImageReader;

    let mut cache = SCALED_IMG_CACHE.lock().unwrap();
    if !cache.contains_key(path) {
        drop(cache);
        let img = ImageReader::open(path)?
            .decode()?
            .resize_exact(SCALED_SIZE, SCALED_SIZE, FilterType::Nearest)
            .into_rgb32f();

        // Dssim/imgref wrapper struct for the second image to be compared
        let b = Img::new(
            img.as_raw().to_owned(),
            SCALED_SIZE.try_into().unwrap(),
            SCALED_SIZE.try_into().unwrap(),
        );
        cache = SCALED_IMG_CACHE.lock().unwrap();
        cache.insert(path.to_owned(), dssim.create_image(&b).unwrap());
    }

    return Ok(cache.get(path).unwrap().to_owned());
}
