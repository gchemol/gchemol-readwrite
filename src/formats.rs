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

type FileReader = BufReader<File>;
// imports:1 ends here

// exports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*exports][exports:1]]
pub(self) use gchemol_core::{Atom, AtomKind, Bond, BondKind, Lattice, Molecule};
pub(self) use guts::prelude::*;

pub(self) mod parser {
    pub use text_parser::parsers::*;
    pub use text_parser::{Bunches, Partition, Partitions, ReadContext, TextReader};
}
// exports:1 ends here

// traits

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*traits][traits:1]]
pub(self) trait ChemicalFile: ParseMolecule + Partition {
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

    /// print a brief description about a chemical file format
    fn describe(&self) {
        println!(
            "filetype: {:?}, possible extensions: {:?}",
            self.ftype(),
            self.possible_extensions()
        );
    }
}

/// Parse a molecule from string slice.
pub(self) trait ParseMolecule {
    /// parse molecule from string slice in a part of chemical file.
    fn parse_molecule(&self, input: &str) -> Result<Molecule>;

    /// Mark the start position for a bunch of lines containing a single
    /// molecule in a large text file.
    fn mark_bunch(&self) -> Box<Fn(&str) -> bool> {
        todo!()
    }

    /// Seek the position of a specific line.
    fn seek_line(&self) -> Option<Box<Fn(&str) -> bool>> {
        None
    }
}
// traits:1 ends here

// write

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*write][write:1]]
/// Write molecules into path in specific chemical file format.
pub(super) fn write_chemical_file<'a, P: AsRef<Path>>(
    path: P,
    mols: impl IntoIterator<Item = &'a Molecule>,
    fmt: Option<&str>,
) -> Result<()> {
    use std::fs::File;

    let path = path.as_ref();
    if let Some(cf) = guess_chemical_file_format(path, fmt) {
        let mut fp = File::create(path).with_context(|| format!("Failed to create file: {:?}", path))?;

        for mol in mols {
            let s = cf.format_molecule(mol)?;
            fp.write(s.as_bytes());
        }
    }

    Ok(())
}
// write:1 ends here

// parse iter

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*parse iter][parse iter:1]]
use text_parser::*;

/// Parse many molecules
pub(self) trait ParseMoleculeIter {
    type IterMolecule: Iterator<Item = Result<Molecule>>;

    /// Return an iterator over parsed molecules from reader `r`.
    fn parse_molecules(&self, r: TextReader<FileReader>) -> Self::IterMolecule;

    /// Return an iterator over parsed molecules from path `p`.
    fn parse_molecules_from_path<P: AsRef<Path>>(&self, p: P) -> Result<Self::IterMolecule> {
        let reader = TextReader::from_path(p)?;
        Ok(self.parse_molecules(reader))
    }
}

// cannot use dynamic dispatching
impl<T> ParseMoleculeIter for T
where
    T: ChemicalFile + Copy,
{
    type IterMolecule = ParsedMolecules<Self>;

    /// Return an iterator over parsed molecules from reader `r`.
    fn parse_molecules(&self, mut r: TextReader<FileReader>) -> Self::IterMolecule {
        if let Some(skip) = self.seek_line() {
            if r.seek_line(skip).is_err() {
                error!("skip reading lines error");
            }
        }
        ParsedMolecules {
            partitions: r.partitions(*self),
            parser: *self,
        }
    }
}

pub(self) struct ParsedMolecules<T>
where
    T: ChemicalFile,
{
    partitions: Partitions<FileReader, T>,
    parser: T,
}

impl<T> Iterator for ParsedMolecules<T>
where
    T: ChemicalFile,
{
    type Item = Result<Molecule>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(part) = self.partitions.next() {
            let parsed = self.parser.parse_molecule(&part);
            Some(parsed)
        } else {
            None
        }
    }
}
// parse iter:1 ends here

// read/adhoc2

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*read/adhoc2][read/adhoc2:1]]
pub(super) fn read_chemical_file<P: AsRef<Path>>(path: P, fmt: Option<&str>) -> impl Iterator<Item = Result<Molecule>> {
    use self::cif::*;
    use self::mol2::*;
    use self::xyz::*;

    let path = path.as_ref();

    let x1 = if XyzFile().parsable(path) {
        XyzFile().parse_molecules_from_path(path).ok()
    } else {
        None
    };

    let x2 = if PlainXyzFile().parsable(path) {
        PlainXyzFile().parse_molecules_from_path(path).ok()
    } else {
        None
    };

    let x3 = if Mol2File().parsable(path) {
        Mol2File().parse_molecules_from_path(path).ok()
    } else {
        None
    };

    let x4 = if CifFile().parsable(path) {
        CifFile().parse_molecules_from_path(path).ok()
    } else {
        None
    };

    x1.into_iter()
        .flatten()
        .chain(x2.into_iter().flatten())
        .chain(x3.into_iter().flatten())
        .chain(x4.into_iter().flatten())
}
// read/adhoc2:1 ends here

// backends

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*backends][backends:1]]
macro_rules! avail_parsers {
    () => {
        vec![
            Box::new(self::xyz::XyzFile()),
            Box::new(self::xyz::PlainXyzFile()),
            Box::new(self::mol2::Mol2File()),
            Box::new(self::cif::CifFile()),
            Box::new(self::vasp_input::PoscarFile()),
            // Box::new(self::sdf::SdfFile()),
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
