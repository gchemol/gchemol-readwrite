// header

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*header][header:1]]
// parses the following record types in a PDB file:
//
// CRYST
// ATOM or HETATM
// TER
// END or ENDMDL
// CONECT
// header:1 ends here

// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use super::parser::*;
// imports:1 ends here

// crystal
// # References
// - [[https://www.wwpdb.org/documentation/file-format-content/format33/sect8.html][wwPDB Format version 3.3: Crystallographic and Coordinate Transformation Section]]

// # Example
// CRYST1   18.126   18.126    7.567  90.00  90.00 120.00 P6/MMM
// ORIGX1      1.000000  0.000000  0.000000        0.00000
// ORIGX2      0.000000  1.000000  0.000000        0.00000
// ORIGX3      0.000000  0.000000  1.000000        0.00000
// SCALE1      0.055169  0.031852  0.000000        0.00000
// SCALE2      0.000000  0.063704  0.000000        0.00000
// SCALE3      0.000000  0.000000  0.132153        0.00000

// # Record Format
//  COLUMNS      DATA  TYPE    FIELD          DEFINITION
//  -------------------------------------------------------------
//  1 -  6       Record name   "CRYST1"
//  7 - 15       Real(9.3)     a              a (Angstroms).
//  16 - 24      Real(9.3)     b              b (Angstroms).
//  25 - 33      Real(9.3)     c              c (Angstroms).
//  34 - 40      Real(7.2)     alpha          alpha (degrees).
//  41 - 47      Real(7.2)     beta           beta (degrees).
//  48 - 54      Real(7.2)     gamma          gamma (degrees).
//  56 - 66      LString       sGroup         Space  group.
//  67 - 70      Integer       z              Z value.

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*crystal][crystal:1]]
fn read_lattice(s: &str) -> IResult<&str, Lattice> {
    let cryst1 = tag("CRYST1");
    let read_length = map_res(take_s(9), |x| x.trim().parse::<f64>());
    let read_angle = map_res(take_s(7), |x| x.trim().parse::<f64>());
    let take1 = take_s(1);
    let take11 = take_s(11);
    do_parse!(
        s,
        cryst1 >> // CRYST1
        a     : read_length  >> // vector a
        b     : read_length  >> // vector b
        c     : read_length  >> // vector c
        alpha : read_angle   >> // angle alpha
        beta  : read_angle   >> // angle beta
        gamma : read_angle   >> // angle gamma
                take1        >> // skip one char
                take11       >> // space group
                read_line    >> // ignore remaing part
        (
            {
                let mut lat = Lattice::from_params(a, b, c, alpha, beta, gamma);

                lat
            }
        )
    )
}

#[test]
fn test_pdb_lattice() {
    let lines = "CRYST1   18.126   18.126    7.567  90.00  90.00 120.00 P6/MMM
ORIGX1      1.000000  0.000000  0.000000        0.00000
ORIGX2      0.000000  1.000000  0.000000        0.00000
ORIGX3      0.000000  0.000000  1.000000        0.00000
SCALE1      0.055169  0.031852  0.000000        0.00000
SCALE2      0.000000  0.063704  0.000000        0.00000
SCALE3      0.000000  0.000000  0.132153        0.00000
ATOM      1  O2  MOL     2      -4.808   4.768   2.469  1.00  0.00           O
ATOM      2  O3  MOL     2      -6.684   6.549   1.983  1.00  0.00           O
ATOM      3 T1   MOL     2      -5.234   6.009   1.536  1.00  0.00          Si1+
";
    let (_, mut v) = read_lattice(lines).expect("pdb lattice");
    let abc = v.lengths();
    assert_eq!(abc[1], 18.126);
}
// crystal:1 ends here

// element
// # guess element from data in columns 55-80
// 55 - 60        Real(6.2)     occupancy    Occupancy.
// 61 - 66        Real(6.2)     tempFactor   Temperature  factor.
// 77 - 78        LString(2)    element      Element symbol, right-justified.
// 79 - 80        LString(2)    charge       Charge  on the atom.

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*element][element:1]]
fn guess_element<'a>(name: &'a str, r: &'a str) -> Option<&'a str> {
    // 1. return element symbol without whitespace
    if let Some(sym) = r.get(22..24).and_then(|s| Some(s.trim())) {
        if !sym.is_empty() {
            return Some(sym);
        }
    }

    // 2. check atom name
    // ignore the first char if it is a digit
    if let Some(e) = name.chars().next() {
        if !e.is_alphabetic() {
            return name.get(1..2);
        }
    }
    return name.get(0..1);
}

