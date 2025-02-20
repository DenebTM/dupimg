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
use rayon::{iter::IntoParallelRefIterator, prelude::ParallelIterator, ThreadPoolBuilder};
use std::{
    collections::{HashMap, HashSet},
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
    let mut lhs_entries = gather_files(&args.lhs_filenames, args.recurse)?;
    if entries.len() < 1 {
        return Ok(());
    }

    let mut hash_cache = HashCache::load(args.hash_size, cache_dir)?;
    eprintln!("Loaded {} entries from cache.", hash_cache.len());

    let all_cached = all(entries.iter().chain(lhs_entries.iter()), |path| {
        hash_cache.contains(path)
    });
    if !all_cached {
        let hasher = HasherConfig::new()
            .hash_size(args.hash_size, args.hash_size)
            .to_hasher();

        let all_entries: HashSet<PathBuf> =
            entries.iter().chain(lhs_entries.iter()).cloned().collect();
        let total = all_entries.len();

        eprintln!("Computing hashes... ");
        let hash_result = hash_all(&all_entries, &hasher, &mut hash_cache)?;
        eprintln!("done.");

        for err_path in hash_result.failed.iter().chain(&hash_result.notfound) {
            entries.remove(err_path);
            lhs_entries.remove(err_path);
        }

        eprintln!("in cache:  {:6>} / {total:6>}", hash_result.cached.len());
        eprintln!("hashed:    {:6>} / {total:6>}", hash_result.new.len());
        eprintln!("failed:    {:6>} / {total:6>}", hash_result.failed.len());
        eprintln!("not found: {:6>} / {total:6>}", hash_result.notfound.len());
    }

    let dist_matrix: Arc<Mutex<HashMap<PathBuf, HashMap<PathBuf, u32>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // check against original args in case all LHS entries were invalid and removed
    let combs: HashSet<UniqueTuple<PathBuf>> = if args.lhs_filenames.len() > 0 {
        lhs_entries
            .iter()
            .cartesian_product(&entries)
            .filter(|(a, b)| a != b)
            .map(|(a, b)| UniqueTuple(a.clone(), b.clone()))
            .collect()
    } else {
        entries
            .iter()
            .tuple_combinations()
            .filter(|(a, b)| a != b)
            .map(|(a, b)| UniqueTuple(a.clone(), b.clone()))
            .collect()
    };

    eprintln!("Computing hamming distances... ");
    combs.par_iter().for_each(|UniqueTuple(path1, path2)| {
        compare::compare(
            path1,
            path2,
            args.threshold,
            &hash_cache,
            dist_matrix.clone(),
        )
        .unwrap_or_else(|err| eprintln!("{err}"))
    });
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
