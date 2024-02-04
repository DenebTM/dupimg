use args::Args;
use clap::Parser;
use compare::prescale;
use dssim_core::Dssim;
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};
use walkdir::WalkDir;

use crate::compare::compare_imgs;
use std::path::PathBuf;

mod args;
mod cache;
mod compare;

fn main() {
    let args = Args::parse();

    match gather_files(&args.filenames, args.recurse) {
        Ok(mut entries) => {
            let dssim = Dssim::new();

            ThreadPoolBuilder::new()
                .num_threads(args.max_threads.unwrap_or(num_cpus::get()))
                .build_global()
                .unwrap();
            if !args.no_prescale {
                eprintln!("Prescaling images, please stand by...");
                for err_path in prescale(&entries.clone(), &dssim) {
                    if let Some(index) = entries.iter().position(|e| e == err_path) {
                        entries.remove(index);
                    }
                }
                eprintln!("Done.");
            }

            if args.left_filenames.len() > 0 {
                if let Ok(left_entries) = gather_files(&args.left_filenames, args.recurse) {
                    left_entries.into_par_iter().for_each(move |left_entry| {
                        compare_imgs(&left_entry, &entries, args.threshold.unwrap(), &dssim)
                            .unwrap_or_else(|err| eprintln!("{err}"))
                    });
                }
            } else {
                entries.clone().into_par_iter().for_each(move |entry| {
                    compare_imgs(&entry, &entries, args.threshold.unwrap(), &dssim)
                        .unwrap_or_else(|err| eprintln!("{err}"))
                });
            }
        }
        Err(err) => eprintln!("{err}"),
    }
}

fn is_allowed_ext(filename: &PathBuf) -> bool {
    let allowed = [
        "jpg", "jpeg", "jfif", "png", "gif", "bmp", "ico", "tiff", "webp", "avif", "pbm", "pgm",
        "ppm", "tga",
    ];
    let ext = match filename.extension() {
        None => "",
        Some(ext) => ext.to_str().unwrap_or(""),
    };

    allowed.contains(&ext.to_lowercase().as_str())
}

fn gather_files(filenames: &Vec<PathBuf>, recurse: bool) -> Result<Vec<PathBuf>, &str> {
    let mut files: Box<dyn Iterator<Item = PathBuf>> = Box::new(
        filenames
            .iter()
            .filter(|f| {
                f.is_file()
                    || !recurse && {
                        eprintln!("Ignoring '{}': --recurse not set", f.display());
                        false
                    }
            })
            .map(|e| e.to_owned()),
    );

    if recurse {
        let dirs = filenames
            .iter()
            .filter(|f| f.is_dir())
            .map(|f| {
                WalkDir::new(f)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path().to_owned())
                    .filter(|e| e.is_file())
            })
            .flatten();

        files = Box::new(files.chain(dirs));
    }

    let final_list: Vec<_> = files
        .into_iter()
        .filter(|f| {
            is_allowed_ext(f) || {
                eprintln!("Ignoring '{}': Unsupported file format", f.display());
                false
            }
        })
        .collect();

    match final_list.len() {
        0 => Err("No files to process"),
        _ => Ok(final_list),
    }
}
