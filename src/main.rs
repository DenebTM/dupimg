use anyhow::Result;
use args::Args;
use cache::HashCache;
use clap::Parser;
use compare::hash_paths;
use image_hasher::HasherConfig;
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};
use walkdir::WalkDir;

use crate::compare::compare_imgs;
use std::{
    io::{stdout, Write},
    path::PathBuf,
};

mod args;
mod cache;
mod compare;

fn main() -> Result<()> {
    let args = Args::parse();

    ThreadPoolBuilder::new()
        .num_threads(args.max_threads.unwrap_or(num_cpus::get()))
        .build_global()?;

    let mut entries = gather_files(&args.filenames, args.recurse)?;

    let mut hash_cache = HashCache::load(args.hash_size)?;

    let hasher = HasherConfig::new()
        .hash_size(args.hash_size, args.hash_size)
        .to_hasher();

    eprint!("Calculating hashes... ");
    stdout().flush()?;
    for err_path in hash_paths(&entries.clone(), &hasher, &mut hash_cache) {
        if let Some(index) = entries.iter().position(|e| e == err_path) {
            entries.remove(index);
        }
    }
    eprintln!("done.");

    if args.left_filenames.len() > 0 {
        let left_entries = gather_files(&args.left_filenames, args.recurse)?;

        left_entries.into_par_iter().for_each(move |left_entry| {
            compare_imgs(&left_entry, &entries, args.threshold, &hasher, &hash_cache)
                .unwrap_or_else(|err| eprintln!("{err}"))
        });
    } else {
        entries.clone().into_par_iter().for_each(move |entry| {
            compare_imgs(&entry, &entries, args.threshold, &hasher, &hash_cache)
                .unwrap_or_else(|err| eprintln!("{err}"))
        });
    }

    // hash_cache.save_all()?;

    Ok(())
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

fn gather_files(filenames: &Vec<PathBuf>, recurse: bool) -> Result<Vec<PathBuf>> {
    let mut files: Box<dyn Iterator<Item = PathBuf>> = Box::new(
        filenames
            .iter()
            .filter(|f| {
                (f.exists() || {
                    eprintln!("Ignoring '{}': file not found", f.display());
                    false
                }) && (f.is_file()
                    || !recurse && {
                        eprintln!("Ignoring '{}': --recurse not set", f.display());
                        false
                    })
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
                eprintln!("Ignoring '{}': unsupported file format", f.display());
                false
            }
        })
        .collect();

    Ok(final_list)
}
