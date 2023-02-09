use crate::compare::compare_img;
use std::{
    env,
    path::PathBuf,
    thread::{self, JoinHandle},
};

mod cache;
mod compare;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    let entries: Result<Vec<PathBuf>, &str> = if args.len() >= 2 {
        args.iter()
            .map(|arg| {
                let path = PathBuf::from(arg);

                if !path.is_file() {
                    return Err("Only files are allowed");
                }

                let ext = match path.extension() {
                    None => "",
                    Some(ext) => ext.to_str().unwrap_or(""),
                };
                match ext.to_lowercase().as_str() {
                    "jpg" | "png" | "jpeg" | "jfif" => Ok(path),
                    _ => Err("Only PNG and JPEG files are supported"),
                }
            })
            .collect()
    } else {
        Err("Too few arguments given")
    };

    match entries {
        Ok(entries) => {
            let mut handles: Vec<JoinHandle<()>> = vec![];

            let entries_iter = entries.clone().into_iter();
            for entry in entries_iter {
                let entries_b = entries.clone();

                handles.push(thread::spawn(move || {
                    compare_img(&entry, &entries_b)
                        .unwrap_or_else(|err| println!("Error: {}", err));
                }))
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }
        Err(err) => print_usage(err),
    }
}

fn print_usage(err: &str) {
    println!(
        "
Error: {}

Usage: {} FILENAME1 FILENAME2 [additional file names...]
",
        err,
        env::args().nth(0).unwrap_or("EXECUTABLE_NAME".to_owned())
    )
}
