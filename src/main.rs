use std::path::PathBuf;

use dssim_core::Dssim;
use image::{imageops::FilterType, ImageError};
use imgref::Img;

fn main() {
    println!("Hello, world!");
    let mut workdir = PathBuf::from(".");
    workdir.push("testimg/Noodle");

    let entries: Vec<PathBuf> = workdir
        .read_dir()
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().unwrap().is_file())
        .filter(|e| match e.path().extension() {
            Some(ext) => {
                let str = ext.to_str().unwrap();
                str == "jpg" || str == "png"
            }
            None => false,
        })
        .map(|e| e.path())
        .collect();

    let mut entries_b = entries.to_vec();

    for e in entries {
        println!("{} vs:", e.display());

        match compare_img(&e, &entries_b) {
            Ok(()) => println!("done!"),
            Err(e) => println!("Error: {}", e),
        }

        entries_b.remove(0);
    }

    // for arg in env::args() {
    //     println!("{arg}");
    // }
}

fn compare_img(img_path: &PathBuf, other: &Vec<PathBuf>) -> Result<(), ImageError> {
    let scaled_size = 200;

    use image::io::Reader as ImageReader;

    if other.len() == 0 {
        return Ok(());
    }

    let d = Dssim::new();

    let img = ImageReader::open(img_path)?
        .decode()?
        .resize(scaled_size, scaled_size, FilterType::Nearest)
        .into_rgb32f();

    let a = Img::new(
        img.as_raw().to_owned(),
        scaled_size.try_into().unwrap(),
        scaled_size.try_into().unwrap(),
    );
    let img_a = d.create_image(&a).unwrap();

    for other_path in other {
        if other_path == img_path {
            continue;
        }
        println!("  {}", other_path.display());

        let other_img = ImageReader::open(other_path)?
            .decode()?
            .resize(200, 200, FilterType::Nearest)
            .into_rgb32f();

        let b = Img::new(
            other_img.as_raw().to_owned(),
            scaled_size.try_into().unwrap(),
            scaled_size.try_into().unwrap(),
        );

        let img_b = d.create_image(&b).unwrap();

        let (diff, _) = d.compare(&img_a, &img_b);
        println!("    {}", diff);
    }

    Ok(())
}
