#[cfg(not(any(feature = "runtime-tokio", feature = "runtime-async-std")))]
compile_error!("one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("only one of 'runtime-async-std' or 'runtime-tokio' features must be enabled");

#[cfg(feature = "runtime-tokio")]
pub(crate) use tokio::fs;

#[cfg(feature = "runtime-async-std")]
pub(crate) mod fs {
    use std::fs;
    use std::path::Path;

    // Only need read_to_string
    pub async fn read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
        let path = path.as_ref().to_owned();
        smol::Task::blocking(async move { fs::read_to_string(&path) }).await
    }
}
