// [[file:../../gchemol-readwrite.note::1645b9f1][1645b9f1]]
//! https://wiki.openchemistry.org/Chemical_JSON
use gut::prelude::*;

use gchemol_core::{Atom, Lattice, Molecule};
// 1645b9f1 ends here

// [[file:../../gchemol-readwrite.note::635f7a3e][635f7a3e]]
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct CjsonMolecule {
    #[serde(rename = "chemical json")]
    cjson_version: usize,
    name: String,
    atoms: CjsonAtoms,
    properties: Value,
    inchi: Option<String>,
    formula: Option<String>,
    bonds: Option<CjsonBonds>,
    #[serde(rename = "unit cell")]
    unit_cell: Option<CjsonUnitCell>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CjsonBonds {
    connections: CjsonConnections,
    order: Vec<usize>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CjsonConnections {
    index: Vec<usize>,
    ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CjsonUnitCell {
    a: f64,
    b: f64,
    c: f64,
    alpha: f64,
    beta: f64,
    gamma: f64,
}

#[derive(Debug, Serialize, Deserialize)]
enum CjsonCoords {
    #[serde(rename = "3d")]
    Cart(Vec<f64>),
    #[serde(rename = "3d fractional")]
    Frac(Vec<f64>),
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CjsonAtoms {
    elements: CjsonElements,
    coords: Option<CjsonCoords>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CjsonElements {
    number: Vec<usize>,
}
// 635f7a3e ends here

// [[file:../../gchemol-readwrite.note::64eb0a93][64eb0a93]]
use vecfx::*;

pub(self) fn format_molecule(mol: &Molecule) -> Result<String> {
    let number: Vec<_> = mol.atoms().map(|(_, a)| a.number()).collect();
    let elements = CjsonElements { number };
    let coords: Vec<_> = mol.positions().collect();
    let coords = CjsonCoords::Cart(coords.as_flat().to_vec()).into();
    let m = CjsonMolecule {
        atoms: CjsonAtoms { elements, coords },
        ..Default::default()
    };
    let s = serde_json::to_string_pretty(&m)?;
    Ok(s)
}

pub(self) fn parse_molecule(s: &str) -> Result<Molecule> {
    let m: CjsonMolecule = serde_json::from_str(&s)?;
    let atom_numbers = m.atoms.elements.number;
    ensure!(m.atoms.coords.is_some(), "cjson has no coords");

    let mut pbc = false;
    let coords = match m.atoms.coords.unwrap() {
        CjsonCoords::Cart(coords) => coords.as_3d().to_vec(),
        CjsonCoords::Frac(coords) => {
            pbc = true;
            coords.as_3d().to_vec()
        }
    };

    // ensure!(coords.len(), atom_numbers.len(), "invalid cjson atoms");
    let mol = if pbc {
        ensure!(m.unit_cell.is_some(), "malformed cjson: found frac coords but no unit cell");
        let u = m.unit_cell.unwrap();
        let lattice = Lattice::from_params(u.a, u.b, u.c, u.alpha, u.beta, u.gamma);

        let atoms = atom_numbers.into_iter().zip(coords).map(|(i, frac)| Atom::new(i, lattice.to_cart(frac)));
        let mut mol = Molecule::from_atoms(atoms);
        mol.set_lattice(lattice);
        mol
    } else {
        let atoms = atom_numbers.into_iter().zip(coords).map(|(i, coord)| Atom::new(i, coord));
        Molecule::from_atoms(atoms)
    };

    Ok(mol)
}
// 64eb0a93 ends here

// [[file:../../gchemol-readwrite.note::9f750ad2][9f750ad2]]
use super::ChemicalFile;
use super::ParseMolecule;

#[derive(Clone, Copy, Debug)]
/// Chemical JSON
pub struct ChemicalJsonFile();

impl ChemicalFile for ChemicalJsonFile {
    fn ftype(&self) -> &str {
        "text/cjson"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".cjson"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        let s = format_molecule(mol)?;
        Ok(s)
    }
}

impl ParseMolecule for ChemicalJsonFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        let mol = parse_molecule(input)?;
        Ok(mol)
    }
}
// 9f750ad2 ends here

// [[file:../../gchemol-readwrite.note::268351d5][268351d5]]
use super::*;

// read all available stream at once
impl super::parser::ReadPart for ChemicalJsonFile {}

impl ChemicalJsonFile {
    pub fn partitions<R: BufRead + Seek>(&self, mut r: TextReader<R>) -> Result<impl Iterator<Item = String>> {
        Ok(r.partitions(*self))
    }
}
// 268351d5 ends here
