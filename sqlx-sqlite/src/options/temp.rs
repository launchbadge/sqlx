use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{io, mem};
use std::borrow::Cow;
use std::ffi::OsString;
use once_cell::sync::OnceCell;

#[cfg(doc)]
use crate::{SqliteConnectOptions, SqliteConnection};

/// Handle tracking a named, temporary path for a SQLite database.
///
/// The path will be deleted when the last handle is dropped.
///
/// If the path represents a file ([`Self::lazy_file()`], [`Self::builder().file_mode()`]),
/// then only the file itself will be deleted. Any temporary files created by SQLite,
/// if not automatically deleted by SQLite itself, will remain in the parent directory.
///
/// If the path represents a directory ([`Self::lazy_dir()`], [`Self::builder().dir_mode()`]),
/// then the directory and all its contents, including any other files created by SQLite,
/// will be deleted.
///
/// The handle can be cloned and shared with other threads.
/// [`SqliteConnectOptions`] will retain a handle, as will its clones,
/// as will any [`SqliteConnection`]s opened with them.
///
/// [`Self::builder().file_mode()`]: SqliteTempPathBuilder::file_mode
/// [`Self::builder().dir_mode()`]: SqliteTempPathBuilder::file_mode
#[derive(Clone, Debug)]
pub struct SqliteTempPath {
    inner: Arc<TempPathInner>,
}

/// Builder for [`SqliteTempPath`].
///
/// Created by [`SqliteTempPath::builder()`].
#[derive(Debug)]
pub struct SqliteTempPathBuilder {
    inner: TempPathInner,
}

struct TempPathInner {
    db_path: OnceCell<PathBuf>,

    parent_dir: Option<PathBuf>,
    filename: Cow<'static, Path>,
    is_dir: bool,
    create_missing_parents: bool,
}

impl SqliteTempPath {
    /// Lazily create a temporary file in [`std::env::temp_dir()`].
    ///
    /// The file will not be created until the first connection.
    ///
    /// The file will be deleted when the last instance of this handle is dropped.
    ///
    /// For advanced configuration, use [`Self::builder()`] to get a [`SqliteTempPathBuilder`].
    pub fn lazy_file() -> Self {
        Self::builder().build()
    }

    /// Lazily create a temporary directory in [`std::env::temp_dir()`].
    ///
    /// The directory will not be created until the first connection.
    ///
    /// The directory and all its contents, including any other files created by SQLite,
    /// will be deleted when the last instance of this handle is dropped.
    ///
    /// For advanced configuration, use [`Self::builder()`] to get a [`SqliteTempPathBuilder`].
    pub fn lazy_dir() -> Self {
        Self::builder().dir_mode().build()
    }

    /// Create a temporary directory immediately, returning the handle.
    ///
    /// This will spawn a blocking task in the current runtime.
    ///
    /// ### Panics
    /// If no runtime is available.
    pub async fn create_dir() -> io::Result<Self> {
        let this = Self::lazy_dir();
        this.force_create().await?;
        Ok(this)
    }

    /// Get a builder to configure a new temporary path.
    ///
    /// See [`SqliteTempPathBuilder`] for details.
    pub fn builder() -> SqliteTempPathBuilder {
        SqliteTempPathBuilder::new()
    }

    /// Create the temporary path immediately, returning the path to the database file.
    ///
    /// If the path has already been created, this returns immediately.
    ///
    /// This will spawn a blocking task in the current runtime to create the path.
    ///
    /// ### Panics
    /// If no runtime is available.
    ///
    /// See [`.force_create_blocking()`][Self::force_create_blocking]
    /// for a version that blocks instead of spawning a task.
    pub async fn force_create(&self) -> io::Result<&Path> {
        let this = self.clone();

        sqlx_core::rt::spawn_blocking(move || this.force_create_blocking().map(|_| ())).await?;

        Ok(self
            .inner
            .db_path
            .get()
            .expect("BUG: `self.inner` should be initialized at this point!"))
    }

    /// Create the temporary path immediately, returning the path to the database file.
    ///
    /// If the path has already been created, this returns immediately.
    ///
    /// ### Blocking
    /// This requires touching the filesystem, which may block the current thread.
    ///
    /// See [`.force_create()`][Self::force_create] for an asynchronous version
    /// that uses a background task instead of blocking, but requires an async runtime.
    pub fn force_create_blocking(&self) -> io::Result<&Path> {
        Ok(self.inner.try_get()?)
    }

    /// Return the path to the database file if the path has been created.
    ///
    /// If this handle represents a directory, the database file may not exist yet.
    pub fn get_db_path(&self) -> Option<&Path> {
        // For whatever reason, autoderef fails here
        self.inner.db_path.get().map(|p| &**p)
    }
}

impl SqliteTempPathBuilder {
    fn new() -> Self {
        Self {
            inner: TempPathInner {
                db_path: OnceCell::new(),
                parent_dir: None,
                filename: Cow::Borrowed(Path::new("db.sqlite3")),
                is_dir: false,
                create_missing_parents: true,
            }
        }
    }

