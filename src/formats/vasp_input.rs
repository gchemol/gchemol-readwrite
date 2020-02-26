// header
// VASP POSCAR file format:
// - http://cms.mpi.univie.ac.at/vasp/guide/node59.html


// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*header][header:1]]

// header:1 ends here

// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::parser::*;
use super::*;
// imports:1 ends here

// cell

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*cell][cell:1]]
fn poscar_cell_vectors(s: &str) -> IResult<&str, [[f64; 3]; 3]> {
    do_parse!(
        s,
        space0 >> va: xyz_array >> eol >> // vector a
        space0 >> vb: xyz_array >> eol >> // vector b
        space0 >> vc: xyz_array >> eol >> // vector c
        ([va, vb, vc])
    )
}

#[test]
fn test_poscar_cell_vectors() {
    let lines = " 21.23300000  0.00000000  0.00000000
  0.00000000 26.60400000  0.00000000
  0.00000000  0.00000000 12.67600000
";

    let (_, x) = poscar_cell_vectors(lines).expect("POSCAR cell vectors");
    assert_eq!(21.233, x[0][0]);
}
// cell:1 ends here

// elements

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*elements][elements:1]]
fn poscar_ion_types(s: &str) -> IResult<&str, (Vec<&str>, Vec<usize>)> {
    let elements = separated_list(space1, alpha1);
    let natoms = separated_list(space1, unsigned_digit);
    do_parse!(
        s,
        space0 >> e: elements       >> eol >> // element list
        space0 >> n: natoms         >> eol >> // natoms list
        ((e, n))
    )
}

#[test]
fn test_formats_vasp_poscar_ion_types() {
    let lines = " O    Si   C    N    H
 225  112   8    1    19 \n";
    let (_, v) = poscar_ion_types(lines).expect("POSCAR ion types");
    assert_eq!(5, v.0.len());
}
// elements:1 ends here

// coordinates

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*coordinates][coordinates:1]]
// Selective dynamics -- optional, can be omitted
// only the first character is relevant
fn selective_dynamics(s: &str) -> IResult<&str, &str> {
    let selective = tag_no_case("S");
    do_parse!(s, x: selective >> read_line >> (x))
}

/// Consume three chars in selective dynamics flag (T/F) separated by one or
/// more spaces Return the frozen flag array
fn selective_dynamics_flags(s: &str) -> IResult<&str, [bool; 3]> {
    let tf_flag = one_of("TF");
    do_parse!(
        s,
        x: tf_flag >> space1 >> y: tf_flag >> space1 >> z: tf_flag >> // T T F
        ([x == 'T', y == 'T', z == 'T'])
    )
}

#[test]
fn test_poscar_select_dynamics() {
    let (_, x) = selective_dynamics_flags("T T F").unwrap();
    assert_eq!(x, [true, true, false]);
}

// Direct/Cartesian -- lattice coordinates type
// only the first character is relevant
fn direct_or_catersian(s: &str) -> IResult<&str, &str> {
    let coords_type = alt((tag_no_case("D"), tag_no_case("C")));
    do_parse!(s, d: coords_type >> read_line >> (d))
}

// combine two parsers
fn poscar_select_direct(s: &str) -> IResult<&str, (bool, bool)> {
    let selective_line = opt(selective_dynamics);
    let direct_line = direct_or_catersian;
    do_parse!(
        s,
        s: selective_line >>    // Selective dynamics
        d: direct_line    >>    // Direct
        ({
            let d = d.to_lowercase();
            (s.is_some(), d == "d")
        })
    )
}

#[test]
fn test_poscar_select_direct() {
    let lines = "Selective dynamics
Direct\n";

    let (_, (s, d)) = poscar_select_direct(lines).expect("poscar selective/direct");
    assert_eq!(true, s);
    assert_eq!(true, d);

    let (_, (s, d)) = poscar_select_direct("Direct\n").expect("poscar direct");
    assert_eq!(false, s);
    assert_eq!(true, d);
    let (_, (s, d)) = poscar_select_direct("Cartesian\n").expect("poscar catersian");
    assert_eq!(false, s);
    assert_eq!(false, d);
}

// 0.05185     0.39121     0.29921  T T T # O
// 0.81339     0.57337     0.68777  T T T # O
fn poscar_position(s: &str) -> IResult<&str, ([f64; 3], Option<[bool; 3]>)> {
    let frozen_flags = opt(selective_dynamics_flags);
    do_parse!(
        s,
        space0 >> p: xyz_array >> space0 >> // Coordinates
        f: frozen_flags >> read_line     >> // T T T
        ((p, f))
    )
}

