// [[file:../../gchemol-readwrite.note::*imports][imports:1]]
/// Tripos Mol2 File Format
///
/// Reference
/// ---------
/// http://tripos.com/tripos_resources/fileroot/pdfs/mol2_format.pdf
/// http://chemyang.ccnu.edu.cn/ccb/server/AIMMS/mol2.pdf
///
use super::{parser::*, *};
// imports:1 ends here

// [[file:../../gchemol-readwrite.note::*atom][atom:1]]
fn read_atom_record(s: &str) -> IResult<&str, (usize, Atom)> {
    let mut optional = opt(atom_subst_and_charge);
    do_parse!(
        s,
        space0 >> id: unsigned_digit >> space1 >>  // atom index
        name: not_space >> space1 >> // atom name
        xyz : xyz_array >> space1 >> // cartesian coordinates
        emtype: mm_type >> space0 >> // Element and Atom type
        optional >> eol >>  // substructure and partial charge, which could be omitted
        ({
            let (e, mtype) = emtype;
            let mut a = Atom::new(e, xyz);
            a.set_label(name.trim());
            (id, a)
        })
    )
}

#[test]
fn test_formats_mol2_atom() {
    let line = " 3	C3	2.414	0.000	0.000	C.ar	1	BENZENE	0.000	DICT\n";
    let (r, (_, a)) = read_atom_record(line).expect("mol2 full");
    assert_eq!("C", a.symbol());

    let line = " 3	C3	2.414	0.000	0.000	C.ar	1	BENZENE	0.000\n";
    let (r, (_, a)) = read_atom_record(line).expect("mol2 atom: missing status bit");
    assert_eq!("C", a.symbol());

    let line = " 3	C3	2.414	0.000	0.000	C.ar	1	BENZENE \n";
    let (r, (_, a)) = read_atom_record(line).expect("mol2 atom: missing partial charge");
    assert_eq!("C", a.symbol());

    let line = " 3	C3	2.414	0.000	0.000	C.ar\n";
    let (r, (_, a)) = read_atom_record(line).expect("mol2 atom: missing substructure");
    assert_eq!("C", a.symbol());
}

// parse mol2 atom type. example records:
// C, C.2, C.3, C.ar
fn mm_type(s: &str) -> IResult<&str, (&str, Option<&str>)> {
    let mut mtype = opt(preceded(tag("."), alphanumeric1));
    do_parse!(
        s,
        s: alpha1 >> // element symbol
        t: mtype  >> // atom type for force field
        ((s, t))
    )
}

#[test]
fn test_mol2_mmtype() {
    let (_, (sym, mtype)) = mm_type("C.ar\n").expect("mol2 atom type");
    assert_eq!("C", sym);
    assert_eq!(Some("ar"), mtype);

    let (_, (sym, mtype)) = mm_type("C.4\n").expect("mol2 atom type 2");
    assert_eq!("C", sym);
    assert_eq!(Some("4"), mtype);

    let (_, (sym, mtype)) = mm_type("C ").expect("mol atom type: missing mm type");
    assert_eq!("C", sym);
    assert_eq!(None, mtype);
}

// substructure id and subtructure name
fn atom_subst_and_charge(s: &str) -> IResult<&str, (usize, &str, Option<f64>)> {
    let mut charge = opt(double);
    let mut status_bit = opt(alpha1);
    do_parse!(
        s,
        subst_id   : unsigned_digit >> space1 >> // xx
        subst_name : not_space      >> space1 >> // xx
        charge     : charge         >> space0 >> // xx
        status_bit : status_bit     >> // xx
        ((subst_id, subst_name, charge))
    )
}

