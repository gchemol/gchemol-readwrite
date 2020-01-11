// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::{parser::*, *};
// imports:1 ends here

// atoms

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*atoms][atoms:1]]
type Point3 = [f64; 3];

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
    let (_, (symbol, _)) = read_atom_xyz(line).unwrap();
    assert_eq!("C", symbol);

    let line = "6 -11.4286 -1.3155  0.0000 0.0 0.0 0.0\n";
    let (_, (symbol, position)) = read_atom_xyz(line).unwrap();
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
                warn!("Malformed xyz format: expect {} atoms, but found {}", n, atoms.len());
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
H -9.1509  1.5395  0.0000
H -9.1509 -1.0905  0.0000
H -11.4286 -2.4055  0.0000
H -13.7062 -1.0905  0.0000
H -13.7062  1.5395  0.0000\n";

    let (_, (_, atoms)) = read_atoms_xyz(txt).unwrap();
    assert_eq!(12, atoms.len());
}
// xyz/pxyz:1 ends here