#[test]
fn test_guess_element() {
    // case 1: with columns containing element symbols
    let x = guess_element("1CA ", "  1.00  0.00      UC1 SI");
    assert_eq!(Some("SI"), x);
    let x = guess_element("1CA ", "  1.00  0.00      UC1  I");
    assert_eq!(Some("I"), x);

    // case 2: without columns containing element symbols
    let x = guess_element("CA  ", "");
    assert_eq!(Some("C"), x);
    let x = guess_element("1SA  ", "");
    assert_eq!(Some("S"), x);
    let x = guess_element(" N B ", "");
    assert_eq!(Some("N"), x);
    // when the remained is just whitespace
    let x = guess_element(" H   ", "                        ");
    assert_eq!(Some("H"), x);
}
// element:1 ends here

// atom records
// # Example
// ATOM      3  SI2 SIO2X   1       3.484   3.484   3.474  1.00  0.00      UC1 SI
// # Format
// 55 - 60        Real(6.2)     occupancy    Occupancy.
// 61 - 66        Real(6.2)     tempFactor   Temperature  factor.
// 77 - 78        LString(2)    element      Element symbol, right-justified.
// 79 - 80        LString(2)    charge       Charge  on the atom.

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*atom records][atom records:1]]
// Return Atom index (sn) and Atom object
fn read_atom_record(s: &str) -> IResult<&str, (usize, Atom)> {
    let tag_atom = alt((tag("ATOM  "), tag("HETATM")));
    let take1 = take_s(1);
    let take3 = take_s(3);
    let take4 = take_s(4);
    let read_coord = map_res(take_s(8), |x| x.trim().parse::<f64>());
    let read_sn = map_res(take_s(5), |x| x.trim().parse::<usize>());
    do_parse!(
        s,
        tag_atom >> // 1-6
        sn      : read_sn >> // 7-11
                  take1   >> // 12
        name    : take4   >> // 13-16
        alt_loc : take1   >> // 17
        res_name: take3   >> // 18-20
                  take1   >> // 21
        chain_id: take1   >> // 22
        res_seq : take4   >> // 23-26
        icode   : take1   >> // 27
                  take3   >> // 28-30
        x       : read_coord   >> // 31-38
        y       : read_coord   >> // 39-46
        z       : read_coord   >> // 47-54
        rest    : read_line                                  >>
        (
            {
                // TODO: take more attributes
                let sym = guess_element(name, rest).unwrap();
                let mut a = Atom::new(sym, [x, y, z]);

                (sn, a)
            }
        )
    )
}

// Render atom in pdb format
fn format_atom(i: usize, a: &Atom) -> String {
    let [x, y, z] = a.position();
    format!(
        "ATOM  {index:>5} {name:<4}{alt_loc:1}{res_name:<3} {chain_id:1}{res_seq:>4}{icode:>1}   {x:-8.3}{y:-8.3}{z:-8.3}  1.00  0.00          {symbol:>2}\n",
        index=i,
        alt_loc=" ",
        res_name="xx",
        name=a.label(),
        chain_id=1,
        res_seq=1,
        icode=" ",
        symbol=a.symbol(),
        x=x,
        y=y,
        z=z,
    )
}

#[test]
fn test_pdb_atom() {
    let line = "ATOM      3  SI2 SIO2X   1       3.484   3.484   3.474\n";
    let (_, (i, a)) = read_atom_record(line).expect("pdb atom");
    assert_eq!(3, i);
    assert_eq!("S", a.symbol());
    assert_eq!([3.484, 3.484, 3.474], a.position());

    let line = "ATOM      3  SI2 SIO2X   1       3.484   3.484   3.474  1.00  0.00      UC1 SI\n";
    let (_, (i, a)) = read_atom_record(line).expect("pdb atom");
    assert_eq!("Si", a.symbol());

    let line = "HETATM 1632  O1S MID E   5      -6.883   5.767  26.435  1.00 26.56           O \n";
    let (_, (i, a)) = read_atom_record(line).expect("pdb atom");
    assert_eq!(1632, i);
    assert_eq!("O", a.symbol());
    assert_eq!([-6.883, 5.767, 26.435], a.position());

    let line = format_atom(3, &a);
    let (_, (i, b)) = read_atom_record(&line).expect("pdb atom");
    assert_eq!(3, i);
    assert_eq!(a.symbol(), b.symbol());
    assert_eq!(a.position(), b.position());
}