/// simple translation without considering the bonding pattern
/// http://www.sdsc.edu/CCMS/Packages/cambridge/pluto/atom_types.html
/// I just want material studio happy to accept my .mol2 file
fn get_atom_type(atom: &Atom) -> &str {
    match atom.symbol() {
        "C" => "C.3",
        "P" => "P.3",
        "Co" => "Co.oh",
        "Ru" => "Ru.oh",
        "O" => "O.2",
        "N" => "N.3",
        "S" => "S.2",
        "Ti" => "Ti.oh",
        "Cr" => "Cr.oh",
        _ => atom.symbol(),
    }
}

fn format_atom(a: &Atom) -> String {
    let position = a.position();
    format!(
        "{name:8} {x:-12.5} {y:-12.5} {z:-12.5} {symbol:8} {subst_id:5} {subst_name:8} {charge:-6.4}\n",
        name = a.label(),
        x = position[0],
        y = position[1],
        z = position[2],
        // FIXME:
        symbol = get_atom_type(a),
        subst_id = 1,
        subst_name = "SUBUNIT",
        charge = 0.0,
    )
}
// atom:1 ends here

// [[file:../../gchemol-readwrite.note::*atoms][atoms:1]]
/// Parse Tripos Atom section
fn read_atoms(s: &str) -> IResult<&str, Vec<(usize, Atom)>> {
    let mut tag_atom = tag("@<TRIPOS>ATOM");
    let mut atoms = many1(read_atom_record);
    do_parse!(s, tag_atom >> eol >> s: atoms >> (s))
}

#[test]
fn test_mol2_get_atoms() {
    let lines = "@<TRIPOS>ATOM
      1 N           1.3863   -0.2920    0.0135 N.ar    1  UNL1       -0.2603
      2 N          -1.3863    0.2923    0.0068 N.ar    1  UNL1       -0.2603
      3 C           0.9188    0.9708   -0.0188 C.ar    1  UNL1        0.0456
      4 C          -0.4489    1.2590   -0.0221 C.ar    1  UNL1        0.0456
      5 C          -0.9188   -0.9709    0.0073 C.ar    1  UNL1        0.0456
      6 C           0.4489   -1.2591    0.0106 C.ar    1  UNL1        0.0456
      7 H           1.6611    1.7660   -0.0258 H       1  UNL1        0.0845
      8 H          -0.8071    2.2860   -0.0318 H       1  UNL1        0.0845
      9 H           0.8071   -2.2861    0.0273 H       1  UNL1        0.0845
     10 H          -1.6611   -1.7660    0.0214 H       1  UNL1        0.0845

";
    let (_, atoms) = read_atoms(lines).expect("mol2 atoms");
    assert_eq!(10, atoms.len());
}
// atoms:1 ends here

// [[file:../../gchemol-readwrite.note::*bonds][bonds:1]]
/// Parse Tripos Bond section
fn read_bonds(s: &str) -> IResult<&str, Vec<(usize, usize, Bond)>> {
    let mut tag_bond = tag("@<TRIPOS>BOND");
    let mut bonds = many0(read_bond_record);
    do_parse!(s, tag_bond >> eol >> bonds: bonds >> (bonds))
}

#[test]
fn test_mol2_bonds() {
    let lines = "@<TRIPOS>BOND
     1    13    11    1
     2    11    12    1
     3     8     4    1
     4     7     3    1
     5     4     3   ar

";

    let (_, x) = read_bonds(lines).expect("mol2 bonds");
    assert_eq!(5, x.len());
}

fn read_bond_record(s: &str) -> IResult<&str, (usize, usize, Bond)> {
    do_parse!(
        s,
        space0 >>                        // ignore leading space
        unsigned_digit >> space1 >>      // bond_id
        n1: unsigned_digit >> space1 >>  // origin_atom_id
        n2: unsigned_digit >> space1 >>  // target_atom_id
        bo: alphanumeric1  >> space0 >>  // bond_type
        read_line >>                     // ignore status_bits
        ({
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
        })
    )
}

