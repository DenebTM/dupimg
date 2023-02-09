use crate::compare::compare_img;
use std::{env, path::PathBuf};

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
            let mut entries_b = entries.to_vec();

            for entry in entries {
                entries_b.remove(0);
                if entries_b.len() < 1 {
                    break;
                }

                println!("{} vs:", entry.display());

                match compare_img(&entry, &entries_b) {
                    Ok(()) => println!("done!"),
                    Err(err) => println!("Error: {}", err),
                }
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
