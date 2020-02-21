// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::{parser::*, *};
// imports:1 ends here

// atoms

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*atoms][atoms:1]]
type Point3 = [f64; 3];

// C -10.0949 -0.5455  0.0000
fn read_atom_xyz(s: &str) -> IResult<&str, (&str, Point3)> {
    let element = alt((digit1, alpha1));
    do_parse!(
        s,
        space0 >> s: element >> space1 >> p: xyz_array >>
        read_line >>              // ignore the remaing
        ((s, p))
    )
}

#[test]
fn test_read_atom() {
    let line = "C -11.4286 -1.3155  0.0000\n";
    let (_, (symbol, _)) = read_atom_xyz(line).expect("xyz atom");
    assert_eq!("C", symbol);

    let line = "6 -11.4286 -1.3155  0.0000 0.0 0.0 0.0\n";
    let (_, (symbol, position)) = read_atom_xyz(line).expect("xyz atom velocity");
    assert_eq!("6", symbol);
    assert_eq!(0.0, position[2]);
}

/// Create a list of atoms from many lines in xyz format
/// # Example
/// C -11.4286  1.7645  0.0000
/// C -10.0949  0.9945  0.0000
/// C -10.0949 -0.5455  0.0000
fn read_atoms_pxyz(s: &str) -> IResult<&str, Vec<(&str, Point3)>> {
    many1(read_atom_xyz)(s)
}

#[test]
fn test_read_atoms() {
    let txt = "C -11.4286  1.7645  0.0000
C -10.0949  0.9945  0.0000
C -10.0949 -0.5455  0.0000
C -11.4286 -1.3155  0.0000
\n";
    let (_, atoms) = read_atoms_pxyz(txt).expect("read_atoms");
    assert_eq!(4, atoms.len());
}
// atoms:1 ends here

// xyz/pxyz

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*xyz/pxyz][xyz/pxyz:1]]
// return molecule title and atoms
fn read_atoms_xyz(s: &str) -> IResult<&str, (&str, Vec<(&str, Point3)>)> {
    do_parse!(
        s,
        n: read_usize >>          // the number of atoms
        title: read_line >>       // title line
        atoms: read_atoms_pxyz >> // symbols and positions
        ({
            if n != atoms.len() {
                warn!("Informal xyz format: expect {} atoms, but found {}", n, atoms.len());
            }
            (title.trim(), atoms)
        })
    )
}

#[test]
fn test_read_molecule_xyz() {
    let txt = "12

C -11.4286  1.7645  0.0000
C -10.0949  0.9945  0.0000
C -10.0949 -0.5455  0.0000
C -11.4286 -1.3155  0.0000
C -12.7623 -0.5455  0.0000
C -12.7623  0.9945  0.0000
H -11.4286  2.8545  0.0000
H -9.15090  1.5395  0.0000
H -9.15090 -1.0905  0.0000
H -11.4286 -2.4055  0.0000
H -13.7062 -1.0905  0.0000
H -13.7062  1.5395  0.0000";

    let (_, (_, atoms)) = read_atoms_xyz(txt).expect("xyz atoms");
    assert_eq!(12, atoms.len());
}
// xyz/pxyz:1 ends here

// parse

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*parse][parse:1]]
fn parse_molecule(input: &str, plain: bool) -> Result<Molecule> {
    // plain xyz style with coordinates only?
    let mol = if plain {
        let (_, atoms) =
            read_atoms_pxyz(input).map_err(|e| format_err!("Failed to parse atoms in plain xyz format: {}", e))?;

        build_mol(atoms)
    } else {
        let (_, (title, atoms)) =
            read_atoms_xyz(input).map_err(|e| format_err!("Failed to parse atoms in xyz format: {}", e))?;

        let mut mol = build_mol(atoms);
        mol.set_title(&title);
        mol
    };

    Ok(mol)
}

