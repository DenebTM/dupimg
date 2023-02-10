use threadpool::ThreadPool;

use crate::compare::compare_imgs;
use std::{env, path::PathBuf};

mod cache;
mod compare;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    let entries: Result<Vec<PathBuf>, String> = if args.len() >= 2 {
        args.iter()
            .map(|arg| {
                let path = PathBuf::from(arg);

                if path.is_dir() {
                    return Err(format!(
                        "Invalid argument: {}\nCannot process directories",
                        path.display()
                    ));
                }

                let ext = match path.extension() {
                    None => "",
                    Some(ext) => ext.to_str().unwrap_or(""),
                };
                match ext.to_lowercase().as_str() {
                    "jpg" | "png" | "jpeg" | "jfif" => Ok(path),
                    _ => Err(format!(
                        "Invalid argument: {}\nOnly PNG and JPEG files are supported",
                        path.display()
                    )),
                }
            })
            .collect()
    } else {
        Err("Too few arguments given".to_owned())
    };

    match entries {
        Ok(entries) => {
            let pool = ThreadPool::new(num_cpus::get() * 4);

            let entries_iter = entries.clone();
            for entry in entries_iter {
                let entries_b = entries.clone();
                pool.execute(move || {
                    compare_imgs(&entry, &entries_b)
                        .unwrap_or_else(|err| println!("Error while processing images: {:?}", err))
                });
            }

            pool.join();
        }
        Err(err) => print_usage(err.to_owned()),
    }
}

fn print_usage(err: String) {
    eprintln!(
        "
{}

Usage: {} FILENAME1 FILENAME2 [additional file names...]
",
        err,
        env::args().nth(0).unwrap_or("EXECUTABLE_NAME".to_owned())
    )
}
