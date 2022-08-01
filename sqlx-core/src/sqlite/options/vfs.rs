use crate::error::Error;
use std::str::FromStr;

/// Refer to [SQLite documentation] for available VFSes.
/// Currently only standard VFSes are supported
///
/// [SQLite documentation]: https://www.sqlite.org/vfs.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteVfs {
    #[cfg(target_family = "unix")]
    Unix,
    #[cfg(target_family = "unix")]
    UnixDotfile,
    #[cfg(target_family = "unix")]
    UnixExcl,
    #[cfg(target_family = "unix")]
    UnixNone,
    #[cfg(target_family = "unix")]
    UnixNamedsem,
    #[cfg(target_family = "windows")]
    Win32,
    #[cfg(target_family = "windows")]
    Win32Longpath,
    #[cfg(target_family = "windows")]
    Win32None,
    #[cfg(target_family = "windows")]
    Win32LongpathNone,
}

impl SqliteVfs {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            #[cfg(target_family = "unix")]
            SqliteVfs::Unix => "unix",
            #[cfg(target_family = "unix")]
            SqliteVfs::UnixDotfile => "unix-dotfile",
            #[cfg(target_family = "unix")]
            SqliteVfs::UnixExcl => "unix-excl",
            #[cfg(target_family = "unix")]
            SqliteVfs::UnixNone => "unix-none",
            #[cfg(target_family = "unix")]
            SqliteVfs::UnixNamedsem => "unix-namedsem",
            #[cfg(target_family = "windows")]
            SqliteVfs::Win32 => "win32",
            #[cfg(target_family = "windows")]
            SqliteVfs::Win32Longpath => "win32-longpath",
            #[cfg(target_family = "windows")]
            SqliteVfs::Win32None => "win32-none",
            #[cfg(target_family = "windows")]
            SqliteVfs::Win32LongpathNone => "win32-longpath-none",
        }
    }
}

impl FromStr for SqliteVfs {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match &*s.to_ascii_lowercase() {
            #[cfg(target_family = "unix")]
            "unix" => SqliteVfs::Unix,
            #[cfg(target_family = "unix")]
            "unix-dotfile" => SqliteVfs::UnixDotfile,
            #[cfg(target_family = "unix")]
            "unix-excl" => SqliteVfs::UnixExcl,
            #[cfg(target_family = "unix")]
            "unix-none" => SqliteVfs::UnixNone,
            #[cfg(target_family = "unix")]
            "unix-namedsem" => SqliteVfs::UnixNamedsem,
            #[cfg(target_family = "windows")]
            "win32" => SqliteVfs::Win32,
            #[cfg(target_family = "windows")]
            "win32-longpath" => SqliteVfs::Win32Longpath,
            #[cfg(target_family = "windows")]
            "win32-none" => SqliteVfs::Win32None,
            #[cfg(target_family = "windows")]
            "win32-longpath-none" => SqliteVfs::Win32LongpathNone,
            _ => {
                return Err(Error::Configuration(
                    format!("unknown value {:?} for `vfs`", s).into(),
                ));
            }
        })
    }
}
