use clap::Parser;
use compare::prescale;
use dssim_core::Dssim;
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};

use crate::compare::compare_imgs;
use std::path::PathBuf;

mod cache;
mod compare;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Traverse directories listed in <FILENAMES>
When this is set, all non-image files will be ignored."
    )]
    recurse: bool,

    #[arg(
        short,
        long,
        help = "Only show results with a similarity score <= <THRESHOLD>"
    )]
    threshold: Option<f64>,

    #[arg(help = "Files (and/or directories: -r) to check", required(true))]
    filenames: Vec<PathBuf>,

    #[arg(
        short,
        long,
        help = "Scale images on the first comparison instead of at the start
This may lead to a more unpredictable runtime."
    )]
    no_prescale: bool,
}

fn main() {
    let args = Args::parse();

    match gather_files(&args.filenames, args.recurse) {
        Ok(mut entries) => {
            let dssim = Dssim::new();

            ThreadPoolBuilder::new()
                .num_threads(num_cpus::get() * 2)
                .build_global()
                .unwrap();
            if !args.no_prescale {
                println!("Prescaling images, please stand by...");
                for err_path in prescale(&entries.clone(), &dssim) {
                    if let Some(index) = entries.iter().position(|e| e == err_path) {
                        entries.remove(index);
                    }
                }
                println!("Done.");
            }

            entries.clone().into_par_iter().for_each(move |entry| {
                compare_imgs(&entry, &entries, args.threshold.unwrap_or(0.1), &dssim)
                    .unwrap_or_else(|err| eprintln!("{err}"))
            });
        }
        Err(err) => eprintln!("{err}"),
    }
}

// ik this is overengineered leave me alone
fn gather_files(filenames: &Vec<PathBuf>, recurse: bool) -> Result<Vec<PathBuf>, String> {
    filenames.iter().try_fold(vec![], |files, path| {
        if path.is_dir() {
            return if recurse {
                match &path
                    .read_dir()
                    .map_err(|err| format!("I/O Error in '{}': {}", path.display(), err))?
                    .map(|subentry| match subentry {
                        Ok(de) => Ok(de.path()),
                        Err(err) => Err(format!("I/O Error in '{}': {}", path.display(), err)),
                    })
                    .collect()
                {
                    Ok(col) => Ok([files, gather_files(col, recurse)?].concat()),
                    Err(err) => Err(err.to_owned()),
                }
            } else {
                Err(format!(
                    "Invalid argument: '{}'\n  Will not traverse directories without --recurse",
                    path.display()
                ))
            };
        }

        let ext = match path.extension() {
            None => "",
            Some(ext) => ext.to_str().unwrap_or(""),
        };
        match ext.to_lowercase().as_str() {
            "jpg" | "png" | "jpeg" | "jfif" | "gif" | "bmp" | "ico" | "tiff" | "webp" | "avif"
            | "pbm" | "pgm" | "ppm" | "tga" => Ok([files, vec![path.to_owned()]].concat()),
            _ => {
                if recurse {
                    Ok(files)
                } else {
                    Err(format!(
                        "Invalid argument: '{}'\n  Unsupported format or not an image",
                        path.display()
                    ))
                }
            }
        }
    })
}
