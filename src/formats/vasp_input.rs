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

// chemfile

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*chemfile][chemfile:1]]
#[derive(Clone, Copy, Debug)]
pub struct PoscarFile();

impl ChemicalFile for PoscarFile {
    fn ftype(&self) -> &str {
        "vasp/poscar"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec!["POSCAR", "CONTCAR", ".poscar", ".vasp"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        // Ok(format_molecule(mol))
        todo!()
    }
}

// read all available stream at once
impl Partition for PoscarFile {}

impl ParseMolecule for PoscarFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        // parse_molecule(input)
        todo!()
    }
}
// chemfile:1 ends here