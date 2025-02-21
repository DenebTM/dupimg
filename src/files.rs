use anyhow::Result;
use std::{collections::HashSet, path::PathBuf};
use walkdir::WalkDir;

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

pub fn gather_files<'a, I>(filenames: &'a I, recurse: bool) -> Result<HashSet<PathBuf>>
where
    &'a I: IntoIterator<Item = &'a PathBuf>,
{
    let mut files: Box<dyn Iterator<Item = PathBuf>> = Box::new(
        filenames
            .into_iter()
            .filter(|path| {
                !path.is_dir()
                    || !recurse && {
                        eprintln!("Ignoring '{}': --recurse not set", path.display());
                        false
                    }
            })
            .cloned(),
    );

    if recurse {
        let dirs = filenames
            .into_iter()
            .filter(|f| f.is_dir())
            .map(|f| {
                WalkDir::new(f)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(Result::ok)
                    .map(|e| e.path().to_owned())
                    .filter(|e| e.is_file())
            })
            .flatten();

        files = Box::new(files.chain(dirs));
    }

    let final_list = files
        .into_iter()
        .map(|path| (path.clone(), path.canonicalize()))
        .filter_map(|(path, canon_path)| match canon_path {
            Ok(entry) => Some(entry),
            Err(err) => {
                eprintln!("Ignoring '{}': {err}", path.display());
                None
            }
        })
        .filter(|file| {
            is_allowed_ext(file) || {
                eprintln!("Ignoring '{}': unsupported file format", file.display());
                false
            }
        })
        .collect();

    Ok(final_list)
}
