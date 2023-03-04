use std::borrow::Cow;
use std::cmp;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, Error, ErrorKind, SeekFrom};
use std::marker;
use std::path::{Component, Path, PathBuf};

use filetime::{self, FileTime};

use crate::archive::ArchiveInner;
use crate::error::TarError;
use crate::header::bytes2path;
use crate::other;
use crate::{Archive, Header};

/// A read-only view into an entry of an archive.
///
/// This structure is a window into a portion of a borrowed archive which can
/// be inspected. It acts as a file handle by implementing the Reader trait. An
/// entry cannot be rewritten once inserted into an archive.
pub struct Entry<'a, R: 'a + Read> {
    fields: EntryFields<'a>,
    _ignored: marker::PhantomData<&'a Archive<R>>,
}

// private implementation detail of `Entry`, but concrete (no type parameters)
// and also all-public to be constructed from other modules.
pub struct EntryFields<'a> {
    pub long_pathname: Option<Vec<u8>>,
    pub long_linkname: Option<Vec<u8>>,
    pub header: Header,
    pub size: u64,
    pub header_pos: u64,
    pub file_pos: u64,
    pub data: Vec<EntryIo<'a>>,
    pub preserve_permissions: bool,
    pub preserve_ownerships: bool,
    pub preserve_mtime: bool,
    pub overwrite: bool,
}

pub enum EntryIo<'a> {
    Pad(io::Take<io::Repeat>),
    Data(io::Take<&'a ArchiveInner<dyn Read + 'a>>),
}

/// When unpacking items the unpacked thing is returned to allow custom
/// additional handling by users. Today the File is returned, in future
/// the enum may be extended with kinds for links, directories etc.
#[derive(Debug)]
pub enum Unpacked {
    /// A file was unpacked.
    File(std::fs::File),
    /// A directory, hardlink, symlink, or other node was unpacked.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl<'a, R: Read> Entry<'a, R> {
    /// Returns the path name for this entry.
    ///
    /// This method may fail if the pathname is not valid Unicode and this is
    /// called on a Windows platform.
    ///
    /// Note that this function will convert any `\` characters to directory
    /// separators, and it will not always return the same value as
    /// `self.header().path()` as some archive formats have support for longer
    /// path names described in separate entries.
    ///
    /// It is recommended to use this method instead of inspecting the `header`
    /// directly to ensure that various archive formats are handled correctly.
    pub fn path(&self) -> io::Result<Cow<Path>> {
        self.fields.path()
    }

    /// Returns the raw bytes listed for this entry.
    ///
    /// Note that this function will convert any `\` characters to directory
    /// separators, and it will not always return the same value as
    /// `self.header().path_bytes()` as some archive formats have support for
    /// longer path names described in separate entries.
    pub fn path_bytes(&self) -> Cow<[u8]> {
        self.fields.path_bytes()
    }

    /// Returns the link name for this entry, if any is found.
    ///
    /// This method may fail if the pathname is not valid Unicode and this is
    /// called on a Windows platform. `Ok(None)` being returned, however,
    /// indicates that the link name was not present.
    ///
    /// Note that this function will convert any `\` characters to directory
    /// separators, and it will not always return the same value as
    /// `self.header().link_name()` as some archive formats have support for
    /// longer path names described in separate entries.
    ///
    /// It is recommended to use this method instead of inspecting the `header`
    /// directly to ensure that various archive formats are handled correctly.
    pub fn link_name(&self) -> io::Result<Option<Cow<Path>>> {
        self.fields.link_name()
    }

    /// Returns the link name for this entry, in bytes, if listed.
    ///
    /// Note that this will not always return the same value as
    /// `self.header().link_name_bytes()` as some archive formats have support for
    /// longer path names described in separate entries.
    pub fn link_name_bytes(&self) -> Option<Cow<[u8]>> {
        self.fields.link_name_bytes()
    }

    /// Returns access to the header of this entry in the archive.
    ///
    /// This provides access to the metadata for this entry in the archive.
    pub fn header(&self) -> &Header {
        &self.fields.header
    }

    /// Returns access to the size of this entry in the archive.
    ///
    /// In the event the size is stored in a pax extension, that size value
    /// will be referenced. Otherwise, the entry size will be stored in the header.
    pub fn size(&self) -> u64 {
        self.fields.size
    }