    /// Configure the builder for creating a temporary file.
    ///
    /// This is the default.
    pub fn file_mode(&mut self) -> &mut Self {
        self.inner.is_dir = false;
        self
    }

    /// Configure the builder for creating a temporary directory.
    pub fn dir_mode(&mut self) -> &mut Self {
        self.inner.is_dir = true;
        self
    }

    /// Set the parent directory to use instead of [`std::env::temp_dir()`].
    ///
    /// Use [`.create_missing_parents()`][Self::create_missing_parents] to set
    /// whether any missing directories in this path are created, or not.
    pub fn parent_dir(&mut self, parent_dir: impl Into<PathBuf>) -> &mut Self {
        self.inner.parent_dir = Some(parent_dir.into());
        self
    }

    /// Set `true` to create any missing parent directories, `false` to error.
    ///
    /// Defaults to `true`.
    pub fn create_missing_parents(&mut self, value: bool) -> &mut Self {
        self.inner.create_missing_parents = value;
        self
    }

    /// Set the database filename to use, if building a directory, or filename suffix otherwise.
    ///
    /// Use of path separators is not recommended.
    pub fn filename(&mut self, filename: impl Into<PathBuf>) -> &mut Self {
        self.inner.filename = Cow::Owned(filename.into());
        self
    }

    /// Build a [`SqliteTempPath`] with the given [`tempfile::Tempdir`].
    ///
    /// The lifetime of the `TempDir` will be managed by `SqliteTempPath`.
    ///
    /// This will clear the [`parent_dir`][Self::parent_dir] setting
    /// and switch to [`dir_mode`][Self::dir_mode].
    ///
    /// The builder may be reused afterward, but is reset to default settings.
    pub fn build_with_tempdir(&mut self, tempdir: tempfile::TempDir) -> SqliteTempPath {
        let mut inner = self.take_inner();

        inner.parent_dir = None;
        inner.is_dir = true;

        // Panic safety: don't disarm `TempDir` until we've set `db_path`.
        inner.db_path.set(tempdir.path().join(&inner.filename))
            .expect("BUG: `db_path` already initialized");

        mem::forget(tempdir);

        SqliteTempPath {
            inner: Arc::new(inner),
        }
    }

    /// Build a [`SqliteTempPath`] with the given settings.
    ///
    /// The builder may be reused afterward, but is reset to default settings.
    pub fn build(&mut self) -> SqliteTempPath {
        SqliteTempPath {
            inner: Arc::new(self.take_inner()),
        }
    }

    fn take_inner(&mut self) -> TempPathInner {
        mem::replace(self, Self::new()).inner
    }
}

impl TempPathInner {
    fn try_get(&self) -> io::Result<&PathBuf> {
        self.db_path.get_or_try_init(move || {
            let mut builder = tempfile::Builder::new();

            builder.prefix("sqlx-sqlite");

            if self.is_dir {
                let mut path = self
                    .parent_dir
                    .as_ref()
                    .map_or_else(|| builder.tempdir(), |parent| builder.tempdir_in(parent))?
                    .into_path();

                path.push(&self.filename);

                Ok(path)
            } else {
                builder.suffix(&*self.filename);

                Ok(self
                    .parent_dir
                    .as_ref()
                    .map_or_else(|| builder.tempfile(), |parent| builder.tempfile_in(parent))?
                    .into_temp_path()
                    // Uses `FileSetAttributeW(FILE_ATTRIBUTE_TEMPORARY)` on Windows
                    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea#caching_behavior
                    .keep()?)
            }
        })
    }
}

impl Debug for TempPathInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TempPathInner")
            .field(
                "db_path",
                &self.db_path.get()
                    .map_or(
                        Path::new("<not yet created>"),
                        |p| p,
                    ),
            )
            .field("parent_dir", &self.parent_dir)
            .field("filename", &self.filename)
            .field("is_dir", &self.is_dir)
            .field("create_missing_parents", &self.create_missing_parents)
            .finish()
    }
}

impl Drop for TempPathInner {
    fn drop(&mut self) {
        let Some(path) = self.db_path.take() else {
            return;
        };

        let remove_dir_all = self.is_dir;

        // Drop the path on a blocking task or fall back to executing it synchronously.
        let res = sqlx_core::rt::try_spawn_blocking(move || {
            let res = if let Some(Some(dir)) = remove_dir_all.then(|| path.parent()) {
                std::fs::remove_dir_all(dir)
            } else {
                std::fs::remove_file(&path)
            };

            match res {
                Ok(()) => {
                    tracing::debug!(remove_dir_all, "successfully deleted SqliteTempPath");
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    tracing::debug!(
                        remove_dir_all,
                        "did not delete SqliteTempPath, not found (error {e:?})"
                    );
                }
                Err(e) => {
                    tracing::warn!(remove_dir_all, "error deleting SqliteTempPath: {e:?}");
                }
            }
        });

        // If a runtime is not available, it's likely we're shutting down or on a worker thread.
        // Either way, we can just block.
        if let Err(remove_sync) = res {
            remove_sync();
        }
    }
}
