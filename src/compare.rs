use std::path::PathBuf;

use dssim_core::{Dssim, DssimImage};
use image::{imageops::FilterType, ImageError};
use imgref::Img;

use crate::cache::SCALED_IMG_CACHE;

static SCALED_SIZE: u32 = 200;

pub fn compare_img(img_path: &PathBuf, other: &Vec<PathBuf>) -> Result<(), ImageError> {
    if other.len() == 0 {
        return Ok(());
    }

    let d = Dssim::new();
    let img_a = dssim_from_path(img_path, &d).unwrap();

    for other_path in other {
        println!("  {}", other_path.display());

        let img_b = dssim_from_path(other_path, &d).unwrap();

        let (diff, _) = d.compare(&img_a, &img_b);
        println!("    {}", diff);
    }

    Ok(())
}

fn dssim_from_path(path: &PathBuf, dssim: &Dssim) -> Result<DssimImage<f32>, ImageError> {
    use image::io::Reader as ImageReader;

    let mut cache = SCALED_IMG_CACHE.lock().unwrap();
    if !cache.contains_key(path) {
        let img = ImageReader::open(path)?
            .decode()?
            .resize(SCALED_SIZE, SCALED_SIZE, FilterType::Nearest)
            .into_rgb32f();

        // Dssim/imgref wrapper struct for the second image to be compared
        let b = Img::new(
            img.as_raw().to_owned(),
            SCALED_SIZE.try_into().unwrap(),
            SCALED_SIZE.try_into().unwrap(),
        );

        cache.insert(path.to_owned(), dssim.create_image(&b).unwrap());
    }

    return Ok(cache.get(path).unwrap().to_owned());
}
