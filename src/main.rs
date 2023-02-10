use clap::Parser;
use threadpool::ThreadPool;

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
When this is set, all non-image files will be ignored"
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
}

fn main() {
    let args = Args::parse();

    match gather_files(&args.filenames, args.recurse) {
        Ok(entries) => {
            let pool = ThreadPool::new(num_cpus::get() * 4);

            let entries_iter = entries.clone();
            for entry in entries_iter {
                let entries_b = entries.clone();
                pool.execute(move || {
                    compare_imgs(&entry, &entries_b, args.threshold)
                        .unwrap_or_else(|err| println!("Error while processing images: {:?}", err))
                });
            }

            pool.join();
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
            "jpg" | "png" | "jpeg" | "jfif" => Ok([files, vec![path.to_owned()]].concat()),
            _ => {
                if recurse {
                    Ok(files)
                } else {
                    Err(format!(
                        "Invalid argument: '{}'\n  Only PNG and JPEG files are supported",
                        path.display()
                    ))
                }
            }
        }
    })
}
