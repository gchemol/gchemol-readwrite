// [[file:../gchemol-readwrite.note::43d6e881][43d6e881]]
//! Template rendering for molecule

use indexmap::{indexmap, IndexMap};
use serde_json::json;

use gchemol_core::{Atom, Molecule};
use gut::prelude::*;
// 43d6e881 ends here

// [[file:../gchemol-readwrite.note::925f7269][925f7269]]
mod hbs;
mod jinja;
mod tera;
// 925f7269 ends here

// [[file:../gchemol-readwrite.note::*core][core:1]]
#[derive(Debug, Serialize)]
struct AtomData {
    index: usize,
    element_index: usize,
    symbol: String,
    number: usize,
    freezing: [bool; 3],
    x: f64,
    y: f64,
    z: f64,
    fx: f64,
    fy: f64,
    fz: f64,
    vx: f64,
    vy: f64,
    vz: f64,
}

impl Default for AtomData {
    fn default() -> Self {
        AtomData {
            index: 0,
            element_index: 0,
            symbol: "C".into(),
            number: 6,
            freezing: [false; 3],
            x: 0.0,
            y: 0.0,
            z: 0.0,
            fx: 0.0,
            fy: 0.0,
            fz: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
        }
    }
}

#[derive(Debug, Serialize)]
struct BondData {
    i: usize,
    j: usize,
    order: f64,
}

#[derive(Debug, Serialize)]
struct UnitCell {
    a: f64,
    b: f64,
    c: f64,
    alpha: f64,
    beta: f64,
    gamma: f64,
    va: [f64; 3],
    vb: [f64; 3],
    vc: [f64; 3],
}

#[derive(Debug, Serialize)]
struct SpeciesData {
    index: usize,
    element_symbol: String,
    element_number: usize,
    number_of_atoms: usize,
}

#[derive(Debug, Serialize)]
struct MoleculeData {
    title: String,
    unit_cell: Option<UnitCell>,
    number_of_atoms: usize,
    number_of_bonds: usize,
    number_of_species: usize,
    atoms: Vec<AtomData>,
    bonds: Vec<BondData>,

    // mapping element type and numbers as in VASP POSCAR
    // O C H  # element symbol
    // 1 2 3  # element count
    element_types: Vec<(String, usize)>,
    species: Vec<SpeciesData>,
}

/// construct a shallow representation of molecule for templating
pub(self) fn renderable(mol: &Molecule) -> serde_json::Value {
    // unit cell data
    let unit_cell = if let Some(lat) = mol.lattice {
        let [va, vb, vc] = lat.vectors();
        let [a, b, c] = lat.lengths();
        let [alpha, beta, gamma] = lat.angles();

        let cell = UnitCell {
            a,
            b,
            c,
            alpha,
            beta,
            gamma,
            va: va.into(),
            vb: vb.into(),
            vc: vc.into(),
        };

        Some(cell)
    } else {
        None
    };

    let mut element_types: IndexMap<String, usize> = indexmap! {};
    for (_, a) in mol.atoms() {
        let k = a.symbol().into();
        let c = element_types.entry(k).or_insert(0);
        *c += 1;
    }

    // atoms data
    let mut atoms = vec![];
    for (i, a) in mol.atoms() {
        let [x, y, z] = a.position();
        let index = i;
        let number = a.number();
        let symbol = a.symbol().to_string();
        let [fx, fy, fz] = mol.lattice.map(|lat| lat.to_frac([x, y, z]).into()).unwrap_or([0.0; 3]);

        let element_index = {
            let (x, _, _) = element_types.get_full(a.symbol()).expect("element type index");
            x + 1
        };

        let [vx, vy, vz] = a.velocity();
        let freezing = a.freezing();
        atoms.push(AtomData {
            index,
            element_index,
            symbol,
            number,
            freezing,
            x,
            y,
            z,
            fx,
            fy,
            fz,
            vx,
            vy,
            vz,
        })
    }

    // convert indexmap to plain list
    let element_types: Vec<(_, _)> = element_types.into_iter().collect();

    let n = element_types.len();
    let species: Vec<_> = element_types
        .iter()
        .enumerate()
        .map(|(i, (s, n))| SpeciesData {
            index: i + 1,
            element_symbol: s.clone(),
            // FIXME: dirty
            element_number: Atom::new(s.as_str(), [0.0; 3]).number(),
            number_of_atoms: *n,
        })
        .collect();

    let bonds = vec![];
    let md = MoleculeData {
        title: mol.title(),
        number_of_atoms: mol.natoms(),
        number_of_bonds: mol.nbonds(),
        number_of_species: n,
        unit_cell,
        atoms,
        bonds,
        element_types,
        species,
    };

    json!({
        "molecule": md,
    })
}
// core:1 ends here

// [[file:../gchemol-readwrite.note::3582ce8f][3582ce8f]]
/// Render molecule in user defined format
pub trait TemplateRendering {
    /// Render with input template file.
    fn render_with(&self, f: &std::path::Path) -> Result<String>;
}

impl TemplateRendering for Molecule {
    fn render_with(&self, path: &std::path::Path) -> Result<String> {
        let template = gut::fs::read_file(path)?;

        // possible extension in lowercase only
        match path.extension().and_then(|x| x.to_str()) {
            Some("hbs") => self::hbs::render_molecule_with(&self, &template),
            Some("tera") => self::tera::render_molecule_with(&self, &template),
            _ => self::jinja::render_molecule_with(&self, &template),
        }
    }
}
// 3582ce8f ends here

// [[file:../gchemol-readwrite.note::e22b38a2][e22b38a2]]
/// Convert molecule to json string ready for template rendering
pub fn to_json(mol: &Molecule) -> Result<String> {
    let data = renderable(mol);
    let serialized = serde_json::to_string_pretty(&data)?;
    Ok(serialized)
}

/// Convert molecule to json Value data structure for template rendering
pub fn to_json_value(mol: &Molecule) -> serde_json::Value {
    renderable(mol)
}
// e22b38a2 ends here

// [[file:../gchemol-readwrite.note::0f1f521b][0f1f521b]]
pub use self::jinja::Template;
// 0f1f521b ends here
