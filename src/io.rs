// header

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*header][header:1]]
//===============================================================================#
//   DESCRIPTION:  basic read & write support for molecular file
//
//       OPTIONS:  ---
//  REQUIREMENTS:  ---
//         NOTES:  ---
//        AUTHOR:  Wenping Guo <ybyygu@gmail.com>
//       LICENCE:  GPL version 3
//       CREATED:  <2018-04-11 Wed 15:42>
//       UPDATED:  <2020-01-13 Mon 12:02>
//===============================================================================#
// header:1 ends here

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

// api

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*api][api:1]]
/// Read an iterator over `Molecule` from file.
/// file format will be determined according to the path
// https://stackoverflow.com/questions/26368288/how-do-i-stop-iteration-and-return-an-error-when-iteratormap-returns-a-result
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<Molecule>> {
    // let path = path.as_ref();
    // FileOptions::new().read(path)
    todo!()
}

/// Write molecules into file
/// file format will be determined according to the path
pub fn write<P: AsRef<Path>>(path: P, mols: &[Molecule]) -> Result<()> {
    // let path = path.as_ref();
    // FileOptions::new().write(path, mols)
    todo!()
}
// api:1 ends here

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
        // let path = path.as_ref();
        // let cf = guess_chemfile_from_filename(path)?;
        // let mut mols = cf.parse(path)?;
        // mols.pop()
        //     .ok_or(format_err!("No molecule: {:?}", path.display()))
        todo!()
    }
}

impl ToFile for Molecule {
    /// Save molecule to an external file
    fn to_file<T: AsRef<Path>>(&self, filename: T) -> Result<()> {
        // let filename = filename.as_ref();
        // let cf = guess_chemfile(&filename.display().to_string(), None)
        //     .ok_or(format_err!("not supported file format: {:?}", filename))?;
        // let t = cf.format_molecule(&self)?;

        // t.to_file(filename)?;

        // Ok(())
        todo!()
    }
}
// molecule:1 ends here
