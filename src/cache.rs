use std::{
    collections::HashMap,
    fs::{self, File},
    io::{LineWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};

use image::DynamicImage;
use image_hasher::{Hasher, ImageHash};

static CACHE_LOCATION: &str = "~/.cache/dupimg";

pub struct HashCache {
    hashes: Arc<Mutex<HashMap<PathBuf, Arc<ImageHash>>>>,
    csv_writer: Arc<Mutex<csv::Writer<LineWriter<File>>>>,
}

impl HashCache {
    fn create_writer(persist_path: &PathBuf) -> Result<csv::Writer<LineWriter<File>>> {
        let line_writer = LineWriter::new(
            fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(persist_path.clone())?,
        );

        Ok(csv::Writer::from_writer(line_writer))
    }

    pub fn load(size: u32) -> Result<HashCache> {
        let persist_path = Path::new(shellexpand::full(CACHE_LOCATION)?.as_ref())
            .join(format!("hashes_{size}.csv"));

        let mut path_map = HashMap::new();

        match csv::Reader::from_path(&persist_path) {
            Ok(mut csv_reader) => {
                for result in csv_reader.records() {
                    let record = result?;
                    let line = record.position().unwrap().line();

                    let hash_base64 = record
                        .get(1)
                        .ok_or_else(|| anyhow!("Line {line}: Missing image hash"))?;
                    let img_hash = ImageHash::<Box<[u8]>>::from_base64(hash_base64)
                        .map_err(|_| anyhow!("Line {line}: Invalid base64 bytes"))?;

                    let path = record.get(0).map(PathBuf::from);

                    let img_hash = Arc::new(img_hash);
                    path_map.insert(path.unwrap(), img_hash.clone());
                }

                Ok(())
            }

            Err(err) => (|| -> Result<()> {
                if let csv::ErrorKind::Io(io_err) = err.kind() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        fs::create_dir_all(persist_path.parent().unwrap())
                            .context("Failed to create persist directory")?;
                        File::create(&persist_path)
                            .and_then(|mut f| f.write(b"path,img_hash\n"))
                            .context("Failed to create persist file")?;

                        return Ok(());
                    }
                }

                Err(err.into())
            })(),
        }?;

        let csv_writer = Self::create_writer(&persist_path)?;

        Ok(HashCache {
            hashes: Arc::new(Mutex::new(path_map)),
            csv_writer: Arc::new(Mutex::new(csv_writer)),
        })
    }

    // pub fn save_all(&self) -> Result<()> {
    //     self.csv_writer
    //         .lock()
    //         .unwrap()
    //         .write_record(&["path", "img_hash"])
    //         .context("Failed to write to persist file")?;

    //     for (path, img_hash) in self.path_map.lock().unwrap().iter() {
    //         self.csv_writer
    //             .lock()
    //             .unwrap()
    //             .write_record(&[path.display().to_string(), img_hash.to_base64()])
    //             .context("Failed to write to persist file")?;
    //     }

    //     Ok(())
    // }

    fn save_record(&self, (path, img_hash): (&PathBuf, &ImageHash)) -> Result<()> {
        self.csv_writer
            .lock()
            .unwrap()
            .write_record(&[path.display().to_string(), img_hash.to_base64()])
            .context("Failed to append to persist file")?;

        Ok(())
    }

    pub fn try_get(&self, path: &PathBuf, hasher: &Hasher) -> Result<Arc<ImageHash>> {
        let maybe_hash = self.hashes.lock().unwrap().get(path).cloned();
        Ok(if let Some(img_hash) = maybe_hash {
            img_hash
        } else {
            let img = image::open(path.clone())
                .with_context(|| format!("Could not process '{}'", path.display()))?
                // .adjust_contrast(30.0)
                .into_rgba32f();

            let img = DynamicImage::ImageRgba32F(img);
            let img_hash = Arc::new(hasher.hash_image(&img));
            self.hashes
                .lock()
                .unwrap()
                .insert(path.clone(), img_hash.clone());
            self.save_record((&path, &img_hash))?;

            img_hash
        })
    }
}
