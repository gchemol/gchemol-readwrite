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
    fn parse_from<S: AsRef<str>, T: AsRef<str>>(s: S, fmt: T) -> Result<Molecule>;
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

// api

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*api][api:1]]
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

/// Read molecules in specific chemical file format.
pub fn read_format<P: AsRef<Path>>(path: P, fmt: &str) -> Result<impl Iterator<Item = Molecule>> {
    let mols = crate::formats::read_chemical_file(path, Some(fmt))?;
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
// api:1 ends here
