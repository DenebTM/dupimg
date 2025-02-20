mod args;
mod compare;
mod files;
mod hash;
mod unique_tuple;

use anyhow::Result;
use args::Args;
use clap::Parser;
use files::gather_files;
use hash::{cache::HashCache, hash_all};
use image_hasher::HasherConfig;
use itertools::{all, Itertools};
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use unique_tuple::UniqueTuple;

fn main() -> Result<()> {
    let args = Args::parse();

    let cache_dir = PathBuf::from(shellexpand::full(&args.cache_dir)?.as_ref());

    if args.clean {
        eprintln!("Cleaning cache directory at '{}'", cache_dir.display());
        for file in fs::read_dir(&cache_dir)?
            .filter_map(Result::ok)
            .filter(|e| {
                e.path().is_file()
                    && e.path()
                        .extension()
                        .map_or(false, |ext| ext.to_ascii_lowercase() == "csv")
            })
        {
            fs::remove_file(file.path())?;
        }
    }

    ThreadPoolBuilder::new()
        .num_threads(args.max_threads.unwrap_or(num_cpus::get()))
        .build_global()?;

    let mut entries = gather_files(&args.filenames, args.recurse)?;
    if entries.len() < 1 {
        return Ok(());
    }

    let mut hash_cache = HashCache::load(args.hash_size, cache_dir)?;
    eprintln!("Loaded {} entries from cache.", hash_cache.len());

    let all_cached = all(&entries, |path| hash_cache.contains(path));
    if !all_cached {
        let hasher = HasherConfig::new()
            .hash_size(args.hash_size, args.hash_size)
            .to_hasher();

        eprintln!("Computing hashes... ");
        let hash_result = hash_all(&entries, &hasher, &mut hash_cache)?;
        for err_path in hash_result.failed.iter().chain(&hash_result.notfound) {
            if let Some(index) = entries.iter().position(|e| e == err_path) {
                entries.remove(index);
            }
        }
        eprintln!("done.");
    }

    let dist_matrix: Arc<Mutex<HashMap<PathBuf, HashMap<PathBuf, u32>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    eprintln!("Computing hamming distances... ");
    if args.lhs_filenames.len() > 0 {
        let left_entries = gather_files(&args.lhs_filenames, args.recurse)?;

        left_entries.into_par_iter().for_each(|left_entry| {
            compare::compare_all(
                &left_entry,
                &entries,
                args.threshold,
                &hash_cache,
                dist_matrix.clone(),
            )
            .unwrap_or_else(|err| eprintln!("{err}"))
        });
    } else {
        let combs: Vec<(&PathBuf, &PathBuf)> = entries
            .iter()
            .tuple_combinations()
            .filter(|(a, b)| a != b)
            .map(UniqueTuple::from)
            .unique()
            .map(UniqueTuple::into)
            .collect();

        combs.into_par_iter().for_each(|(path1, path2)| {
            compare::compare(
                path1,
                path2,
                args.threshold,
                &hash_cache,
                dist_matrix.clone(),
            )
            .unwrap_or_else(|err| eprintln!("{err}"))
        })
    }
    eprintln!("done.");

    let mut printed: Vec<PathBuf> = Vec::new();

    for (path1, close_list) in dist_matrix
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .sorted_by_cached_key(|(path1, _)| path1.clone())
    {
        if printed.contains(&path1) {
            continue;
        }

        let path1 = path1.display().to_string().replace('\'', "\\'");
        println!("\n'{path1}'");
        for (path2, dist) in close_list
            .clone()
            .into_iter()
            .sorted_by_cached_key(|(path1, _)| path1.clone())
        {
            printed.push(path2.clone());

            let path2 = path2.display().to_string().replace('\'', "\\'");
            println!("{dist:>6}\t'{path2}'");
        }
    }

    Ok(())
}
