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
    let lines: Vec<_> = input.trim().lines().collect();
    ensure!(lines.len() > 2, "invalid xyz part: {lines:?}");

    let mol = if plain {
        build_mol_xyz(&lines[..])?
    } else {
        let natoms: usize = lines[0].trim().parse()?;
        let title = lines[1].trim();
        let mut mol = build_mol_xyz(&lines[2..])?;
        mol.set_title(title.to_owned());
        let natoms_ = mol.natoms();
        if natoms_ != natoms {
            warn!("found xyz format error: expand {natoms}, but found {natoms_}");
        }
        mol
    };

    Ok(mol)
}

/// Handle dummy TV atoms (transitional vector, traditionally used in
/// Gaussian/MOPAC package for periodic system)
fn build_mol_xyz(lines: &[&str]) -> Result<Molecule> {
    let mut atoms = vec![];
    for line in lines.iter() {
        let a: Atom = line.parse()?;
        atoms.push(a);
    }

    // HACK: parse TV/VEC for lattice vectors
    let mut lat_vectors = vec![];
    let atoms = atoms.into_iter().filter_map(|a| match a.kind() {
        AtomKind::Dummy(x) => {
            // ASE/ADF writes cell vectors using VEC line
            if x == "TV" || x == "VEC1" || x == "VEC2" || x == "VEC3" {
                trace!("found TV dummy atom.");
                lat_vectors.push(a.position());
            }
            None
        }
        AtomKind::Element(x) => Some(a),
    });

    let mut mol = Molecule::from_atoms(atoms);

    // construct lattice parsed from three "TV" dummy atoms.
    if lat_vectors.len() == 3 {
        let lat = Lattice::new([lat_vectors[0], lat_vectors[1], lat_vectors[2]]);
        mol.set_lattice(lat);
    } else if !lat_vectors.is_empty() {
        error!("Expect 3, but found {} TV atoms.", lat_vectors.len());
    }

    Ok(mol)
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

// [[file:../../gchemol-readwrite.note::c62a0af4][c62a0af4]]
impl XyzFile {
    pub fn partitions<R: BufRead + Seek>(&self, mut reader: TextReader<R>) -> Result<impl Iterator<Item = String>> {
        let get_natoms = |line: &str| line.trim().parse::<usize>().ok();

        let iter = std::iter::from_fn(move || {
            let mut buf = String::new();
            let _ = reader.read_line(&mut buf)?;
            let natoms: usize = get_natoms(&buf)?;
            // skip comment line
            let _ = reader.read_line(&mut buf)?;
            // skip lines for n atoms
            for _ in 0..natoms {
                let n = reader.read_line(&mut buf)?;
            }
            // read extra lines which may exists as TV or VEC for cell vectors
            if let Some(line) = reader.peek_line() {
                if get_natoms(&line).is_some() {
                    return Some(buf);
                } else {
                    for _ in 0..3 {
                        reader.read_line(&mut buf);
                    }
                }
            }
            return Some(buf);
        });
        Ok(iter)
    }
}
// c62a0af4 ends here

// [[file:../../gchemol-readwrite.note::d27ea4ee][d27ea4ee]]
impl PlainXyzFile {
    pub fn partitions<R: BufRead + Seek>(&self, mut reader: TextReader<R>) -> Result<impl Iterator<Item = String>> {
        let iter = std::iter::from_fn(move || {
            let mut buf = String::new();
            let mut eof = false;
            // stop when found an empty line or reach EOF
            loop {
                if let Some(n) = reader.read_line(&mut buf) {
                    let m = buf.len();
                    if buf[m - n..].trim().is_empty() {
                        break Some(buf);
                    }
                } else {
                    // we should not miss the last part when reach EOF
                    if buf.is_empty() {
                        break None;
                    } else {
                        break Some(buf);
                    }
                }
            }
        });
        Ok(iter)
    }
}
// d27ea4ee ends here

// [[file:../../gchemol-readwrite.note::9078bfde][9078bfde]]
/// Return Lattice object by reading data from `line` in ase [extxyz](https://wiki.fysik.dtu.dk/ase/ase/io/formatoptions.html#xyz) format.
///
/// Lattice="13.5142 0.0 0.0 0.0 14.9833 0.0 0.0 0.0 20.0" Properties=species:S:1:pos:R:3 pbc="T T T"
pub fn read_lattice_extxyz(line: &str) -> Option<Lattice> {
    if line.starts_with("Lattice=") {
        if let Some(pos) = &line[9..].find("\"") {
            let lattice_str = &line[9..pos + 9];
            let lattice_numbers: Vec<f64> = lattice_str.split_ascii_whitespace().filter_map(|value| value.parse().ok()).collect();
            if lattice_numbers.len() != 9 {
                return None;
            }
            let va: [f64; 3] = lattice_numbers[..3].try_into().ok()?;
            let vb: [f64; 3] = lattice_numbers[3..6].try_into().ok()?;
            let vc: [f64; 3] = lattice_numbers[6..9].try_into().ok()?;
            let lat = Lattice::new([va, vb, vc]);
            return Some(lat);
        }
    }
    None
}

#[test]
fn test_lattice_extxyz() {
    let line = "Lattice=\"13.5142 0.0 0.0 0.0 14.9833 0.0 0.0 0.0 20.0\" Properties=species:S:1:pos:R:3 pbc=\"T T T\"";
    let lat = read_lattice_extxyz(line);
    assert!(lat.is_some());
    assert_eq!(lat.unwrap().lengths(), [13.5142, 14.9833, 20.0]);
}
// 9078bfde ends here