#[test]
fn test_formats_mol2_bond_record() {
    let (_, (i, j, b)) = read_bond_record("1	1	2	1 BACKBONE\n").expect("mol2 bond: full");
    assert_eq!(BondKind::Single, b.kind());

    let (_, (i, j, b)) = read_bond_record("1	1	2	1\n").expect("mol2 bond: missing status bits");
    assert_eq!(BondKind::Single, b.kind());

    let (_, (i, j, b)) = read_bond_record("1	1	2	ar\n").expect("mol2 bond: aromatic bond type");
    assert_eq!(BondKind::Aromatic, b.kind());
}

fn format_bond_order(bond: &Bond) -> &str {
    match bond.kind() {
        BondKind::Single => "1",
        BondKind::Double => "2",
        BondKind::Triple => "3",
        BondKind::Quadruple => "4",
        BondKind::Aromatic => "ar",
        BondKind::Partial => "wk", // gaussian view use this
        BondKind::Dummy => "nc",
    }
}
// bonds:1 ends here

// [[file:../../gchemol-readwrite.note::*lattice][lattice:1]]
fn read_lattice(s: &str) -> IResult<&str, Lattice> {
    let tag_crysin = tag("@<TRIPOS>CRYSIN");
    do_parse!(
        s,
        tag_crysin >> eol >>    // goto section header
        space0 >>               // ignore leading spaces
        a: double >> space1 >>  // a
        b: double >> space1 >>  // b
        c: double >> space1 >>  // c
        alpha: double >> space1 >> // angle alpha
        beta : double >> space1 >> // angle beta
        gamma: double >> space1 >> // angle gamma
        space_grp : unsigned_digit >> space1    >>
        setting   : unsigned_digit >> read_line >>
        ({
            Lattice::from_params(a, b, c, alpha, beta, gamma)
        })
    )
}

#[test]
fn test_formats_mol2_crystal() {
    let txt = "@<TRIPOS>CRYSIN
 12.312000 4.959000 15.876000 90.000000 99.070000 90.000000 4 1\n";

    let (_, mut x) = read_lattice(txt).expect("mol2 crystal");
    assert_eq!([12.312, 4.959, 15.876], x.lengths());
}
// lattice:1 ends here

// [[file:../../gchemol-readwrite.note::*parse][parse:1]]
fn read_molecule(s: &str) -> IResult<&str, Molecule> {
    let mut jump = opt(take_until("@<TRIPOS>"));
    let mut read_bonds = opt(read_bonds);
    let mut read_lattice = opt(read_lattice);
    do_parse!(
        s,
        jump >> meta: read_molecule_meta >> // meta data
        jump >> atoms: read_atoms >>        // atoms
        jump >> bonds: read_bonds >>        // optional bonds
        jump >> lattice: read_lattice >>    // optional lattice
        ({
            let (title, natoms, nbonds) = meta;
            let mut mol = Molecule::new(title);

            // assign atoms
            if natoms != atoms.len() {
                warn!("Inconsistency: expected {} atoms, but found {}", natoms, atoms.len());
            }
            mol.add_atoms_from(atoms);

            // assign bonds
            if let Some(bonds) = bonds {
                mol.add_bonds_from(bonds);
            }

            // assign lattice
            if let Some(lattice) = lattice {
                mol.set_lattice(lattice);
            }

            mol
        })
    )
}

fn read_counts(s: &str) -> IResult<&str, (usize, Option<usize>)> {
    let mut opt_num_bonds = opt(unsigned_digit);
    do_parse!(
        s,
        space0 >> n: unsigned_digit >> space0 >> m: opt_num_bonds >> read_line >> ((n, m))
    )
}

fn read_molecule_meta(s: &str) -> IResult<&str, (&str, usize, Option<usize>)> {
    let mut tag_mol = tag("@<TRIPOS>MOLECULE");
    do_parse!(
        s,
        tag_mol >> eol >>        // section header
        title: read_until_eol >> // mol_name
        counts: read_counts   >> // num_aatoms, num_bonds
        read_line >>             // ignore mol_type
        read_line >>             // ignore charge_type
        ({
            let (natoms, nbonds) = counts;
            (title, natoms, nbonds)
        })
    )
}

