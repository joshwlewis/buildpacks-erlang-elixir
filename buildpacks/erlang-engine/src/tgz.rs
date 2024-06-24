use flate2::read::GzDecoder;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use tar::Archive;

#[derive(Debug)]
pub(crate) enum Error {
    Http(Box<ureq::Error>),
    Directory(std::io::Error),
    Entries(std::io::Error),
    Entry(std::io::Error),
    Path(std::io::Error),
    Unpack(std::io::Error),
}

/// Fetches a tarball from the artifact url, extracts files to a directory,
/// and strips the first path component. Care is taken not to write temporary
/// files or read the entire contents into memory. In an error scenario, any
/// archive contents already extracted will not be removed.
///
/// # Errors
///
/// See `Error` for an enumeration of error scenarios.
pub(crate) fn fetch_extract_strip(
    url: String,
    dest_dir: impl AsRef<std::path::Path>,
) -> Result<(), Error> {
    let destination = dest_dir.as_ref();
    let body = ureq::get(&url)
        .call()
        .map_err(|e| Error::Http(Box::new(e)))?
        .into_reader();

    let mut archive = Archive::new(GzDecoder::new(body));
    for entry in archive.entries().map_err(Error::Entries)? {
        let mut file = entry.map_err(Error::Entry)?;
        let original_path = file.path().map_err(Error::Path)?;
        let stripped_path: PathBuf = original_path.components().skip(1).collect();
        let target_path = destination.join(stripped_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(Error::Directory)?;
        }
        file.unpack(&target_path).map_err(Error::Unpack)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_strip_extract() {
        let dest = tempfile::tempdir().expect("Couldn't create test tmpdir");

        fetch_extract_strip(
            "https://mirrors.edge.kernel.org/pub/software/scm/git/git-0.01.tar.gz".to_string(),
            dest.path(),
        )
        .expect("Expected to fetch, extract");

        let target_path = dest.path().join("README");
        assert!(target_path.exists());
    }
}
