use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::LineWriter,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};

use image::DynamicImage;
use image_hasher::{Hasher, ImageHash};
use lazy_static::lazy_static;

static CACHE_LOCATION: &str = "~/.cache/dupimg";

lazy_static! {
    pub static ref ALREADY_CHECKED_CACHE: Mutex<HashSet<(PathBuf, PathBuf)>> =
        Mutex::new(HashSet::new());
}

pub struct HashCache {
    hashes: Arc<Mutex<HashMap<String, ImageHash>>>,
    md5_map: Arc<Mutex<HashMap<PathBuf, String>>>,
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

    pub fn load(
        size: u32,
        hashes: Arc<Mutex<HashMap<String, ImageHash>>>,
        md5_map: Arc<Mutex<HashMap<PathBuf, String>>>,
    ) -> Result<HashCache> {
        let persist_path = Path::new(shellexpand::full(CACHE_LOCATION)?.as_ref())
            .join(format!("hashes_{size}.csv"));

        let (existing_hashes, csv_writer) = match csv::Reader::from_path(&persist_path) {
            Ok(mut csv_reader) => Ok((
                csv_reader
                    .records()
                    .map(|result| -> Result<Option<(String, ImageHash)>> {
                        let record = result?;

                        record
                            .get(0)
                            .map(|md5| {
                                let hash_base64 = record.get(1).ok_or(anyhow!("Missing hash"))?;

                                let img_hash = ImageHash::<Box<[u8]>>::from_base64(hash_base64)
                                    .map_err(|_| anyhow!("Invalid base64 bytes"))?;

                                Ok((md5.to_string(), img_hash))
                            })
                            .transpose()
                    })
                    .map(Result::transpose)
                    .flatten()
                    .collect::<Result<_>>()?,
                Self::create_writer(&persist_path)?,
            )),

            Err(err) => {
                if let csv::ErrorKind::Io(io_err) = err.kind() {
                    if let std::io::ErrorKind::NotFound = io_err.kind() {
                        fs::create_dir_all(persist_path.parent().unwrap())
                            .context("Failed to create persist directory")?;
                        File::create(&persist_path).context("Failed to create persist file")?;

                        Ok((HashMap::new(), Self::create_writer(&persist_path)?))
                    } else {
                        Err(err)
                    }
                } else {
                    Err(err)
                }
            }
        }?;

        hashes.lock().unwrap().extend(existing_hashes);

        Ok(HashCache {
            hashes,
            md5_map,
            csv_writer: Arc::new(Mutex::new(csv_writer)),
        })
    }

    // pub fn save_all(&self) -> Result<()> {
    //     self.csv_writer
    //         .lock()
    //         .unwrap()
    //         .write_record(&["md5", "img_hash"])
    //         .context("Failed to write to persist file")?;

    //     for (md5, img_hash) in self.hashes.lock().unwrap().iter() {
    //         self.csv_writer
    //             .lock()
    //             .unwrap()
    //             .write_record(&[md5, &img_hash.to_base64()])
    //             .context("Failed to write to persist file")?;
    //     }

    //     Ok(())
    // }

    fn save_record(&self, (md5, img_hash): (&String, &ImageHash)) -> Result<()> {
        self.csv_writer
            .lock()
            .unwrap()
            .write_record(&[md5, &img_hash.to_base64()])
            .context("Failed to append to persist file")?;

        Ok(())
    }

    pub fn try_get(&self, path: &PathBuf, hasher: &Hasher) -> Result<ImageHash> {
        if !self.md5_map.lock().unwrap().contains_key(path) {
            let var_name = format!("{:x}", md5::compute(fs::read(path)?));
            let md5 = var_name;
            self.md5_map.lock().unwrap().insert(path.clone(), md5);
        }
        let md5 = self.md5_map.lock().unwrap().get(path).unwrap().clone();

        let maybe_hash = self.hashes.lock().unwrap().get(&md5).cloned();
        Ok(if let Some(img_hash) = maybe_hash {
            img_hash
        } else {
            let img = image::open(path.clone())
                .with_context(|| format!("Could not process '{}'", path.display()))?
                // .adjust_contrast(30.0)
                .into_rgba32f();

            let img = DynamicImage::ImageRgba32F(img);
            let img_hash = hasher.hash_image(&img);
            self.hashes
                .lock()
                .unwrap()
                .insert(md5.clone(), img_hash.clone());
            self.save_record((&md5, &img_hash))?;

            img_hash
        })
    }
}
