// mods

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*mods][mods:1]]
mod xyz;
// mods:1 ends here

// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use guts::fs::*;
use text_parser::IResult;
// imports:1 ends here

// exports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*exports][exports:1]]
pub(crate) use gchemol_core::{Atom, Molecule};
pub(crate) use guts::prelude::*;
// exports:1 ends here

// trait

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*trait][trait:1]]
pub trait ChemicalFile {
    /// parse molecules from chemical file.
    fn parse<P: AsRef<Path>>(&self, p: P) -> Result<Vec<Molecule>> {
        let s = read_file(p)?;
        self.parse_molecules(&s)
    }

    /// print a brief description about a chemical file format
    fn describe(&self) {
        todo!()
    }

    /// Chemical file type.
    fn ftype(&self) -> &str;

    /// Supported file types in file extension, for example:
    /// [".xyz", ".mol2"]
    fn possible_extensions(&self) -> Vec<&str>;

    /// Formatted representation of a Molecule.
    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        unimplemented!()
    }

    /// Parse a single molecule from string slice using facilities.
    fn parse_molecules(&self, chunk: &str) -> Result<Vec<Molecule>> {
        unimplemented!()
    }
}
// trait:1 ends here
