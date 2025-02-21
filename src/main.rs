mod args;
mod compare;
mod files;
mod hash;
mod unique_tuple;

use anyhow::Result;
use args::Args;
use clap::Parser;
use either::Either::{self, Left, Right};
use files::gather_files;
use hash::{cache::HashCache, hash_all};
use image_hasher::HasherConfig;
use itertools::{all, Itertools};
use rayon::{
    iter::{IntoParallelIterator, ParallelBridge},
    prelude::ParallelIterator,
    ThreadPoolBuilder,
};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

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

        if args.filenames.is_empty() {
            return Ok(());
        }
    }

    ThreadPoolBuilder::new()
        .num_threads(args.max_threads.unwrap_or(num_cpus::get()))
        .build_global()?;

    let mut entries = gather_files(&args.filenames, args.recurse)?;
    let mut lhs_entries = gather_files(&args.lhs_filenames, args.recurse)?;
    let all_entries: HashSet<PathBuf> = entries.iter().chain(lhs_entries.iter()).cloned().collect();
    let total = all_entries.len();

    let mut hash_cache = HashCache::load(args.hash_size, cache_dir)?;
    eprintln!("Loaded {} entries from cache.", hash_cache.len());

    let all_cached = all(entries.iter().chain(lhs_entries.iter()), |path| {
        hash_cache.contains(path)
    });
    let (cached, new, failed, notfound) = if !all_cached {
        let hasher = HasherConfig::new()
            .hash_size(args.hash_size, args.hash_size)
            .to_hasher();

        eprintln!("Computing hashes... ");
        let hash_result = hash_all(&all_entries, &hasher, &mut hash_cache)?;
        eprintln!("done.");

        for err_path in hash_result.failed.iter().chain(&hash_result.notfound) {
            entries.remove(err_path);
            lhs_entries.remove(err_path);
        }

        (
            hash_result.cached.len(),
            hash_result.new.len(),
            hash_result.failed.len(),
            hash_result.notfound.len(),
        )
    } else {
        let notfound: Vec<PathBuf> = all_entries
            .into_iter()
            .filter(|path| !path.exists())
            .collect();
        let notfound_len = notfound.len();
        for err_path in notfound {
            entries.remove(&err_path);
            lhs_entries.remove(&err_path);
        }

        (total, 0, 0, notfound_len)
    };

    eprintln!("in cache:  {cached:6>} / {total:6>}");
    eprintln!("hashed:    {new:6>} / {total:6>}");
    eprintln!("failed:    {failed:6>} / {total:6>}");
    eprintln!("not found: {notfound:6>} / {total:6>}");

    // check against original args in case all LHS entries were invalid and removed
    let combs: Either<_, _> = if args.lhs_filenames.len() > 0 {
        Left(lhs_entries.iter().cartesian_product(&entries))
    } else {
        Right(entries.iter().tuple_combinations())
    };

    let readonly_cache = hash_cache.readonly();
    let dist_matrix: Arc<Mutex<HashMap<PathBuf, HashMap<PathBuf, u32>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    eprintln!("Computing hamming distances... ");
    combs
        .into_iter()
        .filter(|(a, b)| a != b)
        .par_bridge()
        .into_par_iter()
        .for_each(|(path1, path2)| {
            compare::compare(
                path1,
                path2,
                args.threshold,
                &readonly_cache,
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
