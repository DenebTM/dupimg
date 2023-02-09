use std::{env, path::PathBuf};

use dssim_core::Dssim;
use image::{imageops::FilterType, ImageError};
use imgref::Img;

static SCALED_SIZE: u32 = 200;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    let entries: Result<Vec<PathBuf>, &str> = if args.len() >= 2 {
        args.iter()
            .map(|arg| {
                let path = PathBuf::from(arg);

                if !path.is_file() {
                    return Err("Only files are allowed");
                }

                let ext = match path.extension() {
                    None => "",
                    Some(ext) => ext.to_str().unwrap_or(""),
                };
                match ext.to_lowercase().as_str() {
                    "jpg" | "png" | "jpeg" | "jfif" => Ok(path),
                    _ => Err("Only PNG and JPEG files are supported"),
                }
            })
            .collect()
    } else {
        Err("Too few arguments given")
    };

    match entries {
        Ok(entries) => {
            let mut entries_b = entries.to_vec();

            for e in entries {
                entries_b.remove(0);
                if entries_b.len() < 1 {
                    break;
                }

                println!("{} vs:", e.display());

                match compare_img(&e, &entries_b) {
                    Ok(()) => println!("done!"),
                    Err(e) => println!("Error: {}", e),
                }
            }
        }
        Err(err) => print_usage(err),
    }
}

fn print_usage(err: &str) {
    println!(
        "
Error: {}

Usage: {} FILENAME1 FILENAME2 [additional file names...]
",
        err,
        env::args().nth(0).unwrap_or("EXECUTABLE_NAME".to_owned())
    )
}

fn compare_img(img_path: &PathBuf, other: &Vec<PathBuf>) -> Result<(), ImageError> {
    use image::io::Reader as ImageReader;
    if other.len() == 0 {
        return Ok(());
    }

    let d = Dssim::new();

    let img = ImageReader::open(img_path)?
        .decode()?
        .resize(SCALED_SIZE, SCALED_SIZE, FilterType::Nearest)
        .into_rgb32f();

    // Dssim/imgref wrapper struct for the first image to be compared
    let a = Img::new(
        img.as_raw().to_owned(),
        SCALED_SIZE.try_into().unwrap(),
        SCALED_SIZE.try_into().unwrap(),
    );
    let img_a = d.create_image(&a).unwrap();

    for other_path in other {
        println!("  {}", other_path.display());

        let other_img = ImageReader::open(other_path)?
            .decode()?
            .resize(SCALED_SIZE, SCALED_SIZE, FilterType::Nearest)
            .into_rgb32f();

        // Dssim/imgref wrapper struct for the second image to be compared
        let b = Img::new(
            other_img.as_raw().to_owned(),
            SCALED_SIZE.try_into().unwrap(),
            SCALED_SIZE.try_into().unwrap(),
        );
        let img_b = d.create_image(&b).unwrap();

        let (diff, _) = d.compare(&img_a, &img_b);
        println!("    {}", diff);
    }

    Ok(())
}