#[test]
fn test_poscar_position() {
    let line = "     0.05185     0.39121     0.29921  T T T # O \n";
    let (_, (position, sflags)) = poscar_position(line).expect("POSCAR position style 1");
    assert_eq!(0.05185, position[0]);
    assert_eq!(0.39121, position[1]);
    assert_eq!(0.29921, position[2]);
    assert_eq!(Some([true, true, true]), sflags);

    let line = "     0.05185     0.39121     0.29921\n";
    let (_, (position, sflags)) = poscar_position(line).expect("POSCAR position style 1");
    assert_eq!(None, sflags);
}
// coordinates:1 ends here

// parse molecule

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*parse molecule][parse molecule:1]]
/// Read Molecule from stream in VASP/POSCAR format
pub(crate) fn parse_poscar_molecule(s: &str) -> IResult<&str, Molecule> {
    let read_ion_positions = many1(poscar_position);
    do_parse!(
        s,
        title            : read_until_eol        >> // system title
        lattice_constant : double >> eol         >> // lattice constant
        cell_vectors     : poscar_cell_vectors   >> // lattice vectors
        ion_types        : poscar_ion_types      >> // ion types
        select_direct    : poscar_select_direct  >> // selective line and direct line
        ion_positions    : read_ion_positions    >> // ion positions
    (
        {
            let selective_dynamics = select_direct.0;
            let direct_coordinates = select_direct.1;

            let mut mol = Molecule::new(title.trim());
            let mut lat = Lattice::new(cell_vectors);
            let x = lat.to_cart([0.0; 3]);
            // lat.scale_by(lattice_constant);

            let mut symbols = vec![];
            let (syms, nums) = ion_types;

            for (&sym, &num) in syms.iter().zip(nums.iter()) {
                for _ in 0..num {
                    symbols.push(sym);
                }
            }

            if symbols.len() != ion_positions.len() {
                eprintln!("WARNING: some ions data not correctly parsed!");
            }

            for (i, (&sym, (pos, sflags))) in symbols.iter().zip(ion_positions).enumerate() {
                let pos: Vector3f = if direct_coordinates {
                    lat.to_cart(pos)
                } else {
                    pos.into()
                };
                let mut a = Atom::new(sym, pos);
                // FIXME: adhoc hacking
                if sflags.is_some() {
                    a.properties.store(POSCAR_SFLAGS_KEY, sflags.unwrap());
                }
                mol.add_atom(i+1, a);
            }
            mol.set_lattice(lat);
            mol
        }
    )
    )
}

#[test]
fn test_poscar_molecule() {
    let lines = "title
1.0
 21.23300000  0.00000000  0.00000000
  0.00000000 26.60400000  0.00000000
  0.00000000  0.00000000 12.67600000
 O    Si   C    N    H
225  112   8    1    19
Selective dynamics
Direct
     0.05185     0.39121     0.29921  T T T # O
     0.81339     0.57337     0.68777  T T T # O
     0.73422     0.23229     0.85313  T T T # O
     0.02246     0.05156     0.49349  T T T # O
     0.64451     0.66726     0.17130  T T T # O
     0.05185     0.07337     0.29921  T T T # O
     0.60095     0.57471     0.17096  T T T # O
     0.64451     0.66726     0.81569  T T T # O
     0.33416     0.64745     0.88951  T T T # O
     0.33416     0.31713     0.09747  T T T # O
     0.93262     0.92263     0.99349  T T T # O
     0.43262     0.79195     0.99349  T T T # O
     0.73422     0.73229     0.13386  T T T # O
     0.22073     0.66726     0.81569  T T T # O
\n";

    let (_, mol) = parse_poscar_molecule(lines).expect("poscar molecule");
    assert_eq!(14, mol.natoms());
    assert!(mol.lattice.is_some());
}
// parse molecule:1 ends here

// format molecule

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*format molecule][format molecule:1]]
const POSCAR_SFLAGS_KEY: &str = "vasp/poscar/sflags";

