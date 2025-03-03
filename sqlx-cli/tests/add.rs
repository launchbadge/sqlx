use assert_cmd::Command;
use std::cmp::Ordering;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn add_migration_ambiguous() -> anyhow::Result<()> {
    for reversible in [true, false] {
        let files = AddMigrations::new()?
            .run("hello world", reversible, true, true, false)?
            .fs_output()?;
        assert_eq!(files.0, Vec::<FileName>::new());
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct FileName {
    id: u64,
    description: String,
    suffix: String,
}

impl PartialOrd<Self> for FileName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.id != other.id {
            self.id.partial_cmp(&other.id)
        } else {
            self.suffix.partial_cmp(&other.suffix)
        }
    }
}

impl FileName {
    fn assert_is_timestamp(&self) {
        //if the library is still used in 2050, this will need bumping ^^
        assert!(
            self.id < 20500101000000,
            "{self:?} is too high for a timestamp"
        );
        assert!(
            self.id > 20200101000000,
            "{self:?} is too low for a timestamp"
        );
    }
}

impl From<PathBuf> for FileName {
    fn from(path: PathBuf) -> Self {
        let filename = path.file_name().unwrap().to_string_lossy();
        let (id, rest) = filename.split_once("_").unwrap();
        let id: u64 = id.parse().unwrap();
        let (description, suffix) = rest.split_once(".").unwrap();
        Self {
            id,
            description: description.to_string(),
            suffix: suffix.to_string(),
        }
    }
}
#[test]
fn add_migration_sequential() -> anyhow::Result<()> {
    {
        let files = AddMigrations::new()?
            .run("hello world", false, false, true, true)?
            .fs_output()?;
        assert_eq!(files.len(), 1);
        files.assert_is_not_reversible();
        assert_eq!(files.0[0].id, 1);
    }
    {
        let files = AddMigrations::new()?
            .run("hello world1", false, false, true, true)?
            .run("hello world2", true, false, true, true)?
            .fs_output()?;
        assert_eq!(files.len(), 2);
        files.assert_is_not_reversible();
        assert_eq!(files.0[0].id, 1);
        assert_eq!(files.0[1].id, 2);
    }
    Ok(())
}
#[test]
fn add_migration_sequential_reversible() -> anyhow::Result<()> {
    {
        let files = AddMigrations::new()?
            .run("hello world", true, false, true, true)?
            .fs_output()?;
        assert_eq!(files.len(), 2);
        files.assert_is_reversible();
        assert_eq!(files.0[0].id, 1);
        assert_eq!(files.0[0].id, 1);
    }
    {
        let files = AddMigrations::new()?
            .run("hello world1", true, false, true, true)?
            .run("hello world2", true, true, false, true)?
            .run("hello world3", true, false, true, true)?
            .fs_output()?;
        assert_eq!(files.len(), 6);
        files.assert_is_reversible();
        assert_eq!(files.0[0].id, 1);
        assert_eq!(files.0[1].id, 1);
        // sequential -> timestamp is one way
        files.0[2].assert_is_timestamp();
        files.0[3].assert_is_timestamp();
        files.0[4].assert_is_timestamp();
        files.0[5].assert_is_timestamp();
    }
    Ok(())
}

#[test]
fn add_migration_timestamp() -> anyhow::Result<()> {
    {
        let files = AddMigrations::new()?
            .run("hello world", false, true, false, true)?
            .fs_output()?;
        assert_eq!(files.len(), 1);
        files.assert_is_not_reversible();
        files.0[0].assert_is_timestamp();
    }
    {
        let files = AddMigrations::new()?
            .run("hello world1", false, true, false, true)?
            .run("hello world2", true, false, true, true)?
            .fs_output()?;
        assert_eq!(files.len(), 2);
        files.assert_is_not_reversible();
        files.0[0].assert_is_timestamp();
        // sequential -> timestamp is one way
        files.0[1].assert_is_timestamp();
    }
    Ok(())
}
#[test]
fn add_migration_timestamp_reversible() -> anyhow::Result<()> {
    {
        let files = AddMigrations::new()?
            .run("hello world", true, false, false, true)?
            .fs_output()?;
        assert_eq!(files.len(), 2);
        files.assert_is_reversible();
        files.0[0].assert_is_timestamp();
        files.0[1].assert_is_timestamp();
    }
    {
        let files = AddMigrations::new()?
            .run("hello world", true, true, false, true)?
            .fs_output()?;
        assert_eq!(files.len(), 2);
        files.assert_is_reversible();
        files.0[0].assert_is_timestamp();
        files.0[1].assert_is_timestamp();
    }
    {
        let files = AddMigrations::new()?
            .run("hello world1", true, true, false, true)?
            .run("hello world2", true, false, true, true)?
            .fs_output()?;
        assert_eq!(files.len(), 4);
        files.assert_is_reversible();
        files.0[0].assert_is_timestamp();
        files.0[1].assert_is_timestamp();
        files.0[2].assert_is_timestamp();
        files.0[3].assert_is_timestamp();
    }
    Ok(())
}

struct AddMigrationsResult(Vec<FileName>);
impl AddMigrationsResult {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn assert_is_reversible(&self) {
        let mut up_cnt = 0;
        let mut down_cnt = 0;
        for file in self.0.iter() {
            if file.suffix == "down.sql" {
                down_cnt += 1;
            } else if file.suffix == "up.sql" {
                up_cnt += 1;
            } else {
                panic!("unknown suffix for {file:?}");
            }
            assert!(file.description.starts_with("hello_world"));
        }
        assert_eq!(up_cnt, down_cnt);
    }
    fn assert_is_not_reversible(&self) {
        for file in self.0.iter() {
            assert_eq!(file.suffix, "sql");
            assert!(file.description.starts_with("hello_world"));
        }
    }
}
struct AddMigrations(TempDir);

impl AddMigrations {
    fn new() -> anyhow::Result<Self> {
        anyhow::Ok(Self(TempDir::new()?))
    }
    fn run(
        self,
        description: &str,
        revesible: bool,
        timestamp: bool,
        sequential: bool,
        expect_success: bool,
    ) -> anyhow::Result<Self> {
        let cmd_result = Command::cargo_bin("cargo-sqlx")?
            .current_dir(&self.0)
            .args(
                [
                    vec!["sqlx", "migrate", "add", description],
                    match revesible {
                        true => vec!["-r"],
                        false => vec![],
                    },
                    match timestamp {
                        true => vec!["--timestamp"],
                        false => vec![],
                    },
                    match sequential {
                        true => vec!["--sequential"],
                        false => vec![],
                    },
                ]
                .concat(),
            )
            .assert();
        if expect_success {
            cmd_result.success();
        } else {
            cmd_result.failure();
        }
        anyhow::Ok(self)
    }
    fn fs_output(&self) -> anyhow::Result<AddMigrationsResult> {
        let files = recurse_files(&self.0)?;
        let mut fs_paths = Vec::with_capacity(files.len());
        for path in files {
            let relative_path = path.strip_prefix(self.0.path())?.to_path_buf();
            fs_paths.push(FileName::from(relative_path));
        }
        Ok(AddMigrationsResult(fs_paths))
    }
}

fn recurse_files(path: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let mut buf = vec![];
    let entries = read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path())?;
            buf.append(&mut subdir);
        }

        if meta.is_file() {
            buf.push(entry.path());
        }
    }
    buf.sort();
    Ok(buf)
}
