// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
/// Tripos Mol2 File Format
///
/// Reference
/// ---------
/// http://tripos.com/tripos_resources/fileroot/pdfs/mol2_format.pdf
/// http://chemyang.ccnu.edu.cn/ccb/server/AIMMS/mol2.pdf
///

use text_parser::parsers::*;

use super::*;
// imports:1 ends here

// bond
// # Sample record
// @<TRIPOS>BOND
//   12	6	12	1
//   	6	5	6	ar
//   5	4	9	am	BACKBONE

// # Format
// bond_id origin_atom_id target_atom_id bond_type [status_bits]


// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*bond][bond:1]]
/// Parse Tripos Bond section
named!(read_bonds<&str, Vec<(usize, usize, Bond)>>, do_parse!(
           tag!("@<TRIPOS>BOND")       >> eol >>
    bonds: many0!(read_bond_record)    >>
    (
        bonds
    )
));

#[test]
fn test_mol2_bonds() {
    let lines = "\
@<TRIPOS>BOND
     1    13    11    1
     2    11    12    1
     3     8     4    1
     4     7     3    1
     5     4     3   ar

";

    let (_, x) = read_bonds(lines).expect("mol2 bonds");
    assert_eq!(5, x.len());
}

named!(read_bond_record<&str, (usize, usize, Bond)>, sp!(do_parse!(
        unsigned_digit >>       // bond_id
    n1: unsigned_digit >>       // origin_atom_id
    n2: unsigned_digit >>       // target_atom_id
    bo: alphanumeric   >>       // bond_type
        read_line      >>       // status_bits
    (
        {
            let bond = match bo.to_lowercase().as_ref() {
                "1"  => Bond::single(),
                "2"  => Bond::double(),
                "3"  => Bond::triple(),
                "ar" => Bond::aromatic(),
                "am" => Bond::aromatic(),
                "nc" => Bond::dummy(),
                "wk" => Bond::partial(), // gaussian view use this
                _    => Bond::single()
            };
            (n1, n2, bond)
        }
    )
)));

#[test]
fn test_formats_mol2_bond_record() {
    let (_, (i, j, b)) = read_bond_record("1	1	2	1 BACKBONE\n").expect("mol2 bond: full");
    assert_eq!(BondKind::Single, b.kind);

    let (_, (i, j, b)) = read_bond_record("1	1	2	1\n").expect("mol2 bond: missing status bits");
    assert_eq!(BondKind::Single, b.kind);

    let (_, (i, j, b)) = read_bond_record("1	1	2	ar\n").expect("mol2 bond: aromatic bond type");
    assert_eq!(BondKind::Aromatic, b.kind);
}

fn format_bond_order(bond: &Bond) -> &str {
    match bond.kind {
        BondKind::Single => "1",
        BondKind::Double => "2",
        BondKind::Triple => "3",
        BondKind::Quadruple => "4",
        BondKind::Aromatic => "ar",
        BondKind::Partial => "wk", // gaussian view use this
        BondKind::Dummy => "nc",
    }
}
// bond:1 ends here

// lattice
// # Format
// @<TRIPOS>CRYSIN
// cell cell cell cell cell cell space_grp setting

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*lattice][lattice:1]]
named!(read_lattice<&str, Lattice>, sp!(do_parse!(
                tag!("@<TRIPOS>CRYSIN") >>
                tag!("\n")              >>
    a         : double                  >>
    b         : double                  >>
    c         : double                  >>
    alpha     : double                  >>
    beta      : double                  >>
    gamma     : double                  >>
    space_grp : unsigned_digit          >>
    setting   : unsigned_digit          >>
                read_line               >>
    (Lattice::from_params(a, b, c, alpha, beta, gamma))
)));

#[test]
fn test_formats_mol2_crystal() {
    let txt = "\
@<TRIPOS>CRYSIN
12.312000 4.959000 15.876000 90.000000 99.070000 90.000000 4 1\n";

    let (_, mut x) = read_lattice(txt)
        .expect("mol2 crystal");
    assert_eq!([12.312, 4.959, 15.876], x.lengths());
}
// lattice:1 ends here