    /// Writes this file to the specified location.
    ///
    /// This function will write the entire contents of this file into the
    /// location specified by `dst`. Metadata will also be propagated to the
    /// path `dst`.
    ///
    /// This function will create a file at the path `dst`, and it is required
    /// that the intermediate directories are created. Any existing file at the
    /// location `dst` will be overwritten.
    ///
    /// > **Note**: This function does not have as many sanity checks as
    /// > `Archive::unpack` or `Entry::unpack_in`. As a result if you're
    /// > thinking of unpacking untrusted tarballs you may want to review the
    /// > implementations of the previous two functions and perhaps implement
    /// > similar logic yourself.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use tar::Archive;
    ///
    /// let mut ar = Archive::new(File::open("foo.tar").unwrap());
    ///
    /// for (i, file) in ar.entries().unwrap().enumerate() {
    ///     let mut file = file.unwrap();
    ///     file.unpack(format!("file-{}", i)).unwrap();
    /// }
    /// ```
    pub fn unpack<P: AsRef<Path>>(&mut self, dst: P) -> io::Result<Unpacked> {
        self.fields.unpack(None, dst.as_ref())
    }

    /// Extracts this file under the specified path, avoiding security issues.
    ///
    /// This function will write the entire contents of this file into the
    /// location obtained by appending the path of this file in the archive to
    /// `dst`, creating any intermediate directories if needed. Metadata will
    /// also be propagated to the path `dst`. Any existing file at the location
    /// `dst` will be overwritten.
    ///
    /// This function carefully avoids writing outside of `dst`. If the file has
    /// a '..' in its path, this function will skip it and return false.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use tar::Archive;
    ///
    /// let mut ar = Archive::new(File::open("foo.tar").unwrap());
    ///
    /// for (i, file) in ar.entries().unwrap().enumerate() {
    ///     let mut file = file.unwrap();
    ///     file.unpack_in("target").unwrap();
    /// }
    /// ```
    pub fn unpack_in<P: AsRef<Path>>(&mut self, dst: P) -> io::Result<bool> {
        self.fields.unpack_in(dst.as_ref())
    }
}

impl<'a, R: Read> Read for Entry<'a, R> {
    fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
        self.fields.read(into)
    }
}

impl<'a> EntryFields<'a> {
    pub fn from<R: Read>(entry: Entry<R>) -> EntryFields {
        entry.fields
    }

