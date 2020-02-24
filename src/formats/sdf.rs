// header

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*header][header:1]]
// MDL SD file format
//
// SD file format reference
// ------------------------
// Ctab block format for V2000
// - http://download.accelrys.com/freeware/ctfile-formats/ctfile-formats.zip
// header:1 ends here

// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::parser::*;
use super::*;
// imports:1 ends here

// counts line

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*counts%20line][counts line:1]]
// aaabbblllfffcccsssxxxrrrpppiiimmmvvvvvv
// aaa = number of atoms
// bbb = number of bonds
fn counts_line(s: &str) -> IResult<&str, (usize, usize)> {
    let read_count = map_res(take_s(3), |x: &str| x.trim().parse::<usize>());
    do_parse!(
        s,
        na: read_count >> // number of atoms
        nb: read_count >> // number of bonds
        read_line >>  // ignore the remaining
        ((na, nb))
    )
}

#[test]
fn test_sdf_counts_line() {
    let line = " 16 14  0  0  0  0  0  0  0  0999 V2000\n";
    let (_, (na, nb)) = counts_line(line).expect("sdf counts line");
    assert_eq!(16, na);
    assert_eq!(14, nb);
}
// counts line:1 ends here

// atoms

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*atoms][atoms:1]]
// Example input
// -------------
//    -1.2940   -0.5496   -0.0457 C   0  0  0  0  0  0  0  0  0  0  0  0
fn get_atom_from(s: &str) -> IResult<&str, Atom> {
    let read_coord = map_res(take_s(10), |x| x.trim().parse::<f64>());
    let read_symbol = map(take_s(3), |x| x.trim());
    do_parse!(
        s,
        x: read_coord  >> // x coords
        y: read_coord  >> // y coords
        z: read_coord  >> // z coords
        s: read_symbol >> // element symbol
        read_line >>      // ignore remaing part
        ({
            Atom::new(s, [x, y, z])
        })
    )
}

// output atom line in .sdf format
fn format_atom(i: usize, a: &Atom) -> String {
    let pos = a.position();
    format!(
        "{x:-10.4} {y:-9.4} {z:-9.4} {sym:3} 0  0  0  0  0  0  0  0  0 {index:2}\n",
        x = pos[0],
        y = pos[1],
        z = pos[2],
        sym = a.symbol(),
        index = i,
    )
}

#[test]
fn test_sdf_atom() {
    let line = "  -13.5661  206.9157  111.5569 C   0  0  0  0  0  0  0  0  0 12 \n\n";
    let (_, a) = get_atom_from(line).expect("sdf atom");
    let line2 = format_atom(12, &a);
    assert_eq!(line[..60], line2[..60]);
}
// atoms:1 ends here

// bonds
// bond type mapping:
// : 1: "Single",
// : 2: "Double",
// : 3: "Triple",
// : 4: "Aromatic",
// : 5: "Single_or_Double",
// : 6: "Single_or_Aromatic",
// : 7: "Double_or_Aromatic",
// : 8: "Any"

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*bonds][bonds:1]]
//   1  4  1  0  0  0  0
fn get_bond_from(s: &str) -> IResult<&str, (usize, usize, Bond)> {
    let read_number = map_res(take_s(3), |x| x.trim().parse::<usize>());
    do_parse!(
        s,
        i: read_number >> // atom i
        j: read_number >> // atom j
        b: read_number >> read_line >> // bond order
        ({
            let bond = match b {
                1 => Bond::single(),
                2 => Bond::double(),
                3 => Bond::triple(),
                4 => Bond::aromatic(),
                _ => {
                    warn!("ignore sdf bond type: {}", b);
                    Bond::single()
                },
            };
            (i, j, bond)
        })
    )
}

use std::fmt::Display;
fn format_bond<T: Display>(index1: T, index2: T, bond: &Bond) -> String {
    format!(
        "{index1:>3}{index2:>3}{order:3}  0  0  0 \n",
        index1 = index1,
        index2 = index2,
        order = 1
    )
}

#[test]
fn test_sdf_bond() {
    let line = "  6  7  1  0  0  0 \n";
    let (_, (index1, index2, bond)) = get_bond_from(line).expect("sdf bond");
    let line2 = format_bond(index1, index2, &bond);
    assert_eq!(line[..9], line2[..9]);
}
// bonds:1 ends here

// molecule

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*molecule][molecule:1]]
pub fn get_molecule_from(input: &str) -> IResult<&str, Molecule> {
    let read_atoms = many1(get_atom_from);
    let read_bonds = many0(get_bond_from);
    let (input, mol) = do_parse!(
        input,
        title   : read_line     >> // molecule title
        software: read_line     >> // version?
        comment : read_line     >> // user comments
        counts  : counts_line   >> // number of atoms and bonds
        atoms   : read_atoms    >> // atoms
        bonds   : read_bonds    >> // bonds
        (
            {
                let naa = atoms.len();
                let nbb = bonds.len();
                let (na, nb) = counts;
                if na != naa {
                    eprintln!("expect {} atoms, but found {}", na, naa);
                }
                if nb != nbb {
                    eprintln!("expect {} bonds, but found {}", nb, nbb);
                }

                let mut mol = Molecule::from_atoms(atoms);
                mol.set_title(title);

                for (i, j, b) in bonds {
                    mol.add_bond(i, j, b);
                }
                mol
            }
        )
    )?;

    Ok((input, mol))
}

fn format_molecule(mol: &Molecule) -> String {
    let mut lines = String::new();

    // molecule title
    lines.push_str(&format!("{}\n", mol.title()));
    // software
    lines.push_str("gchemol\n");
    // comment
    lines.push_str("\n");
    // counts line
    let line = format!(
        "{natoms:3}{nbonds:3}  0  0  0  0  0  0  0  0999 V2000 \n",
        natoms = mol.natoms(),
        nbonds = mol.nbonds()
    );

    lines.push_str(&line);

    for (i, a) in mol.atoms() {
        lines.push_str(&format_atom(i, a));
    }

    for (i, j, b) in mol.bonds() {
        lines.push_str(&format_bond(i, j, &b));
    }

    lines.push_str("M  END\n$$$$\n");

    lines
}
// molecule:1 ends here

// chemfile

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*chemfile][chemfile:1]]
#[derive(Clone, Copy, Debug)]
pub struct SdfFile();

impl ChemicalFile for SdfFile {
    fn ftype(&self) -> &str {
        "text/sdf"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".sd", ".sdf", ".mol"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        if mol.lattice.is_some() {
            eprintln!("WARNING: cannot render Lattice in SDF format!");
        }
        Ok(format_molecule(mol))
    }
}

impl ParseMolecule for SdfFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        let (_, mol) = get_molecule_from(input).map_err(|e| format_err!("parse SDF format failure: {:?}", e))?;
        Ok(mol)
    }
}
// chemfile:1 ends here

// new

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*new][new:1]]
impl ReadPart for SdfFile {
    fn read_next(&self, context: ReadContext) -> ReadAction {
        Terminated(|line: &str| line == "$$$$\n").read_next(context)
    }
}
// new:1 ends here
