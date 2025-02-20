use anyhow::{anyhow, Context, Result};
use image_hasher::ImageHash;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct HashCache {
    hashes: Arc<Mutex<HashMap<PathBuf, Arc<ImageHash>>>>,
    csv_writer: Arc<Mutex<csv::Writer<File>>>,
}

impl HashCache {
    pub fn load(size: u32, cache_dir: PathBuf) -> Result<HashCache> {
        let persist_path = cache_dir.join(format!("hashes_{size}.csv"));

        let mut path_map = HashMap::new();

        match csv::Reader::from_path(&persist_path) {
            Ok(mut csv_reader) => {
                for result in csv_reader.records() {
                    if let Err(err) = result {
                        eprintln!("Warning: {err}");
                        continue;
                    }

                    let record = result.unwrap();
                    let line = record.position().unwrap().line();

                    let path = record.get(0).map(PathBuf::from);

                    // ignore empty lines
                    if let Some(path) = path {
                        // ignore nonexistent files
                        if !path.exists() {
                            continue;
                            // TODO: verbose output?
                        }

                        let hash_base64 = record
                            .get(1)
                            .ok_or_else(|| anyhow!("Line {line}: Missing image hash"))?;
                        let img_hash = ImageHash::<Box<[u8]>>::from_base64(hash_base64)
                            .map_err(|_| anyhow!("Line {line}: Invalid base64 bytes"))?;

                        let img_hash = Arc::new(img_hash);
                        path_map.insert(path, img_hash.clone());
                    }
                }

                Ok(())
            }

            Err(err) => (|| -> Result<()> {
                if let csv::ErrorKind::Io(io_err) = err.kind() {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        fs::create_dir_all(persist_path.parent().unwrap())
                            .context("Failed to create persist directory")?;
                        File::create(&persist_path).context("Failed to create persist file")?;

                        return Ok(());
                    }
                }

                Err(err.into())
            })(),
        }?;

        // commit cache file back to disk with nonexistent path entries removed
        let _self = HashCache {
            hashes: Arc::new(Mutex::new(path_map)),
            csv_writer: Arc::new(Mutex::new(csv::Writer::from_path(persist_path)?)),
        };
        _self.save_all()?;

        Ok(_self)
    }

    fn save_all(&self) -> Result<()> {
        let mut csv_writer = self.csv_writer.lock().unwrap();

        csv_writer
            .write_record(&["path", "img_hash"])
            .context("Failed to write to persist file")?;

        for (path, img_hash) in self.hashes.lock().unwrap().iter() {
            csv_writer
                .write_record(&[path.display().to_string(), img_hash.to_base64()])
                .context("Failed to write to persist file")?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    fn save_record(&self, (path, img_hash): (&PathBuf, &ImageHash)) -> Result<()> {
        self.csv_writer
            .lock()
            .unwrap()
            .write_record(&[path.display().to_string(), img_hash.to_base64()])
            .context("Failed to append to persist file")?;

        Ok(())
    }

    pub fn flush_writes(&self) -> Result<()> {
        let mut csv_writer = self.csv_writer.lock().unwrap();
        csv_writer.flush()?;
        Ok(())
    }

    pub fn contains(&self, path: &PathBuf) -> bool {
        self.hashes.lock().unwrap().contains_key(path)
    }

    pub fn extend<I>(&self, paths: &I)
    where
        I: Iterator<Item = PathBuf>,
    {
    }

    pub fn get(&self, path: &PathBuf) -> Option<Arc<ImageHash>> {
        self.hashes.lock().unwrap().get(path).cloned()
    }

    pub fn try_get_with(
        &self,
        path: &PathBuf,
        init: impl FnOnce() -> Result<ImageHash>,
    ) -> Result<Arc<ImageHash>> {
        Ok(match self.hashes.lock().unwrap().get(path).cloned() {
            Some(img_hash) => img_hash,

            None => {
                let img_hash = Arc::new(init()?);
                self.hashes
                    .lock()
                    .unwrap()
                    .insert(path.clone(), img_hash.clone());
                self.save_record((&path, &img_hash))?;

                img_hash
            }
        })
    }
}