    pub fn into_entry<R: Read>(self) -> Entry<'a, R> {
        Entry {
            fields: self,
            _ignored: marker::PhantomData,
        }
    }

    pub fn read_all(&mut self) -> io::Result<Vec<u8>> {
        // Preallocate some data but don't let ourselves get too crazy now.
        let cap = cmp::min(self.size, 128 * 1024);
        let mut v = Vec::with_capacity(cap as usize);
        self.read_to_end(&mut v).map(|_| v)
    }

    fn path(&self) -> io::Result<Cow<Path>> {
        bytes2path(self.path_bytes())
    }

    fn path_bytes(&self) -> Cow<[u8]> {
        match self.long_pathname {
            Some(ref bytes) => {
                if let Some(&0) = bytes.last() {
                    Cow::Borrowed(&bytes[..bytes.len() - 1])
                } else {
                    Cow::Borrowed(bytes)
                }
            }
            None => self.header.path_bytes(),
        }
    }

    /// Gets the path in a "lossy" way, used for error reporting ONLY.
    fn path_lossy(&self) -> String {
        String::from_utf8_lossy(&self.path_bytes()).to_string()
    }

    fn link_name(&self) -> io::Result<Option<Cow<Path>>> {
        match self.link_name_bytes() {
            Some(bytes) => bytes2path(bytes).map(Some),
            None => Ok(None),
        }
    }

    fn link_name_bytes(&self) -> Option<Cow<[u8]>> {
        match self.long_linkname {
            Some(ref bytes) => {
                if let Some(&0) = bytes.last() {
                    Some(Cow::Borrowed(&bytes[..bytes.len() - 1]))
                } else {
                    Some(Cow::Borrowed(bytes))
                }
            }
            None => self.header.link_name_bytes(),
        }
    }

    fn unpack_in(&mut self, dst: &Path) -> io::Result<bool> {
        // Notes regarding bsdtar 2.8.3 / libarchive 2.8.3:
        // * Leading '/'s are trimmed. For example, `///test` is treated as
        //   `test`.
        // * If the filename contains '..', then the file is skipped when
        //   extracting the tarball.
        // * '//' within a filename is effectively skipped. An error is
        //   logged, but otherwise the effect is as if any two or more
        //   adjacent '/'s within the filename were consolidated into one
        //   '/'.
        //
        // Most of this is handled by the `path` module of the standard
        // library, but we specially handle a few cases here as well.

        let mut file_dst = dst.to_path_buf();
        {
            let path = self.path().map_err(|e| {
                TarError::new(
                    format!("invalid path in entry header: {}", self.path_lossy()),
                    e,
                )
            })?;
            for part in path.components() {
                match part {
                    // Leading '/' characters, root paths, and '.'
                    // components are just ignored and treated as "empty
                    // components"
                    Component::Prefix(..) | Component::RootDir | Component::CurDir => continue,

                    // If any part of the filename is '..', then skip over
                    // unpacking the file to prevent directory traversal
                    // security issues.  See, e.g.: CVE-2001-1267,
                    // CVE-2002-0399, CVE-2005-1918, CVE-2007-4131
                    Component::ParentDir => return Ok(false),

                    Component::Normal(part) => file_dst.push(part),
                }
            }
        }

        // Skip cases where only slashes or '.' parts were seen, because
        // this is effectively an empty filename.
        if *dst == *file_dst {
            return Ok(true);
        }

        // Skip entries without a parent (i.e. outside of FS root)
        let parent = match file_dst.parent() {
            Some(p) => p,
            None => return Ok(false),
        };

        self.ensure_dir_created(&dst, parent)
            .map_err(|e| TarError::new(format!("failed to create `{}`", parent.display()), e))?;

        let target = self.validate_inside_dst(&dst, parent)?;

        self.unpack(Some(&target), &file_dst)
            .map_err(|e| TarError::new(format!("failed to unpack `{}`", file_dst.display()), e))?;

        Ok(true)
    }

    /// Unpack as destination directory `dst`.
    fn unpack_dir(&mut self, dst: &Path) -> io::Result<()> {
        // If the directory already exists just let it slide
        fs::create_dir(dst).or_else(|err| {
            if err.kind() == ErrorKind::AlreadyExists {
                let prev = fs::metadata(dst);
                if prev.map(|m| m.is_dir()).unwrap_or(false) {
                    return Ok(());
                }
            }
            Err(Error::new(
                err.kind(),
                format!("{} when creating dir {}", err, dst.display()),
            ))
        })
    }

    /// Returns access to the header of this entry in the archive.
    fn unpack(&mut self, target_base: Option<&Path>, dst: &Path) -> io::Result<Unpacked> {
        fn set_perms_ownerships(
            dst: &Path,
            f: Option<&mut std::fs::File>,
            header: &Header,
            perms: bool,
            ownerships: bool,
        ) -> io::Result<()> {
            // ownerships need to be set first to avoid stripping SUID bits in the permissions ...
            if ownerships {
                set_ownerships(dst, &f, header.uid()?, header.gid()?)?;
            }
            // ... then set permissions, SUID bits set here is kept
            if let Ok(mode) = header.mode() {
                set_perms(dst, f, mode, perms)?;
            }

            Ok(())
        }

        fn get_mtime(header: &Header) -> Option<FileTime> {
            header.mtime().ok().map(|mtime| {
                // For some more information on this see the comments in
                // `Header::fill_platform_from`, but the general idea is that
                // we're trying to avoid 0-mtime files coming out of archives
                // since some tools don't ingest them well. Perhaps one day
                // when Cargo stops working with 0-mtime archives we can remove
                // this.
                let mtime = if mtime == 0 { 1 } else { mtime };
                FileTime::from_unix_time(mtime as i64, 0)
            })
        }

        let kind = self.header.entry_type();

        if kind.is_dir() {
            self.unpack_dir(dst)?;
            set_perms_ownerships(
                dst,
                None,
                &self.header,
                self.preserve_permissions,
                self.preserve_ownerships,
            )?;
            return Ok(Unpacked::__Nonexhaustive);
        } else if kind.is_hard_link() || kind.is_symlink() {
            let src = match self.link_name()? {
                Some(name) => name,
                None => {
                    return Err(other(&format!(
                        "hard link listed for {} but no link name found",
                        String::from_utf8_lossy(self.header.as_bytes())
                    )));
                }
            };

            if src.iter().count() == 0 {
                return Err(other(&format!(
                    "symlink destination for {} is empty",
                    String::from_utf8_lossy(self.header.as_bytes())
                )));
            }

            if kind.is_hard_link() {
                let link_src = match target_base {
                    // If we're unpacking within a directory then ensure that
                    // the destination of this hard link is both present and
                    // inside our own directory. This is needed because we want
                    // to make sure to not overwrite anything outside the root.
                    //
                    // Note that this logic is only needed for hard links
                    // currently. With symlinks the `validate_inside_dst` which
                    // happens before this method as part of `unpack_in` will
                    Some(ref p) => {
                        let link_src = p.join(src);
                        self.validate_inside_dst(p, &link_src)?;
                        link_src
                    }
                    None => src.into_owned(),
                };
                fs::hard_link(&link_src, dst).map_err(|err| {
                    Error::new(
                        err.kind(),
                        format!(
                            "{} when hard linking {} to {}",
                            err,
                            link_src.display(),
                            dst.display()
                        ),
                    )
                })?;
            } else {
                symlink(&src, dst)
                    .or_else(|err_io| {
                        if err_io.kind() == io::ErrorKind::AlreadyExists && self.overwrite {
                            // remove dest and try once more
                            std::fs::remove_file(dst).and_then(|()| symlink(&src, dst))
                        } else {
                            Err(err_io)
                        }
                    })
                    .map_err(|err| {
                        Error::new(
                            err.kind(),
                            format!(
                                "{} when symlinking {} to {}",
                                err,
                                src.display(),
                                dst.display()
                            ),
                        )
                    })?;
                if self.preserve_mtime {
                    if let Some(mtime) = get_mtime(&self.header) {
                        filetime::set_symlink_file_times(dst, mtime, mtime).map_err(|e| {
                            TarError::new(format!("failed to set mtime for `{}`", dst.display()), e)
                        })?;
                    }
                }
            }
            return Ok(Unpacked::__Nonexhaustive);

            fn symlink(src: &Path, dst: &Path) -> io::Result<()> {
                ::std::os::unix::fs::symlink(src, dst)
            }
        } else if kind.is_gnu_longname() || kind.is_gnu_longlink() {
            return Ok(Unpacked::__Nonexhaustive);
        };

        // Old BSD-tar compatibility.
        // Names that have a trailing slash should be treated as a directory.
        // Only applies to old headers.
        if self.header.as_ustar().is_none() && self.path_bytes().ends_with(b"/") {
            self.unpack_dir(dst)?;
            set_perms_ownerships(
                dst,
                None,
                &self.header,
                self.preserve_permissions,
                self.preserve_ownerships,
            )?;
            return Ok(Unpacked::__Nonexhaustive);
        }

        // Note the lack of `else` clause above. According to the FreeBSD
        // documentation:
        //
        // > A POSIX-compliant implementation must treat any unrecognized
        // > typeflag value as a regular file.
        //
        // As a result if we don't recognize the kind we just write out the file
        // as we would normally.

        // Ensure we write a new file rather than overwriting in-place which
        // is attackable; if an existing file is found unlink it.
        fn open(dst: &Path) -> io::Result<std::fs::File> {
            OpenOptions::new().write(true).create_new(true).open(dst)
        }
        let mut f = (|| -> io::Result<std::fs::File> {
            let mut f = open(dst).or_else(|err| {
                if err.kind() != ErrorKind::AlreadyExists {
                    Err(err)
                } else if self.overwrite {
                    match fs::remove_file(dst) {
                        Ok(()) => open(dst),
                        Err(ref e) if e.kind() == io::ErrorKind::NotFound => open(dst),
                        Err(e) => Err(e),
                    }
                } else {
                    Err(err)
                }
            })?;
            for io in self.data.drain(..) {
                match io {
                    EntryIo::Data(mut d) => {
                        let expected = d.limit();
                        if io::copy(&mut d, &mut f)? != expected {
                            return Err(other("failed to write entire file"));
                        }
                    }
                    EntryIo::Pad(d) => {
                        // TODO: checked cast to i64
                        let to = SeekFrom::Current(d.limit() as i64);
                        let size = f.seek(to)?;
                        f.set_len(size)?;
                    }
                }
            }
            Ok(f)
        })()
        .map_err(|e| {
            let header = self.header.path_bytes();
            TarError::new(
                format!(
                    "failed to unpack `{}` into `{}`",
                    String::from_utf8_lossy(&header),
                    dst.display()
                ),
                e,
            )
        })?;

        if self.preserve_mtime {
            if let Some(mtime) = get_mtime(&self.header) {
                filetime::set_file_handle_times(&f, Some(mtime), Some(mtime)).map_err(|e| {
                    TarError::new(format!("failed to set mtime for `{}`", dst.display()), e)
                })?;
            }
        }
        set_perms_ownerships(
            dst,
            Some(&mut f),
            &self.header,
            self.preserve_permissions,
            self.preserve_ownerships,
        )?;

        return Ok(Unpacked::File(f));

        fn set_ownerships(
            dst: &Path,
            f: &Option<&mut std::fs::File>,
            uid: u64,
            gid: u64,
        ) -> Result<(), TarError> {
            _set_ownerships(dst, f, uid, gid).map_err(|e| {
                TarError::new(
                    format!(
                        "failed to set ownerships to uid={:?}, gid={:?} \
                         for `{}`",
                        uid,
                        gid,
                        dst.display()
                    ),
                    e,
                )
            })
        }

        fn _set_ownerships(
            dst: &Path,
            f: &Option<&mut std::fs::File>,
            uid: u64,
            gid: u64,
        ) -> io::Result<()> {
            use std::os::unix::prelude::*;

            let uid: libc::uid_t = uid.try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::Other, format!("UID {} is too large!", uid))
            })?;
            let gid: libc::gid_t = gid.try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::Other, format!("GID {} is too large!", gid))
            })?;
            match f {
                Some(f) => unsafe {
                    let fd = f.as_raw_fd();
                    if libc::fchown(fd, uid, gid) != 0 {
                        Err(io::Error::last_os_error())
                    } else {
                        Ok(())
                    }
                },
                None => unsafe {
                    let path = std::ffi::CString::new(dst.as_os_str().as_bytes()).map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("path contains null character: {:?}", e),
                        )
                    })?;
                    if libc::lchown(path.as_ptr(), uid, gid) != 0 {
                        Err(io::Error::last_os_error())
                    } else {
                        Ok(())
                    }
                },
            }
        }

        fn set_perms(
            dst: &Path,
            f: Option<&mut std::fs::File>,
            mode: u32,
            preserve: bool,
        ) -> io::Result<()> {
            use std::os::unix::prelude::*;

            let mode = if preserve { mode } else { mode & 0o777 };
            let perm = fs::Permissions::from_mode(mode as _);
            match f {
                Some(f) => f.set_permissions(perm),
                None => fs::set_permissions(dst, perm),
            }
        }
    }

    fn ensure_dir_created(&self, dst: &Path, dir: &Path) -> io::Result<()> {
        let mut ancestor = dir;
        let mut dirs_to_create = Vec::new();
        while ancestor.symlink_metadata().is_err() {
            dirs_to_create.push(ancestor);
            if let Some(parent) = ancestor.parent() {
                ancestor = parent;
            } else {
                break;
            }
        }
        for ancestor in dirs_to_create.into_iter().rev() {
            if let Some(parent) = ancestor.parent() {
                self.validate_inside_dst(dst, parent)?;
            }
            fs::create_dir_all(ancestor)?;
        }
        Ok(())
    }

    fn validate_inside_dst(&self, dst: &Path, file_dst: &Path) -> io::Result<PathBuf> {
        if !file_dst.starts_with(dst) {
            let err = TarError::new(
                format!(
                    "trying to unpack outside of destination path: {}",
                    dst.display()
                ),
                // TODO: use ErrorKind::InvalidInput here? (minor breaking change)
                Error::new(ErrorKind::Other, "Invalid argument"),
            );
            return Err(err.into());
        }
        Ok(dst.to_path_buf())
    }
}

impl<'a> Read for EntryFields<'a> {
    fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.data.get_mut(0).map(|io| io.read(into)) {
                Some(Ok(0)) => {
                    self.data.remove(0);
                }
                Some(r) => return r,
                None => return Ok(0),
            }
        }
    }
}

impl<'a> Read for EntryIo<'a> {
    fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
        match *self {
            EntryIo::Pad(ref mut io) => io.read(into),
            EntryIo::Data(ref mut io) => io.read(into),
        }
    }
}
