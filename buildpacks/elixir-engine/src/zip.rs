use std::{
    fs::{self, File},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};
use tempfile::tempfile;
use zip::{read::ZipArchive, result::ZipError};

#[derive(Debug)]
pub(crate) enum Error {
    Http(Box<ureq::Error>),
    Directory(std::io::Error),
    Entries(std::io::Error),
    Entry(std::io::Error),
    Path,
    File(std::io::Error),
    Unzip(ZipError),
}

/// Fetches a zip from the url, unzips files to a directory, and strips the
/// first path component. Symlinks are not supported. Default file permissions
/// are assumed. In an error scenario, any archive contents already
/// extracted will not be removed.
///
/// # Errors
///
/// See `Error` for an enumeration of error scenarios.
pub(crate) fn fetch_strip_unzip(
    url: String,
    dest_dir: impl AsRef<std::path::Path>,
) -> Result<(), Error> {
    let destination = dest_dir.as_ref();
    let mut body_reader = ureq::get(&url)
        .call()
        .map_err(|e| Error::Http(Box::new(e)))?
        .into_reader();

    let mut tmp_zip = tempfile().map_err(Error::File)?;
    io::copy(&mut body_reader, &mut tmp_zip).map_err(Error::File)?;

    let mut archive = ZipArchive::new(tmp_zip).map_err(Error::Unzip)?;

    archive.extract(destination).map_err(Error::Unzip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_strip_unzip() {
        let dest = tempfile::tempdir().expect("Couldn't create test tmpdir");

        fetch_strip_unzip(
            "https://github.com/git/git/archive/refs/tags/v1.0.0.zip".to_string(),
            dest.path(),
        )
        .expect("Expected to fetch, strip, unzip");

        let target_path = dest.path().join("git-1.0.0").join("README");
        assert!(target_path.exists());
    }
}
