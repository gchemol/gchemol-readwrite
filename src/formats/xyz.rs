// [[file:../../gchemol-readwrite.note::*imports][imports:1]]
use super::parser::*;
use super::*;
// imports:1 ends here

// [[file:../../gchemol-readwrite.note::bdc38ed5][bdc38ed5]]
type Point3 = [f64; 3];

// C -10.0949 -0.5455  0.0000
fn read_atom_xyz(s: &str) -> IResult<&str, (&str, Point3, &str)> {
    let mut element = alt((digit1, alpha1));
    do_parse!(
        s,
        space0 >> s: element >> space1 >> p: xyz_array >>
        remained: read_line >>              // ignore the remaing
        ((s, p, remained))
    )
}

#[test]
fn test_xyz_read_atom() {
    let line = "C -11.4286 -1.3155  0.0000\n";
    let (_, (symbol, _, _)) = read_atom_xyz(line).expect("xyz atom");
    assert_eq!("C", symbol);

    let line = "6 -11.4286 -1.3155  0.0000 0.0 0.0 0.0\n";
    let (_, (symbol, position, _)) = read_atom_xyz(line).expect("xyz atom velocity");
    assert_eq!("6", symbol);
    assert_eq!(0.0, position[2]);
}

/// Create a list of atoms from many lines in xyz format
/// # Example
/// C -11.4286  1.7645  0.0000
/// C -10.0949  0.9945  0.0000
/// C -10.0949 -0.5455  0.0000
fn read_atoms_pxyz(s: &str) -> IResult<&str, Vec<(&str, Point3, &str)>> {
    many1(read_atom_xyz)(s)
}

#[test]
fn test_xyz_read_atoms() {
    let txt = "C -11.4286  1.7645  0.0000
C -10.0949  0.9945  0.0000
C -10.0949 -0.5455  0.0000
C -11.4286 -1.3155  0.0000
\n";
    let (_, atoms) = read_atoms_pxyz(txt).expect("read_atoms");
    assert_eq!(4, atoms.len());
}
// bdc38ed5 ends here

// [[file:../../gchemol-readwrite.note::f20b5155][f20b5155]]
// return molecule title and atoms
fn read_atoms_xyz(s: &str) -> IResult<&str, (&str, Vec<(&str, Point3, &str)>)> {
    do_parse!(
        s,
        n: read_usize >>          // the number of atoms
        title: read_line >>       // title line
        atoms: read_atoms_pxyz >> // symbols and positions
        ({
            if n != atoms.len() {
                warn!("Informal xyz format: expect {} atoms, but found {}", n, atoms.len());
                debug!("{:?}", s);
            }
            (title.trim(), atoms)
        })
    )
}

#[test]
fn test_xyz_read_molecule() {
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
// f20b5155 ends here

// [[file:../../gchemol-readwrite.note::ed71e42e][ed71e42e]]
fn parse_molecule(input: &str, plain: bool) -> Result<Molecule> {
    // plain xyz style with coordinates only?
    let mol = if plain {
        let (_, atoms) = read_atoms_pxyz(input)
            .map_err(|e| format_err!("Failed to parse atoms in plain xyz format: {}", e))?;

        build_mol(atoms)
    } else {
        let (_, (title, atoms)) = read_atoms_xyz(input)
            .map_err(|e| format_err!("Failed to parse atoms in xyz format: {}", e))?;

        let mut mol = build_mol(atoms);
        mol.set_title(&title);
        mol
    };

    Ok(mol)
}

fn parse_velocities(line: &str) -> Option<[f64; 3]> {
    use std::convert::TryInto;

    let vxyz: Option<Vec<f64>> = line.split_whitespace().map(|x| x.parse().ok()).collect();
    vxyz?.try_into().ok()
}

/// Handle dummy TV atoms (transitional vector, traditionally used in
/// Gaussian/MOPAC package for periodic system)
fn build_mol(atoms: Vec<(&str, [f64; 3], &str)>) -> Molecule {
    let mut lat_vectors = vec![];
    let atoms = atoms.into_iter().filter_map(|(sym, positions, other)| {
        let mut a: Atom = (sym, positions).into();
        // HACK: parse velocities
        if !other.trim().is_empty() {
            if let Some(xyz) = parse_velocities(other) {
                a.set_velocity(xyz);
            } else {
                debug!("ignored invalid fields: {other}");
            }
        }
        match a.kind() {
            AtomKind::Dummy(x) => {
                if x == "TV" {
                    trace!("found TV dummy atom.");
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
// ed71e42e ends here

// [[file:../../gchemol-readwrite.note::*xyz][xyz:1]]
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
        if mol.is_periodic() {
            writeln!(&mut lines, "{}", mol.natoms() + 3)?;
        } else {
            writeln!(&mut lines, "{}", mol.natoms())?;
        }
        writeln!(&mut lines, "{}", mol.title())?;

        // only write velocities when they are meaningful
        let write_velocity = !mol.atoms().all(|(_, a)| {
            let [vx, vy, vz] = a.velocity();
            vx == 0.0 && vy == 0.0 && vz == 0.0
        });
        for (_, a) in mol.atoms() {
            let p = a.position();
            let v = a.velocity();
            let sym = a.symbol();
            if write_velocity {
                writeln!(
                    &mut lines,
                    "{:6} {:-18.6}{:-18.6}{:-18.6}{:-18.6}{:-18.6}{:-18.6}",
                    sym, p[0], p[1], p[2], v[0], v[1], v[2]
                )?;
            } else {
                writeln!(&mut lines, "{:6} {:-18.6}{:-18.6}{:-18.6}", sym, p[0], p[1], p[2])?;
            }
        }

        // write lattice transition vectors using TV symbol.
        if let Some(lat) = &mol.lattice {
            for v in lat.vectors().iter() {
                writeln!(&mut lines, "TV {:-12.8} {:-12.8} {:-12.8}", v[0], v[1], v[2]);
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

// [[file:../../gchemol-readwrite.note::*plain xyz][plain xyz:1]]
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

// [[file:../../gchemol-readwrite.note::d27ea4ee][d27ea4ee]]
impl ReadPart for XyzFile {
    fn read_next(&self, context: ReadContext) -> ReadAction {
        let n = context.number_of_lines();
        // the first line contains the number of atoms in this part
        if let Ok(natoms) = context.line(1).trim().parse::<usize>() {
            if n >= natoms + 2 {
                ReadAction::Done(n)
            } else {
                ReadAction::Need(natoms + 2 - n)
            }
        } else {
            warn!("read_part context text: {:?}", context.text());
            warn!("read_part context line: {:?}", context.line(1));
            ReadAction::Error("invalid xyz title".into())
        }
    }
}

impl ReadPart for PlainXyzFile {
    fn read_next(&self, context: ReadContext) -> ReadAction {
        Terminated(|line: &str| line.trim().is_empty()).read_next(context)
    }
}

impl XyzFile {
    pub fn partitions<R: BufRead + Seek>(&self, mut r: TextReader<R>) -> impl Iterator<Item = String> {
        r.partitions(*self)
    }
}

impl PlainXyzFile {
    pub fn partitions<R: BufRead + Seek>(&self, mut r: TextReader<R>) -> impl Iterator<Item = String> {
        r.partitions(*self)
    }
}
// d27ea4ee ends here