fn format_molecule(mol: &Molecule) -> String {
    let mut lines = String::new();
    let title = mol.title();

    lines.push_str(&format!("{}\n", title));
    lines.push_str("1.0\n");
    let mut lattice = mol.lattice.expect("poscar lattice");
    let va = lattice.vector_a();
    let vb = lattice.vector_b();
    let vc = lattice.vector_c();

    for v in [va, vb, vc].iter() {
        let line = format!("{:12.8}{:12.8}{:12.8}\n", v[0], v[1], v[2]);
        lines.push_str(&line);
    }

    // atom symbols and counts
    let mut line1 = String::new();
    let mut line2 = String::new();
    for (s, n) in count_symbols(mol.symbols().collect()) {
        line1.push_str(&format!(" {:^4}", s));
        line2.push_str(&format!(" {:^4}", n));
    }
    lines.push_str(&format!("{}\n", line1));
    lines.push_str(&format!("{}\n", line2));

    // write fractional coordinates for improving accuracy
    lines.push_str("Selective dynamics\nDirect\n");
    for (_, a) in mol.atoms() {
        let p = lattice.to_frac(a.position());
        // FIXME: just a temporary workaround
        let line = if a.properties.contains_key(POSCAR_SFLAGS_KEY) {
            let sflags: [bool; 3] = a
                .properties
                .load(POSCAR_SFLAGS_KEY)
                .expect("vasp selective_dynamics flags");
            format!(
                "{x:18.12} {y:18.12} {z:18.12} {fx} {fy} {fz}\n",
                x = p.x,
                y = p.y,
                z = p.z,
                fx = if sflags[0] { "T" } else { "F" },
                fy = if sflags[1] { "T" } else { "F" },
                fz = if sflags[2] { "T" } else { "F" },
            )
        } else {
            format!("{x:18.12} {y:18.12} {z:18.12} T T T\n", x = p.x, y = p.y, z = p.z)
        };
        let line = format!("{x:18.12} {y:18.12} {z:18.12} T T T\n", x = p.x, y = p.y, z = p.z);
        lines.push_str(&line);
    }

    // final blank line
    lines.push_str("\n");
    // TODO: write velocities

    lines
}

// Panic if symbols is empty
fn count_symbols(symbols: Vec<&str>) -> Vec<(&str, usize)> {
    let mut lines = String::new();

    let mut syms1 = symbols.iter();
    let mut syms2 = symbols.iter().skip(1);
    let mut counts = vec![];

    let mut c = 1;
    let mut s = symbols[0];
    for (&sym1, &sym2) in syms1.zip(syms2) {
        if sym2 == sym1 {
            c += 1;
        } else {
            counts.push((sym1, c));
            c = 1;
        }
        s = sym2;
    }
    // append the last piece
    counts.push((s, c));

    counts
}

#[test]
fn test_poscar_symbols_counts() {
    let symbols = ["C", "C", "C", "H", "O", "O", "C"];
    let x = count_symbols(symbols.to_vec());
    assert_eq!([("C", 3), ("H", 1), ("O", 2), ("C", 1)].to_vec(), x);

    let symbols = ["C", "C"];
    let x = count_symbols(symbols.to_vec());
    assert_eq!([("C", 2)].to_vec(), x);

    let symbols = ["C", "H"];
    let x = count_symbols(symbols.to_vec());
    assert_eq!([("C", 1), ("H", 1)].to_vec(), x);

    let symbols = ["C"];
    let x = count_symbols(symbols.to_vec());
    assert_eq!([("C", 1)].to_vec(), x);
}
// format molecule:1 ends here

// chemfile

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*chemfile][chemfile:1]]
#[derive(Clone, Copy, Debug)]
pub struct PoscarFile();

impl ChemicalFile for PoscarFile {
    fn ftype(&self) -> &str {
        "vasp/input"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec!["poscar", "vasp"]
    }

    /// Determine if file `filename` is parable according to its supported file
    /// extensions
    fn parsable(&self, path: &Path) -> bool {
        let possible_filenames = vec!["CONTCAR", "POSCAR"];
        if let Some(e) = path.extension() {
            let e = e.to_string_lossy().to_lowercase();
            self.possible_extensions().contains(&e.as_str())
        } else
        // no extension: check file name
        {
            if let Some(filename) = path.file_name() {
                let f = filename.to_string_lossy().to_uppercase();
                for x in possible_filenames {
                    if f.starts_with(x) {
                        return true;
                    }
                }
            }
            false
        }
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        Ok(format_molecule(mol))
    }
}

impl ParseMolecule for PoscarFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        let (_, mol) = parse_poscar_molecule(input).map_err(|e| format_err!("parse POSCAR format failure: {:?}", e))?;
        Ok(mol)
    }
}

#[test]
fn test_vasp_input_parsable() {
    let cf = PoscarFile();
    let parsable = |x: &str| cf.parsable(x.as_ref());

    assert!(parsable("POSCAR"));
    assert!(parsable("POSCAR1"));
    assert!(parsable("POSCAR-1"));
    assert!(!parsable("POSCAR.1"));
    assert!(parsable("CONTCAR"));
    assert!(parsable("CONTCAR1"));
    assert!(parsable("POSCAR2"));
    assert!(parsable("poscar2"));
    assert!(parsable("x.poscar"));
    assert!(parsable("x.vasp"));
}
// chemfile:1 ends here

// impl partition

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*impl partition][impl partition:1]]
// read all available stream at once
impl ReadPart for PoscarFile {}
// impl partition:1 ends here
