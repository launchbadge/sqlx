use std::fs;
use std::path::Path;
use std::time::SystemTime;

pub fn new<T: AsRef<Path>>(path: T, migration: String) -> Result<(), anyhow::Error> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let path = path.as_ref().join(format!("{}-{}.sql", time, migration));

    fs::File::create(path)?;

    Ok(())
}
