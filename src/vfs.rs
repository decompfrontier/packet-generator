//! Contains a custom VFS trait that allows the parse to parse files from
//! arbitrary sources.

use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    ops::Deref,
    path::{Path, PathBuf},
};

/// A valid reference to a path for the VFS.
///
/// This type should only be constructed by a [`Vfs::normalize_path`].
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
        VfsPathBuf {
            inner: self.inner.to_owned(),
        }
    }

    #[inline]
    fn clone_into(&self, target: &mut Self::Owned) {
        self.inner.clone_into(&mut target.inner);
    }
}

/// A valid owned path for the VFS.
///
/// This type should only be constructed by a [`Vfs::normalize_path`].
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
#[repr(transparent)]
pub struct VfsPathBuf {
    inner: PathBuf,
}

impl VfsPathBuf {}

impl Deref for VfsPathBuf {
    type Target = VfsPath;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VfsPath::new(&self.inner)
    }
}

impl Borrow<VfsPath> for VfsPathBuf {
    fn borrow(&self) -> &VfsPath {
        self
    }
}

/// Trait for dealing with path sanitization.
///
/// Normal users should never use this trait but always prefer
/// [`Vfs::normalize_path`].
pub trait VfsSanitize {
    /// Function that sanitizes a [`Path`] to be usable with a [`Vfs`].
    ///
    ///
    /// # Errors
    ///
    /// The error-conditions depend on the underlying implementation.
    fn sanitize(path: &Path) -> Result<PathBuf, std::io::Error>;
}

impl VfsPathBuf {
    #[must_use = "Constructing a `VfsPathBuf` implies that you want to use it."]
    /// Constructs a path usable by a [`Vfs`].
    ///
    /// # Errors
    ///
    /// The error-conditions depend on the underlying implementation of
    /// the trait method [`VfsSanitize::sanitize`].
    pub fn new_from_vfs<V>(path: impl AsRef<Path>) -> Result<Self, std::io::Error>
    where
        V: Vfs + VfsSanitize,
    {
        let sanitized = <V as VfsSanitize>::sanitize(path.as_ref())?;
        Ok(Self { inner: sanitized })
    }

    #[must_use]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.inner.to_string_lossy()
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

impl VfsSanitize for DefaultFS {
    fn sanitize(path: &Path) -> Result<PathBuf, std::io::Error> {
        path.canonicalize()
    }
}

impl Vfs for DefaultFS {
    fn normalize_path(path: &Path) -> Result<VfsPathBuf, std::io::Error> {
        VfsPathBuf::new_from_vfs::<Self>(path)
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

impl VfsSanitize for InMemoryFS {
    fn sanitize(path: &Path) -> Result<PathBuf, std::io::Error> {
        Ok(clean_path::clean(path))
    }
}

impl Vfs for InMemoryFS {
    fn normalize_path(path: &Path) -> Result<VfsPathBuf, std::io::Error> {
        match path.to_str() {
            Some(s) => Ok(VfsPathBuf::new_from_vfs::<Self>(s)?),
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
        let path2 = VfsPathBuf {
            inner: path.clone(),
        };

        assert_eq!(path2.inner.to_string_lossy(), path.to_string_lossy());
    }

    #[test]
    fn vfspath_can_clone_into_vfspathbuf() {
        let vfs_path = VfsPath::new("a");
        let mut path = VfsPathBuf {
            inner: PathBuf::new(),
        };
        vfs_path.clone_into(&mut path);

        assert_eq!(&path.inner, "a");
        assert_eq!(&path.inner, &vfs_path.inner);
    }

    #[test]
    fn vfspathbuf_to_string_lossy_works() {
        let path_a = VfsPath::new("a").to_owned();
        assert_eq!(path_a.to_string_lossy(), Cow::Borrowed("a"));

        #[cfg(not(windows))]
        let os_str =
            unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(&[0x61, 0x62, 0xE3, 0x82]) };

        #[cfg(windows)]
        let os_string = {
            use std::os::windows::prelude::*;

            std::ffi::OsString::from_wide(&[0x0061, 0x0062, 0xD800])
        };

        #[cfg(windows)]
        let os_str = &os_string;

        let path_b = VfsPath::new(os_str).to_owned();

        let expected: Cow<'_, str> = Cow::Owned(String::from("ab�"));
        assert_eq!(path_b.to_string_lossy(), expected);
    }

    #[test]
    fn vfspathbuf_derefs_from_box() {
        let path = VfsPath::new("a").to_owned();
        let boxed_path = Box::new(path.clone());
        assert_eq!(path, *boxed_path);
        assert_eq!(&path.inner, "a");
    }

    #[test]
    fn inmem_vfs_adds_files() {
        let mut fs = InMemoryFS::new();

        let path = InMemoryFS::normalize_path(Path::new("foo")).expect("normalization works");

        let () = fs.add_file(&path, "abcd").expect("can add file");

        let content = fs.read_file_to_string(&path).expect("can read file");

        assert_eq!(content, "abcd");
    }

    #[test]
    fn inmem_vfs_normalizes() {
        let path = Path::new("foo.kdl");
        assert_eq!(
            *path,
            InMemoryFS::normalize_path(path)
                .expect("cannot error: path is UTF-8")
                .inner
        );

        let path = PathBuf::from_iter(["test", "foo.kdl"]);
        assert_eq!(
            path,
            InMemoryFS::normalize_path(&path)
                .expect("cannot error: path is UTF-8")
                .inner
        );

        assert_eq!(
            *Path::new("foo.kdl"),
            InMemoryFS::normalize_path(&PathBuf::from_iter(["test", "..", "foo.kdl"]))
                .expect("cannot error: path is UTF-8")
                .inner
        );

        assert_eq!(
            PathBuf::from_iter(["foo", "bar", "quox", "a..kdl"]),
            InMemoryFS::normalize_path(&PathBuf::from_iter([
                ".", "foo", "bax", "..", "bar", ".", "ooo", "..", "quox", "a..kdl"
            ]))
            .expect("cannot error: path is UTF-8")
            .inner
        );
    }
}
