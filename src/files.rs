use anyhow::Result;
use std::path::PathBuf;
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

pub fn gather_files(filenames: &Vec<PathBuf>, recurse: bool) -> Result<Vec<PathBuf>> {
    let mut files: Box<dyn Iterator<Item = PathBuf>> = Box::new(
        filenames
            .iter()
            .filter(|path| {
                (path.exists() || {
                    eprintln!("Ignoring '{}': file not found", path.display());
                    false
                }) && (path.is_file()
                    || !recurse && {
                        eprintln!("Ignoring '{}': --recurse not set", path.display());
                        false
                    })
            })
            .map(ToOwned::to_owned),
    );

    if recurse {
        let dirs = filenames
            .iter()
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

    let final_list: Vec<_> = files
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
