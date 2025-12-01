use anyhow::Result;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Returns the result of [`which::which`] but it returns
/// an optional value whether the specified name exists or not.
pub fn which_opt<T: AsRef<OsStr>>(name: T) -> Result<Option<PathBuf>> {
    match which::which(name) {
        Ok(okay) => Ok(Some(okay)),
        Err(which::Error::CannotFindBinaryPath) => Ok(None),
        Err(error) => Err(error.into()),
    }
}
