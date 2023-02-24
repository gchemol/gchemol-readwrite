// [[file:../gchemol-readwrite.note::*imports][imports:1]]
use gut::fs::*;
use gut::prelude::*;

use gchemol_core::Molecule;
// imports:1 ends here

// [[file:../gchemol-readwrite.note::*traits][traits:1]]
pub trait FromFile: Sized {
    /// Return content of text file in string.
    ///
    /// Do not use this to read large file.
    ///
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
}

pub trait ToFile {
    /// Write string content to an external file.
    ///
    /// _Note:_ Replaces the current file content if the file already exists.
    ///
    fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

pub trait StringIO {
    /// Format molecule as string in specific `fmt`.
    fn format_as<S: AsRef<str>>(&self, fmt: S) -> Result<String>;

    /// Parse molecule from string in specific `fmt`.
    fn parse_from<R: Read + Seek, S: AsRef<str>>(s: R, fmt: S) -> Result<Molecule>;

    fn from_str<S: AsRef<str>>(s: &str, fmt: S) -> Result<Molecule> {
        let f = std::io::Cursor::new(s.as_bytes());
        Self::parse_from(f, fmt)
    }
}
// traits:1 ends here

// [[file:../gchemol-readwrite.note::*file][file:1]]
impl FromFile for String {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        gut::fs::read_file(path)
    }
}

impl ToFile for str {
    fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        gut::fs::write_to_file(path, &self)
    }
}
// file:1 ends here

// [[file:../gchemol-readwrite.note::*molecule][molecule:1]]
impl FromFile for Molecule {
    /// Construct molecule from external text file
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        if let Some(mol) = read(path)?.last() {
            return Ok(mol);
        }
        bail!("No molecule found!");
    }
}

impl ToFile for Molecule {
    /// Save molecule to an external file
    fn to_file<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        write(path, vec![self])
    }
}
// molecule:1 ends here

// [[file:../gchemol-readwrite.note::*string][string:1]]
impl StringIO for Molecule {
    /// Format molecule as string in specific molecular file format. Return
    /// error if cannot format molecule in `fmt`.
    fn format_as<S: AsRef<str>>(&self, fmt: S) -> Result<String> {
        let fmt = fmt.as_ref();
        crate::formats::format_as_chemical_file(&self, fmt)
    }

    /// construct molecule from string in specific molecular file format.
    fn parse_from<R: Read + Seek, S: AsRef<str>>(s: R, fmt: S) -> Result<Molecule> {
        read_from(s, &fmt)?
            .last()
            .ok_or(format_err!("Parse molecule failure in format: {}", fmt.as_ref()))
    }
}
// string:1 ends here

// [[file:../gchemol-readwrite.note::d500136e][d500136e]]
mod find {
    use super::*;

    use walkdir::{DirEntry, WalkDir};

    fn is_hidden(entry: &DirEntry) -> bool {
        // allow walk into "." but not ".foo"
        entry.file_name().to_str().map(|s| s.len() > 1 && s.starts_with(".")).unwrap_or(false)
    }

    // regular file name matching `pattern`
    fn matching(pattern: &str, entry: Option<DirEntry>) -> Option<PathBuf> {
        let entry = entry?;
        if entry.file_type().is_file() {
            let rx = regex::Regex::new(pattern).ok()?;
            let s = entry.file_name().to_str()?;
            if rx.find(s).is_some() {
                return entry.into_path().into();
            }
        }
        None
    }

    /// Recursively find all files in `root` dir with given file name
    /// matching regex `pattern`
    pub fn find_files<'a>(pattern: &'a str, root: &Path) -> impl Iterator<Item = PathBuf> + 'a {
        WalkDir::new(root)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            // do not walk into hidden directories
            .filter_entry(|e| !is_hidden(e))
            .filter_map(|entry| matching(pattern, entry.ok()))
    }

    #[test]
    fn test_find() -> Result<()> {
        let root = "./tests/files";
        let files = find_files(r"\.xyz$", root.as_ref()).collect_vec();
        for file in files {
            assert!(file.to_string_lossy().ends_with(".xyz"));
        }

        Ok(())
    }
}
// d500136e ends here

// [[file:../gchemol-readwrite.note::80c178b0][80c178b0]]
pub use self::find::find_files;

/// Read an iterator over `Molecule` from file.
/// file format will be determined according to the path
pub fn read<P: AsRef<Path>>(path: P) -> Result<impl Iterator<Item = Molecule>> {
    let path = path.as_ref();
    crate::formats::ChemicalFileParser::guess_from_path(path)
        .ok_or(format_err!("No parser for path: {:?}", path))?
        .parse_molecules(path.as_ref())
}

// https://stackoverflow.com/questions/26368288/how-do-i-stop-iteration-and-return-an-error-when-iteratormap-returns-a-result
/// Read all molecules into a Vec from `path`.
pub fn read_all<P: AsRef<Path>>(path: P) -> Result<Vec<Molecule>> {
    let mols: Vec<_> = read(path)?.collect();
    Ok(mols)
}

/// Read molecules from readable source in specific chemical file format.
pub fn read_from<R: Read + Seek, S: AsRef<str>>(source: R, fmt: S) -> Result<impl Iterator<Item = Molecule>> {
    let cf = crate::formats::ChemicalFileParser::new(fmt.as_ref());
    let r = gchemol_parser::TextReader::new(source);
    cf.parse_molecules_from(r)
}

/// Guess chemical file format from `path`
pub fn guess_format_from_path(path: &Path) -> Option<String> {
    crate::formats::ChemicalFileParser::guess_format_from_path(path)
}

/// Write molecules into path. File format will be determined according to the
/// path
pub fn write<'a, P: AsRef<Path>>(path: P, mols: impl IntoIterator<Item = &'a Molecule>) -> Result<()> {
    crate::formats::write_chemical_file(path.as_ref(), mols, None)
}

/// Write molecules into path in specific chemical file format.
pub fn write_format<'a, P: AsRef<Path>>(path: P, mols: impl IntoIterator<Item = &'a Molecule>, fmt: &str) -> Result<()> {
    crate::formats::write_chemical_file(path.as_ref(), mols, Some(fmt))
}
// 80c178b0 ends here
