// [[file:../../gchemol-readwrite.note::574001a8][574001a8]]
use super::*;
// 574001a8 ends here

// [[file:../../gchemol-readwrite.note::7636c8a8][7636c8a8]]
/// The extended XYZ format
#[derive(Copy, Clone, Debug)]
pub struct ExtxyzFile();
// 7636c8a8 ends here

// [[file:../../gchemol-readwrite.note::b83394d9][b83394d9]]
fn get_lattice(flat_vector: Vec<f64>) -> Option<Lattice> {
    if flat_vector.len() >= 9 {
        let va: [f64; 3] = flat_vector[..3].try_into().ok()?;
        let vb: [f64; 3] = flat_vector[3..6].try_into().ok()?;
        let vc: [f64; 3] = flat_vector[6..9].try_into().ok()?;
        let lat = Lattice::new([va, vb, vc]);
        Some(lat)
    } else {
        None
    }
}
// b83394d9 ends here

// [[file:../../gchemol-readwrite.note::9078bfde][9078bfde]]
impl ExtxyzFile {
    /// Return Lattice object by reading data from `line` in ase [extxyz](https://wiki.fysik.dtu.dk/ase/ase/io/formatoptions.html#xyz) format.
    ///
    /// Lattice="13.5142 0.0 0.0 0.0 14.9833 0.0 0.0 0.0 20.0" Properties=species:S:1:pos:R:3 pbc="T T T"
    pub fn read_lattice(line: &str) -> Option<Lattice> {
        if line.starts_with("Lattice=") {
            if let Some(pos) = &line[9..].find("\"") {
                let lattice_str = &line[9..pos + 9];
                let lattice_numbers: Vec<f64> = lattice_str.split_ascii_whitespace().filter_map(|value| value.parse().ok()).collect();
                return get_lattice(lattice_numbers);
            }
        }
        None
    }
}

#[test]
fn test_lattice_extxyz() {
    let line = "Lattice=\"13.5142 0.0 0.0 0.0 14.9833 0.0 0.0 0.0 20.0\" Properties=species:S:1:pos:R:3 pbc=\"T T T\"";
    let lat = ExtxyzFile::read_lattice(line);
    assert!(lat.is_some());
    assert_eq!(lat.unwrap().lengths(), [13.5142, 14.9833, 20.0]);
}
// 9078bfde ends here

// [[file:../../gchemol-readwrite.note::8ac5d7e7][8ac5d7e7]]
use ::extxyz::{read_xyz_frames_direct, Info, RawAtoms};

fn get_array(value: serde_json::Value) -> Option<Vec<f64>> {
    let array = value.as_array()?;
    array.iter().map(|x| x.as_f64()).collect()
}

fn get_molecule_from_extxyz_atoms(raw_atoms: RawAtoms) -> Result<Molecule> {
    ensure!(raw_atoms.natoms == raw_atoms.atoms.len());

    let mut mol = Molecule::default();
    if let Ok(mut info) = raw_atoms.comment.parse::<Info>() {
        // get atom's properties
        for (i, a) in raw_atoms.atoms.into_iter().enumerate() {
            let mut atom = Atom::new(a.element, a.positions);
            // parse extra data for each atom
            let mut atom_properties = info.parse_extra_columns(&a.extra)?;
            atom.properties.raw_map_mut().append(&mut atom_properties);
            mol.add_atom(i + 1, atom);
        }
        // get molecule's properties
        if let Some(lat) = info.pop("Lattice").and_then(get_array).and_then(get_lattice) {
            mol.set_lattice(lat);
        }
        mol.properties.raw_map_mut().append(info.raw_map_mut());
    } else {
        mol.set_title(raw_atoms.comment);
    }

    Ok(mol)
}

impl ExtxyzFile {
    /// Read `Molecule` from file in `path` in extxyz format.
    pub fn read_molecules_from(path: impl AsRef<Path>) -> Result<impl Iterator<Item = Molecule>> {
        let frames = read_xyz_frames_direct(path)?;
        let mols = frames.filter_map(|frame| {
            let atoms = RawAtoms::parse_from(&frame).unwrap();
            let mol = get_molecule_from_extxyz_atoms(dbg!(atoms)).unwrap();
            Some(mol)
        });
        Ok(mols)
    }
}
// 8ac5d7e7 ends here

// [[file:../../gchemol-readwrite.note::ec30581c][ec30581c]]

// ec30581c ends here

// [[file:../../gchemol-readwrite.note::55a4e567][55a4e567]]
#[test]
fn test_read_extxyz() -> Result<()> {
    let f = "tests/files/extxyz/cu.xyz";
    let mols = ExtxyzFile::read_molecules_from(f)?;
    let mol = mols.last().unwrap();
    assert_eq!(mol.natoms(), 107);
    assert!(mol.get_lattice().is_some());

    // test molecule's properties
    let energy: f64 = mol.properties.load("energy")?;
    assert_eq!(energy, 0.63);
    let user_data: Vec<usize> = mol.properties.load("user-data")?;
    assert_eq!(user_data.len(), 3);

    // test atom's properties
    let atom = mol.get_atom(1).unwrap();
    let energy: f64 = atom.properties.load("energy")?;
    assert_eq!(energy, 0.08400641);

    let force: [f64; 3] = atom.properties.load("forces")?;
    assert_eq!(force.len(), 3);

    Ok(())
}
// 55a4e567 ends here