#[test]
fn test_mol2_meta() {
    let txt = "@<TRIPOS>MOLECULE
Molecule Name
5 4
SMALL
NO_CHARGES

";
    let (_, (title, natoms, nbonds)) = read_molecule_meta(txt).expect("mol2 meta");
    assert_eq!(title, "Molecule Name");
    assert_eq!(natoms, 5);
    assert_eq!(nbonds, Some(4));
}
// parse:1 ends here

// [[file:../../gchemol-readwrite.note::*format][format:1]]
fn format_molecule(mol: &Molecule) -> Result<String> {
    let natoms = mol.natoms();
    let nbonds = mol.nbonds();

    let mut lines = String::new();
    lines += "#	Created by gchemol\n\n";
    lines += "@<TRIPOS>MOLECULE\n";
    lines += &format!("{}\n", mol.title());

    // atom count, bond numbers, substructure numbers
    lines += &format!("{:>5} {:>5}\n", natoms, nbonds);
    // molecule type
    lines += "SMALL\n";
    // customed charges
    lines += "USER CHARGES\n";
    // atoms
    lines += "@<TRIPOS>ATOM\n";

    // format atoms
    for (i, a) in mol.atoms() {
        lines += &format!("{:5} {}", i, format_atom(&a));
    }

    // format bonds
    if nbonds > 0 {
        lines += "@<TRIPOS>BOND\n";
        let mut sn = 1;
        for (i, j, b) in mol.bonds() {
            lines += &format!(
                "{sn:4} {bond_i:4} {bond_j:4} {bond_type:3}\n",
                sn = sn,
                bond_i = i,
                bond_j = j,
                bond_type = format_bond_order(&b)
            );
            sn += 1;
        }
    }

    // format crystal
    if let Some(lat) = &mol.lattice {
        lines += "@<TRIPOS>CRYSIN\n";
        let [a, b, c] = lat.lengths();
        let [alpha, beta, gamma] = lat.angles();
        lines += &format!(
            "{a:10.4} {b:10.4} {c:10.4} {alpha:5.2} {beta:5.2} {gamma:5.2} {sgrp} 1\n",
            a = a,
            b = b,
            c = c,
            alpha = alpha,
            beta = beta,
            gamma = gamma,
            // FIXME: crystal space group
            sgrp = 4
        );
    }

    // Final blank line
    lines += "\n";

    Ok(lines)
}
// format:1 ends here

// [[file:../../gchemol-readwrite.note::*impl chemfile][impl chemfile:1]]
#[derive(Copy, Clone, Debug)]
pub struct Mol2File();

impl ChemicalFile for Mol2File {
    fn ftype(&self) -> &str {
        "text/mol2"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".mol2"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        format_molecule(mol)
    }
}

impl ParseMolecule for Mol2File {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        let (_, mol) = read_molecule(input).map_err(|e| format_err!("{:}", e))?;
        Ok(mol)
    }

    /// Skip reading some lines.
    fn pre_read_hook<R: BufRead + Seek>(&self, mut r: TextReader<R>) -> TextReader<R>
    where
        Self: Sized,
    {
        r.seek_line(|line| line.starts_with("@<TRIPOS>MOLECULE"));
        r
    }
}
// impl chemfile:1 ends here

// [[file:../../gchemol-readwrite.note::*new][new:1]]
impl ReadPart for Mol2File {
    fn read_next(&self, context: ReadContext) -> ReadAction {
        Preceded(|line: &str| line.starts_with("@<TRIPOS>MOLECULE")).read_next(context)
    }
}

#[test]
fn test_mol2() -> Result<()> {
    let f = "./tests/files/mol2/multi-obabel.mol2";
    let r = TextReader::from_path(f)?;
    let mols = Mol2File().parse_molecules(r);
    for m in mols {
        dbg!(m?.natoms());
    }

    Ok(())
}
// new:1 ends here