/// Handle dummy TV atoms (transitional vector, traditionally used in
/// Gaussian/MOPAC package for periodic system)
fn build_mol(atoms: Vec<(&str, [f64; 3])>) -> Molecule {
    let mut lat_vectors = vec![];
    let atoms = atoms.into_iter().filter_map(|x| {
        let a: Atom = x.into();
        match a.kind() {
            AtomKind::Dummy(x) => {
                if x == "TV" {
                    debug!("found TV dummy atom.");
                    lat_vectors.push(a.position());
                }
                None
            }
            AtomKind::Element(x) => Some(a),
        }
    });
    let mut mol = Molecule::from_atoms(atoms);

    // construct lattice parsed from three "TV" dummy atoms.
    if lat_vectors.len() == 3 {
        let lat = Lattice::new([lat_vectors[0], lat_vectors[1], lat_vectors[2]]);
        mol.set_lattice(lat);
    } else if !lat_vectors.is_empty() {
        error!("Expect 3, but found {} TV atoms.", lat_vectors.len());
    }
    mol
}
// parse:1 ends here

// format

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*format][format:1]]
fn format_molecule() {
    todo!()
}
// format:1 ends here

// xyz

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*xyz][xyz:1]]
/// Classical XYZ format
#[derive(Copy, Clone, Debug)]
pub(super) struct XyzFile();

impl ChemicalFile for XyzFile {
    fn ftype(&self) -> &str {
        "text/xyz"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".xyz"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        // meta information
        let mut lines = String::new();
        lines.push_str(&format!("{}\n", mol.natoms()));
        lines.push_str(&format!("{}\n", mol.title()));

        // coordinates
        for (_, a) in mol.atoms() {
            let p = a.position();
            let v = a.momentum();
            let sym = a.symbol();
            let s = format!(
                "{:6} {:-18.6}{:-18.6}{:-18.6}{:-18.6}{:-18.6}{:-18.6}\n",
                sym, p[0], p[1], p[2], v[0], v[1], v[2]
            );
            lines.push_str(&s);
        }

        // write lattice transition vectors using TV symbol.
        if let Some(lat) = &mol.lattice {
            for v in lat.vectors().iter() {
                let line = format!("TV {:-12.8} {:-12.8} {:-12.8}\n", v[0], v[1], v[2]);
                lines.push_str(&line);
            }
        }

        Ok(lines)
    }
}

impl ParseMolecule for XyzFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        parse_molecule(input, false)
    }
}
// xyz:1 ends here

// plain xyz

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*plain xyz][plain xyz:1]]
/// Plain xyz coordinates with atom symbols (no atom count line and title line)
#[derive(Debug, Clone, Copy)]
pub(super) struct PlainXyzFile();

impl ParseMolecule for PlainXyzFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        // remove starting empty line
        parse_molecule(input.trim_start(), true)
    }
}

impl ChemicalFile for PlainXyzFile {
    /// Possible file extensions
    fn possible_extensions(&self) -> Vec<&str> {
        [".coord", ".pxyz", ".coords"].to_vec()
    }

    fn ftype(&self) -> &str {
        "text/pxyz"
    }

    /// Return a string representation of molecule
    /// Multiple molecules will be separated by a blank line
    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        let mut lines = String::new();

        for (_, a) in mol.atoms() {
            lines.push_str(format!("{}\n", a.to_string()).as_ref());
        }

        // write lattice transition vectors using TV symbol.
        if let Some(lat) = &mol.lattice {
            for v in lat.vectors().iter() {
                let line = format!("TV {:-12.8} {:-12.8} {:-12.8}\n", v[0], v[1], v[2]);
                lines.push_str(&line);
            }
        }

        // append a blank line as a separator between multiple molecules
        lines.push_str("\n");

        Ok(lines)
    }
}
// plain xyz:1 ends here

// impl partition

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*impl partition][impl partition:1]]
impl Partition for XyzFile {
    fn read_next(&self, context: ReadContext) -> bool {
        !context.next_line().trim().parse::<usize>().is_ok()
    }
}

impl Partition for PlainXyzFile {
    fn read_next(&self, context: ReadContext) -> bool {
        !context.this_line().trim().is_empty()
    }
}
// impl partition:1 ends here
