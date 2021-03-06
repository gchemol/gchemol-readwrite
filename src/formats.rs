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
use gut::fs::*;

type FileReader = BufReader<File>;
// imports:1 ends here

// exports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*exports][exports:1]]
pub(self) use gchemol_core::{Atom, AtomKind, Bond, BondKind, Lattice, Molecule, Vector3f};
pub(self) use gut::prelude::*;

pub(self) mod parser {
    pub use gchemol_parser::parsers::*;
    pub use gchemol_parser::partition::{Partitions, Preceded, ReadAction, ReadContext, ReadPart, Terminated};
    pub use gchemol_parser::TextReader;
}
// exports:1 ends here

// chemical file

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*chemical file][chemical file:1]]
pub(self) trait ChemicalFile: ParseMolecule + ReadPart {
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

    /// Hook before start reading.
    fn pre_read_hook<R: BufRead + Seek>(&self, r: TextReader<R>) -> TextReader<R>
    where
        Self: Sized,
    {
        r
    }
}
// chemical file:1 ends here

// parse iter

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*parse iter][parse iter:1]]
use gchemol_parser::{partition::Partitions, partition::ReadPart, TextReader};

/// Parse many molecules
pub(self) trait ParseMoleculeIter<R>
where
    R: BufRead + Seek,
{
    type IterMolecule: Iterator<Item = Result<Molecule>>;

    /// Return an iterator over parsed molecules from reader `r`.
    fn parse_molecules(&self, r: TextReader<R>) -> Self::IterMolecule;
}

// cannot use dynamic dispatching
impl<R, T> ParseMoleculeIter<R> for T
where
    T: ChemicalFile + Copy,
    R: BufRead + Seek,
{
    type IterMolecule = ParsedMolecules<R, Self>;

    /// Return an iterator over parsed molecules from reader `r`.
    fn parse_molecules(&self, r: TextReader<R>) -> Self::IterMolecule {
        // apply reading hook
        let mut r = self.pre_read_hook(r);
        ParsedMolecules {
            partitions: r.partitions(*self),
            parser: *self,
        }
    }
}

pub(self) struct ParsedMolecules<R, T>
where
    R: BufRead + Seek,
    T: ChemicalFile,
{
    partitions: Partitions<R, T>,
    parser: T,
}

impl<R, T> Iterator for ParsedMolecules<R, T>
where
    R: BufRead + Seek,
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

// read chemfile

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*read chemfile][read chemfile:1]]
use self::cif::CifFile;
use self::gaussian_input::GaussianInputFile;
use self::mol2::Mol2File;
use self::pdb::PdbFile;
use self::sdf::SdfFile;
use self::vasp_input::PoscarFile;
use self::xyz::PlainXyzFile;
use self::xyz::XyzFile;

pub(super) struct ChemicalFileParser(pub String);

impl ChemicalFileParser {
    pub fn new(fmt: &str) -> Self {
        Self(fmt.to_owned())
    }

    pub fn guess_from_path(path: &Path) -> Option<Self> {
        guess_chemical_file_format_from_path(path).map(move |cf| Self::new(cf.ftype()))
    }

    pub fn guess(path: &Path, fmt: Option<&str>) -> Option<Self> {
        guess_chemical_file_format(path, fmt).map(|cf| Self::new(cf.ftype()))
    }

    pub fn parse_molecules_from<R>(&self, r: TextReader<R>) -> Result<impl Iterator<Item = Molecule>>
    where
        R: BufRead + Seek,
    {
        let mut p1 = None;
        let mut p2 = None;
        let mut p3 = None;
        let mut p4 = None;
        let mut p5 = None;
        let mut p6 = None;
        let mut p7 = None;
        let mut p8 = None;

        macro_rules! cf_parse {
            ($cf:expr, $pn:expr) => {
                $pn = Some($cf().parse_molecules(r));
            };
        }

        match self.0.as_str() {
            "text/xyz" => cf_parse!(XyzFile, p1),
            "text/pxyz" => cf_parse!(PlainXyzFile, p2),
            "text/mol2" => cf_parse!(Mol2File, p3),
            "text/cif" => cf_parse!(CifFile, p4),
            "text/sdf" => cf_parse!(SdfFile, p5),
            "text/pdb" => cf_parse!(PdbFile, p6),
            "vasp/input" => cf_parse!(PoscarFile, p7),
            "gaussian/input" => cf_parse!(GaussianInputFile, p8),
            _ => bail!("No available parser found"),
        }
        Ok(p1
            .into_iter()
            .flatten()
            .chain(p2.into_iter().flatten())
            .chain(p3.into_iter().flatten())
            .chain(p4.into_iter().flatten())
            .chain(p5.into_iter().flatten())
            .chain(p6.into_iter().flatten())
            .chain(p7.into_iter().flatten())
            .chain(p8.into_iter().flatten())
            .filter_map(|parsed| match parsed {
                Ok(mol) => Some(mol),
                Err(e) => {
                    eprintln!("found parsing error: {:?}", e);
                    None
                }
            }))
    }