// molecule
// # Format
// - mol_name
// - num_atoms [num_bonds [num_subst [num_feat [num_sets]]]]
// - mol_type
// - charge_type
// - [status_bits [mol_comment]]

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*molecule][molecule:1]]
fn read_molecule(input: &str) -> IResult<&str, Molecule> {
    let (input, _) = read_lines_until(input, "@<TRIPOS>MOLECULE")?;

    // 1. read meta data
    let (input, (mut mol, counts)) = sp!(input, do_parse!(
                      tag!("@<TRIPOS>MOLECULE") >> eol >>
        title       : read_line                 >>
        counts      : read_usize_many           >>
        mol_type    : read_line                 >>
        charge_type : read_line                 >>
        (
            {
                let mut mol = Molecule::new(title);

                (mol, counts)
            }
        )
    ))?;

    // 2. Assign atoms
    let (input, _) = read_lines_until(input, "@<TRIPOS>ATOM")?;
    let (input, atoms) = read_atoms(input)?;
    let natoms = counts[0];
    if natoms != atoms.len() {
        eprintln!("Inconsistency: expected {} atoms, but found {}", natoms, atoms.len());
    }
    // assign atoms
    let mut table = HashMap::new();
    for (i, a) in atoms {
        let n = mol.add_atom(a);
        table.insert(i, n);
    }

    // 3. Assign bonds (optional)
    let (input, current) = peek_line(input)?;
    let input = if current.starts_with("@<TRIPOS>BOND") {
        let (input, bonds) = read_bonds(input)?;
        for (i, j, b) in bonds {
            let ni = table.get(&i).expect(".mol2 file: look up atom in bond record");
            let nj = table.get(&j).expect(".mol2 file: look up atom in bond record");
            mol.add_bond(*ni, *nj, b);
        }
        input
    } else {
        input
    };

    // 4. Crystal (optional)
    let (input, _) = many_till!(input, read_line, peek!(
        alt!(
            tag!("@<TRIPOS>CRYSIN") |
            tag!("@<TRIPOS>MOLECULE") |
            eof
        )
    ))?;

    let (input, current) = peek_line(input)?;
    let input = if current.starts_with("@<TRIPOS>CRYSIN") {
        let (input, lat) = read_lattice(input)?;
        mol.set_lattice(lat);
        input
    } else {
        input
    };

    Ok((input, mol))
}
// molecule:1 ends here

// chemfile

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*chemfile][chemfile:1]]
pub struct Mol2File();

impl ChemFileLike for Mol2File {
    fn ftype(&self) -> &str {
        "text/mol2"
    }

    fn extensions(&self) -> Vec<&str> {
        vec![".mol2"]
    }

    fn parse_molecule<'a>(&self, chunk: &'a str) -> IResult<&'a str, Molecule> {
        read_molecule(chunk)
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        let natoms = mol.natoms();
        let nbonds = mol.nbonds();

        let mut lines = String::new();
        lines.push_str("#	Created by:	gchemol\n");
        lines.push_str("\n");
        lines.push_str("@<TRIPOS>MOLECULE\n");
        lines.push_str(&format!("{}\n", mol.name));

        // atom count, bond numbers, substructure numbers
        lines.push_str(&format!("{:>5} {:>5}\n",
                                natoms,
                                nbonds));
        // molecule type
        lines.push_str("SMALL\n");
        // customed charges
        lines.push_str("USER CHARGES\n");
        // atoms
        lines.push_str("@<TRIPOS>ATOM\n");

        // format atoms
        for (i, a) in mol.view_atoms() {
            let line = format!("{:5} {}", i, format_atom(&a));
            lines.push_str(&line);
        }

        // format bonds
        if nbonds > 0 {
            lines.push_str("@<TRIPOS>BOND\n");
            let mut sn = 1;
            for (i, j, b) in mol.view_bonds() {
                let line = format!("{sn:4} {bond_i:4} {bond_j:4} {bond_type:3}\n",
                                   sn        = sn,
                                   bond_i    = i,
                                   bond_j    = j,
                                   bond_type = format_bond_order(&b));
                lines.push_str(&line);
                sn += 1;
            }
        }

        // format crystal
        if let Some(mut lat) = &mol.lattice {
            lines.push_str("@<TRIPOS>CRYSIN\n");
            let [a, b, c] = lat.lengths();
            let [alpha, beta, gamma] = lat.angles();
            let line = format!("{a:10.4} {b:10.4} {c:10.4} {alpha:5.2} {beta:5.2} {gamma:5.2} {sgrp} 1\n",
                               a     = a,
                               b     = b,
                               c     = c,
                               alpha = alpha,
                               beta  = beta,
                               gamma = gamma,
                               // FIXME: crystal space group
                               sgrp  = 4);

            lines.push_str(&line);
        }

        // final blank line
        lines.push_str("\n");

        Ok(lines)
    }
}
// chemfile:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*test][test:1]]
#[test]
fn test_formats_mol2() {
    let file = Mol2File();

    let mols = file.parse(Path::new("tests/files/mol2/ch3f-dos.mol2")).expect("mol2 ch3f");
    assert_eq!(1, mols.len());

    // when missing final blank line
    // gaussview generated .mol2 file
    let mols = file.parse(Path::new("tests/files/mol2/alanine-gv.mol2")).expect("gv generated mol2 file");
    assert_eq!(1, mols.len());
    let mol = &mols[0];
    assert_eq!(12, mol.natoms());
    assert_eq!(11, mol.nbonds());

    // molecule trajectory
    // openbabel converted .mol2 file
    let mols = file.parse(Path::new("tests/files/mol2/multi-obabel.mol2")).expect("mol2 multi");

    let natoms_expected = vec![16, 10, 16, 16, 16, 13];
    let natoms: Vec<_> = mols.iter().map(|m| m.natoms()).collect();
    assert_eq!(natoms_expected, natoms);

    let nbonds_expected = vec![14, 10, 14, 14, 14, 12];
    let nbonds: Vec<_> = mols.iter().map(|m| m.nbonds()).collect();
    assert_eq!(nbonds_expected, nbonds);
    assert_eq!(6, mols.len());

    // single molecule with a lattice
    // discovery studio generated .mol2 file
    let mols = file.parse(Path::new("tests/files/mol2/LTL-crysin-ds.mol2")).expect("mol2 crysin");
    assert_eq!(1, mols.len());
    assert!(mols[0].lattice.is_some());

}
// test:1 ends here
