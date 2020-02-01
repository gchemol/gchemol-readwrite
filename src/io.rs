// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use guts::fs::*;
use guts::prelude::*;

use gchemol_core::Molecule;
// imports:1 ends here

// traits

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*traits][traits:1]]
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
    fn parse_from<R: Read, S: AsRef<str>>(s: R, fmt: S) -> Result<Molecule>;

    fn from_str<S: AsRef<str>>(s: &str, fmt: S) -> Result<Molecule> {
        let f = std::io::Cursor::new(s.as_bytes());
        Self::parse_from(f, fmt)
    }
}
// traits:1 ends here

// file

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*file][file:1]]
impl FromFile for String {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        guts::fs::read_file(path)
    }
}

impl ToFile for str {
    fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        guts::fs::write_to_file(path, &self)
    }
}
// file:1 ends here

// molecule

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*molecule][molecule:1]]
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

// string

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*string][string:1]]
impl StringIO for Molecule {
    /// Format molecule as string in specific molecular file format. Return
    /// error if cannot format molecule in `fmt`.
    fn format_as<S: AsRef<str>>(&self, fmt: S) -> Result<String> {
        let fmt = fmt.as_ref();
        crate::formats::format_as_chemical_file(&self, fmt)
    }

    /// construct molecule from string in specific molecular file format.
    fn parse_from<R: Read, S: AsRef<str>>(s: R, fmt: S) -> Result<Molecule> {
        read_from(s, &fmt)?
            .last()
            .ok_or(format_err!("Parse molecule failure in format: {}", fmt.as_ref()))
    }
}
// string:1 ends here

// functions

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*functions][functions:1]]
/// Read an iterator over `Molecule` from file.
/// file format will be determined according to the path
pub fn read<P: AsRef<Path>>(path: P) -> Result<impl Iterator<Item = Molecule>> {
    let mols = crate::formats::read_chemical_file(path, None)?;
    Ok(mols.filter_map(|parsed| match parsed {
        Ok(mol) => Some(mol),
        Err(e) => {
            eprintln!("found parsing error: {:?}", e);
            None
        }
    }))
}

// https://stackoverflow.com/questions/26368288/how-do-i-stop-iteration-and-return-an-error-when-iteratormap-returns-a-result
/// Read all molecules into a Vec from `path`.
pub fn read_all<P: AsRef<Path>>(path: P) -> Result<Vec<Molecule>> {
    let mols: Vec<_> = read(path)?.collect();
    Ok(mols)
}

/// Read molecules from readable source in specific chemical file format.
pub fn read_from<R: Read, S: AsRef<str>>(mut source: R, fmt: S) -> Result<impl Iterator<Item = Molecule>> {
    // FIXME: adhoc hacking
    use tempfile::tempdir;

    // read stream and write it into a temporary file
    let mut buf = String::new();
    let _ = source.read_to_string(&mut buf)?;
    let dir = tempdir()?;
    let path = dir.path().join("test");
    guts::fs::write_to_file(&path, &buf)?;

    let mols = crate::formats::read_chemical_file(path, Some(fmt.as_ref()))?;
    Ok(mols.filter_map(|parsed| match parsed {
        Ok(mol) => Some(mol),
        Err(e) => {
            eprintln!("found parsing error: {:?}", e);
            None
        }
    }))
}

/// Write molecules into path. File format will be determined according to the
/// path
pub fn write<'a, P: AsRef<Path>>(path: P, mols: impl IntoIterator<Item = &'a Molecule>) -> Result<()> {
    crate::formats::write_chemical_file(path, mols, None)
}

/// Write molecules into path in specific chemical file format.
pub fn write_format<'a, P: AsRef<Path>>(
    path: P,
    mols: impl IntoIterator<Item = &'a Molecule>,
    fmt: &str,
) -> Result<()> {
    crate::formats::write_chemical_file(path, mols, Some(fmt))
}
// functions:1 ends here
