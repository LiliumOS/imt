#[cfg(feature = "tar")]
use std::io::{Seek, Write};
use std::{
    io::{ErrorKind, Read},
    iter::FusedIterator,
};

use bincode::error::{DecodeError, EncodeError};
use indexmap::IndexMap;

use crate::{config::format_config, file::File};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Path(pub Vec<String>);

impl Path {
    pub fn starts_with(&self, other: &Path) -> bool {
        if self.0.len() < other.0.len() {
            return false;
        }

        let l = other.0.len();

        &self.0[..l] == &other.0
    }
}

impl core::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sep = "";

        for elem in &self.0 {
            f.write_str(sep)?;
            sep = "::";
            f.write_str(elem)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Bundle {
    files: IndexMap<Path, File>,
}

impl core::fmt::Debug for Bundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.files.fmt(f)
    }
}

impl Bundle {
    pub fn create() -> Self {
        Self {
            files: IndexMap::new(),
        }
    }

    pub fn add_file(&mut self, path: Path, file: File) {
        self.files.insert(path, file);
    }

    pub fn parse_file<R: Read>(&mut self, path: Path, mut file: R) -> Result<(), DecodeError> {
        let file = bincode::decode_from_std_read(&mut file, format_config())?;

        self.add_file(path, file);

        Ok(())
    }

    pub fn add_files<I: IntoIterator<Item = (Path, File)>>(&mut self, files: I) {
        for (path, file) in files {
            self.add_file(path, file);
        }
    }

    pub fn try_add_files<E, I: IntoIterator<Item = Result<(Path, File), E>>>(
        &mut self,
        files: I,
    ) -> Result<(), E> {
        for item in files {
            let (path, file) = item?;
            self.add_file(path, file);
        }
        Ok(())
    }

    pub fn parse_files<R: Read, I: IntoIterator<Item = Result<(Path, R), std::io::Error>>>(
        &mut self,
        files: I,
    ) -> Result<(), DecodeError> {
        for item in files {
            let (path, reader) = item.map_err(|e| DecodeError::Io {
                inner: e,
                additional: 0,
            })?;

            self.parse_file(path, reader)?;
        }
        Ok(())
    }

    pub fn write_files<
        F: for<'a> FnMut(
            &[String],
            &'a mut (
                        dyn for<'b> FnMut(&'b mut (dyn std::io::Write + 'b)) -> std::io::Result<()>
                            + 'a
                    ),
        ) -> std::io::Result<()>,
    >(
        &self,
        prefix: &Path,
        mut supplier: F,
    ) -> std::io::Result<()> {
        for (path, file) in &self.files {
            let (check, without_prefix) = path.0.split_at(prefix.0.len());
            assert_eq!(check, &prefix.0);
            supplier(without_prefix, &mut |mut w| {
                bincode::encode_into_std_write(file, &mut w, format_config())
                    .map_err(|e| match e {
                        EncodeError::Io { inner, .. } => inner,
                        e => std::io::Error::new(ErrorKind::InvalidInput, e),
                    })
                    .map(|_| ())
            })?;
        }
        Ok(())
    }

    #[cfg(feature = "tar")]
    pub fn parse_tar<R: Read>(&mut self, prefix: Path, tar: R) -> Result<(), DecodeError> {
        use tar::Archive;

        let mut archive = Archive::new(tar);

        self.parse_files(
            archive
                .entries()
                .map_err(|e| DecodeError::Io {
                    inner: e,
                    additional: 0,
                })?
                .filter_map(|e| {
                    let entry = match e {
                        Ok(e) => e,
                        Err(e) => return Some(Err(e)),
                    };

                    let name = match entry.path() {
                        Ok(name) => name,
                        Err(e) => {
                            return Some(Err(e));
                        }
                    };

                    let path = name.as_os_str().to_str()?;

                    let path = path.strip_suffix(".imt")?;

                    let mut gpath = prefix.0.clone();

                    gpath.extend(
                        path.split(std::path::MAIN_SEPARATOR)
                            .map(String::from)
                            .collect::<Vec<_>>(),
                    );

                    Some(Ok((Path(gpath), entry)))
                }),
        )
    }

    #[cfg(feature = "tar")]
    pub fn write_tar<W: Write + Seek>(&mut self, prefix: &Path, tar: W) -> std::io::Result<()> {
        use tar::Builder;

        let mut archive = Builder::new(tar);

        self.write_files(&prefix, |path, writer_cb| {
            use std::path::PathBuf;

            use tar::Header;

            let mut path = path.iter().collect::<PathBuf>();
            path.add_extension("imt");

            let mut header = Header::new_gnu();

            writer_cb(&mut archive.append_writer(&mut header, path)?)
        })
    }

    pub fn get(&self, path: &Path) -> Option<&File> {
        self.files.get(path)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter(self.files.iter())
    }
}

impl IntoIterator for Bundle {
    type Item = (Path, File);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.files.into_iter())
    }
}

impl<'a> IntoIterator for &'a Bundle {
    type Item = (&'a Path, &'a File);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a>(indexmap::map::Iter<'a, Path, File>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a Path, &'a File);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

pub struct IntoIter(indexmap::map::IntoIter<Path, File>);

impl Iterator for IntoIter {
    type Item = (Path, File);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for IntoIter {}