    pub fn parse_molecules(&self, path: &Path) -> Result<impl Iterator<Item = Molecule>> {
        let r = TextReader::from_path(path).context("Parse molecules from path failed")?;
        self.parse_molecules_from(r)
    }
}
// read chemfile:1 ends here

// write chemifile

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*write chemifile][write chemifile:1]]
/// Write molecules into path in specific chemical file format.
pub(super) fn write_chemical_file<'a>(
    path: &Path,
    mols: impl IntoIterator<Item = &'a Molecule>,
    fmt: Option<&str>,
) -> Result<()> {
    if let Some(cf) = guess_chemical_file_format(path, fmt) {
        let mut fp = File::create(path).with_context(|| format!("Failed to create file: {:?}", path))?;
        for mol in mols {
            let s = cf.format_molecule(mol)?;
            fp.write(s.as_bytes());
        }
    } else {
        bail!("No suitable chemical file format found for {:?}", path);
    }

    Ok(())
}

/// Return formatted representation of molecule in specific chemical file
/// format.
pub(super) fn format_as_chemical_file(mol: &Molecule, fmt: &str) -> Result<String> {
    if let Some(cf) = guess_chemical_file_format_from_ftype(fmt) {
        return cf.format_molecule(mol);
    }
    bail!("No suitable chemical file format found for {:}", fmt);
}
// write chemifile:1 ends here

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
            Box::new(self::gaussian_input::GaussianInputFile()),
            Box::new(self::sdf::SdfFile()),
            Box::new(self::pdb::PdbFile()),
        ]
    };
}
/// guess the most appropriate file format by file type
fn guess_chemical_file_format_from_ftype(fmt: &str) -> Option<Box<dyn ChemicalFile>> {
    let backends: Vec<Box<dyn ChemicalFile>> = avail_parsers!();
    for x in backends {
        if x.ftype() == fmt.to_lowercase() {
            return Some(x);
        }
    }
    // no suitable backend
    None
}

/// guess the most appropriate file format by file path extensions
fn guess_chemical_file_format_from_path(filename: &Path) -> Option<Box<dyn ChemicalFile>> {
    let backends: Vec<Box<dyn ChemicalFile>> = avail_parsers!();
    for x in backends {
        if x.parsable(filename) {
            return Some(x);
        }
    }
    // no suitable backend
    None
}

/// guess the most appropriate file format by file path extensions
fn guess_chemical_file_format(filename: &Path, fmt: Option<&str>) -> Option<Box<dyn ChemicalFile>> {
    fmt.and_then(|fmt| guess_chemical_file_format_from_ftype(fmt))
        .or_else(|| guess_chemical_file_format_from_path(filename))
}

/// description of all backends
pub fn describe_backends() {
    let backends: Vec<Box<dyn ChemicalFile>> = avail_parsers!();

    for cf in backends {
        cf.describe();
    }
}

#[test]
fn test_backends() {
    let f = "/tmp/test.xyz";
    let cf = guess_chemical_file_format(f.as_ref(), None).expect("guess xyz");
    assert_eq!(cf.ftype(), "text/xyz");

    let f = "/tmp/test";
    let cf = guess_chemical_file_format(f.as_ref(), Some("text/xyz")).expect("guess xyz ftype");
    assert_eq!(cf.ftype(), "text/xyz");

    let f = "/tmp/test.poscar";
    let cf = guess_chemical_file_format(f.as_ref(), None).expect("guess xyz ftype");
    assert_eq!(cf.ftype(), "vasp/input");
}
// backends:1 ends here
