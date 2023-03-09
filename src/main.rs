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

    #[arg(
        short = 'j',
        long = "max-threads",
        help = "Maximum number of worker threads to start
Defaults to $(nproc) * 2"
    )]
    max_threads: Option<usize>,
}

fn main() {
    let args = Args::parse();

    match gather_files(&args.filenames, args.recurse) {
        Ok(mut entries) => {
            let dssim = Dssim::new();

            ThreadPoolBuilder::new()
                .num_threads(args.max_threads.unwrap_or(num_cpus::get() * 2))
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

fn gather_files(paths: &Vec<PathBuf>, recurse: bool) -> Result<Vec<PathBuf>, String> {
    let mut files: Box<dyn Iterator<Item = PathBuf>> = Box::new(
        paths
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
        let dirs = paths
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

    Ok(files
        .into_iter()
        .filter(|f| {
            is_allowed_ext(f) || {
                eprintln!("Ignoring '{}': Unsupported file format", f.display());
                false
            }
        })
        .collect())
}
