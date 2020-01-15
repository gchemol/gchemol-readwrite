// mods

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*mods][mods:1]]
mod cif;
mod mol2;
mod pdb;
mod sdf;
mod xyz;

mod gaussian_input;
mod vasp_input;
// mods:1 ends here

// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use guts::fs::*;
// imports:1 ends here

// exports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*exports][exports:1]]
pub(self) use gchemol_core::{Atom, AtomKind, Bond, BondKind, Lattice, Molecule};
pub(self) use guts::prelude::*;

pub(self) mod parser {
    pub use text_parser::parsers::*;
    pub use text_parser::{Bunches, TextReader};
}
// exports:1 ends here

// traits

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*traits][traits:1]]
pub(self) trait ChemicalFile: ParseMolecule {
    /// Chemical file type.
    fn ftype(&self) -> &str;

    /// Supported file types in file extension, for example:
    /// [".xyz", ".mol2"]
    fn possible_extensions(&self) -> Vec<&str>;

    /// Formatted representation of a Molecule.
    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        unimplemented!()
    }

    /// Determine if file `filename` is parable according to its supported file
    /// extensions
    fn parsable(&self, filename: &Path) -> bool {
        let filename = format!("{}", filename.display());
        let filename = filename.to_lowercase();
        for s in self.possible_extensions() {
            if filename.ends_with(&s.to_lowercase()) {
                return true;
            }
        }

        false
    }

    // /// Save multiple molecules into a file
    // fn write(&self, filename: &Path, mols: &[Molecule]) -> Result<()> {
    //     use crate::io::prelude::ToFile;

    //     let txt = self.format(mols)?;
    //     &txt.to_file(filename)?;
    //     Ok(())
    // }

    /// print a brief description about a chemical file format
    fn describe(&self) {
        println!(
            "filetype: {:?}, possible extensions: {:?}",
            self.ftype(),
            self.possible_extensions()
        );
    }
}
// traits:1 ends here

// adhoc

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*adhoc][adhoc:1]]
use self::parser::*;

type FileReader = BufReader<File>;

/// Parse a molecule from string slice.
pub(self) trait ParseMolecule {
    /// parse molecule from string slice in a part of chemical file.
    fn parse_molecule(&self, input: &str) -> Result<Molecule>;

    /// Mark the start position for a bunch of lines containing a single
    /// molecule in a large text file.
    fn mark_bunch(&self) -> Box<Fn(&str) -> bool>;
}

/// Read molecules in specific chemical file format.
pub(super) fn read_chemical_file<P: AsRef<Path>>(path: P, fmt: Option<&str>) -> impl Iterator<Item = Result<Molecule>> {
    let cf = guess_chemical_file_format(path.as_ref(), fmt);

    let mut parsed_mols = None;
    if let Some(parser) = cf {
        match TextReader::from_path(path) {
            Ok(reader) => {
                // string buffer
                let mut part = String::new();
                let bunches = reader.bunches(parser.mark_bunch());
                let mols = bunches.map(move |lines| {
                    for line in lines {
                        part += &line;
                        part += "\n";
                    }
                    let mol = parser.parse_molecule(&part);

                    // reset string buffer
                    part.clear();
                    mol
                });
                parsed_mols = Some(mols);
            }
            Err(e) => {
                error!("read file error: {}", e);
            }
        }
    } else {
        error!("no available parser!");
    }

    parsed_mols.into_iter().flatten()
}
// adhoc:1 ends here

// backends

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*backends][backends:1]]
macro_rules! avail_parsers {
    () => {
        vec![
            Box::new(self::xyz::XyzFile()),
            Box::new(self::xyz::PlainXyzFile()),
            // Box::new(self::mol2::Mol2File()),
            // Box::new(self::sdf::SdfFile()),
            // Box::new(self::vasp::PoscarFile()),
            // Box::new(self::cif::CifFile()),
            // Box::new(self::pdb::PdbFile()),
        ]
    };
}

/// guess the most appropriate file format by its file extensions
pub(self) fn guess_chemical_file_format(filename: &Path, fmt: Option<&str>) -> Option<Box<ChemicalFile>> {
    let backends: Vec<Box<ChemicalFile>> = avail_parsers!();
    // 1. by file type
    if let Some(fmt) = fmt {
        for x in backends {
            if x.ftype() == fmt.to_lowercase() {
                return Some(x);
            }
        }
    // 2. or by file extension
    } else {
        for x in backends {
            if x.parsable(filename) {
                return Some(x);
            }
        }
    }

    // 3. return None if no suitable backend
    None
}

/// description of all backends
pub fn describe_backends() {
    let backends: Vec<Box<ChemicalFile>> = avail_parsers!();

    for cf in backends {
        cf.describe();
    }
}
// backends:1 ends here