fn read_atoms(s: &str) -> IResult<&str, Vec<(usize, Atom)>> {
    let read_atom_list = many0(read_atom_record);
    do_parse!(s, atoms: read_atom_list >> (atoms))
}

#[test]
fn test_pdb_get_atoms() {
    let lines = "HETATM 1631  S   MID E   5      -5.827   4.782  25.917  1.00 24.57           S
HETATM 1634  C1  MID E   5      -3.761   3.904  27.580  1.00 28.14           C
ATOM   1634  C1  MID E   5      -3.761   3.904  27.580  1.00 28.14           C
HETATM 1641  C8  MID E   5      -2.096   3.018  29.071  1.00 30.82           C\n\n";
    let (_, atoms) = read_atoms(lines).expect("pdb atoms");
    assert_eq!(4, atoms.len());
}
// atom records:1 ends here

// bond records
// # References
// - https://www.wwpdb.org/documentation/file-format-content/format33/sect10.html

// # Format
// COLUMNS       DATA  TYPE      FIELD        DEFINITION
// -------------------------------------------------------------------------
//  1 -  6       Record name    "CONECT"
//  7 - 11       Integer        serial       Atom  serial number
// 12 - 16       Integer        serial       Serial number of bonded atom
// 17 - 21       Integer        serial       Serial  number of bonded atom
// 22 - 26       Integer        serial       Serial number of bonded atom
// 27 - 31       Integer        serial       Serial number of bonded atom

// # Example
// CONECT 1179  746 1184 1195 1203
// CONECT 1179 1211 1222
// CONECT 1021  544 1017 1020 1022

// # NOTE
// Expected to fail if atom index is larger than 9999 since neighboring numbers
// will overlap

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*bond records][bond records:1]]
fn read_bond_record(s: &str) -> IResult<&str, Vec<(usize, usize)>> {
    let tag_conect = tag("CONECT");
    let atom_sn = map_res(take_s(5), |x| x.trim().parse::<usize>());
    let atom_sn2 = map_res(take_s(5), |x| x.trim().parse::<usize>());
    let bonded_atoms = many1(atom_sn2);
    do_parse!(
        s,
        tag_conect >>                    // CONECT
        current: atom_sn       >>        // serial number of current atom
        others : bonded_atoms  >> eol >> // serial number of bonded atoms
        (
            {
                let mut pairs = vec![];
                for other in others {
                    pairs.push((current, other));
                }

                pairs
            }
        )
    )
}

fn format_bonds(mol: &Molecule) -> String {
    let mut lines = String::new();

    // connectivity
    // FIXME: add new method in molecule
    let mut map = std::collections::HashMap::new();
    for (i, j, b) in mol.bonds() {
        let mut neighbors = map.entry(i).or_insert(vec![]);
        neighbors.push((j, b.order()));
    }
    for (i, a) in mol.atoms() {
        if let Some(neighbors) = map.get(&i) {
            let mut line = format!("CONECT{:>5}", i);
            for (j, _) in neighbors {
                line.push_str(&format!("{:>5}", j));
            }
            lines.push_str(&format!("{}\n", line));
        }
    }

    lines
}

#[test]
fn test_pdb_read_bond() {
    let line = "CONECT 1179 1211 1222 \n";
    let (_, x) = read_bond_record(line).expect("pdb bond record test1");
    assert_eq!(2, x.len());

    let line = "CONECT 2041 2040 2042\n";
    let (_, x) = read_bond_record(line).unwrap();
    assert_eq!(2, x.len());

    let line = "CONECT 1179  746 11        \n";
    let (r, x) = read_bond_record(line).unwrap();
    assert_eq!(2, x.len());
}

fn read_bonds(s: &str) -> IResult<&str, Vec<(usize, usize)>> {
    let bond_list = many0(read_bond_record);
    do_parse!(
        s,
        bonds: bond_list >> (bonds.into_iter().flat_map(|x| x).collect())
    )
}

#[test]
fn test_pdb_get_bonds() {
    let lines = "CONECT 2028 2027 2029
CONECT 2041 2040 2042
CONECT 2043 2042 2044
\n";

    let (_, x) = read_bonds(lines).expect("pdb bonds");
    assert_eq!(6, x.len());

    let lines = "CONECT 2028 2027 2029
\n";

    let (_, x) = read_bonds(lines).expect("pdb missing bonds");
    assert_eq!(2, x.len());
}
// bond records:1 ends here
