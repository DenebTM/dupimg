use anyhow::Result;
use args::Args;
use cache::HashCache;
use clap::Parser;
use compare::hash_paths;
use files::gather_files;
use image_hasher::HasherConfig;
use itertools::Itertools;
use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};
use unique_tuple::UniqueTuple;

use std::{
    collections::HashMap,
    io::{stdout, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

mod args;
mod cache;
mod compare;
mod files;
mod unique_tuple;

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
    hash_cache.flush()?;
    eprintln!("done.");

    let dist_matrix: Arc<Mutex<HashMap<PathBuf, HashMap<PathBuf, u32>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    eprint!("Computing hamming distances... ");
    if args.left_filenames.len() > 0 {
        let left_entries = gather_files(&args.left_filenames, args.recurse)?;

        left_entries.into_par_iter().for_each(|left_entry| {
            compare::compare_all(
                &left_entry,
                &entries,
                args.threshold,
                &hasher,
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
                &hasher,
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
