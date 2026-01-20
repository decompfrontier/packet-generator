//! Contains a custom VFS trait that allows the parse to parse files from
//! arbitrary sources.

use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    ops::Deref,
    path::{Path, PathBuf},
};

/// A valid reference to a path for the VFS.
#[derive(Debug, PartialEq, PartialOrd, Eq, Hash)]
#[repr(transparent)]
pub struct VfsPath {
    inner: Path,
}

impl VfsPath {
    pub fn new<P: AsRef<Path> + ?Sized>(p: &P) -> &Self {
        // SAFETY:
        //  - `VfsPath` is repr(transparent).
        //  - `p` is a valid path by virtue of having the reference.
        unsafe { &*(std::ptr::from_ref::<Path>(p.as_ref()) as *const Self) }
    }
}

impl ToOwned for VfsPath {
    type Owned = VfsPathBuf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        VfsPathBuf(self.inner.to_owned())
    }

    #[inline]
    fn clone_into(&self, target: &mut Self::Owned) {
        self.inner.clone_into(&mut target.0);
    }
}

/// A valid owned path for the VFS.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub struct VfsPathBuf(PathBuf);

impl Deref for VfsPathBuf {
    type Target = VfsPath;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VfsPath::new(&self.0)
    }
}

impl Borrow<VfsPath> for VfsPathBuf {
    fn borrow(&self) -> &VfsPath {
        self
    }
}

impl VfsPathBuf {
    #[must_use]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.0.to_string_lossy()
    }
}

/// A Virtual File System trait for generically reading files.
pub trait Vfs {
    /// Normalizes a path in some VFS-specific representation.
    ///
    /// Can only provide the following guarantees:
    ///     - Removes all `..` by resolving them to their parent.
    ///     - Removes all `.` by resolving them to the current directory.
    ///
    /// # Errors
    ///
    /// May return `Err` when normalizing the path.
    fn normalize_path(path: &Path) -> Result<VfsPathBuf, std::io::Error>;

    /// # Errors
    ///
    /// May return `Err` when reading the file at `path`.
    fn read_file_to_string(&self, path: &VfsPath) -> Result<String, std::io::Error>;
}

pub struct DefaultFS;

impl Vfs for DefaultFS {
    fn normalize_path(path: &Path) -> Result<VfsPathBuf, std::io::Error> {
        path.canonicalize().map(VfsPathBuf)
    }

    fn read_file_to_string(&self, path: &VfsPath) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&path.inner)
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryFS {
    files: HashMap<VfsPathBuf, String>,
}

impl InMemoryFS {
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Inserts a file in the in-memory filesystem.
    ///
    /// # Errors
    ///
    /// Returns `Err` if path normalization failed.
    pub fn add_file(&mut self, path: &VfsPath, content: &str) -> Result<(), std::io::Error> {
        self.files.insert(path.to_owned(), content.into());

        Ok(())
    }
}

impl Vfs for InMemoryFS {
    fn normalize_path(path: &Path) -> Result<VfsPathBuf, std::io::Error> {
        match path.to_str() {
            Some(s) => Ok(VfsPathBuf(path_clean::clean(s).into())),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidFilename,
                String::from("path is not UTF-8"),
            )),
        }
    }

    fn read_file_to_string(&self, path: &VfsPath) -> Result<String, std::io::Error> {
        self.files.get(path).map(ToOwned::to_owned).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "file {} not present in in-memory database",
                    path.inner.to_string_lossy()
                ),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn vfspath_is_a_path() {
        let path = Path::new("test.kdl");
        let path2 = VfsPath::new(&path);

        assert_eq!(&path2.inner, path);
        assert_eq!(path2.inner.as_os_str(), path.as_os_str());
        assert_eq!(path2.inner.to_string_lossy(), path.to_string_lossy());
    }

    #[test]
    fn vfspathbuf_is_a_pathbuf() {
        let path = PathBuf::from("test.kdl");
        let path2 = VfsPathBuf(path.clone());

        assert_eq!(path2.0.to_string_lossy(), path.to_string_lossy());
    }

    #[test]
    fn inmem_vfs_normalizes() {
        assert_eq!(
            "foo.kdl",
            InMemoryFS::normalize_path(Path::new("foo.kdl"))
                .expect("cannot error: path is UTF-8")
                .inner
                .to_string_lossy()
        );

        assert_eq!(
            "test/foo.kdl",
            InMemoryFS::normalize_path(Path::new("test/foo.kdl"))
                .expect("cannot error: path is UTF-8")
                .inner
                .to_string_lossy()
        );

        assert_eq!(
            "foo.kdl",
            InMemoryFS::normalize_path(Path::new("test/../foo.kdl"))
                .expect("cannot error: path is UTF-8")
                .inner
                .to_string_lossy()
        );

        assert_eq!(
            "foo/bar/quox/a..kdl",
            InMemoryFS::normalize_path(Path::new("./foo/bax/../bar/./ooo/../quox/a..kdl"))
                .expect("cannot error: path is UTF-8")
                .inner
                .to_string_lossy()
        );
    }
}
