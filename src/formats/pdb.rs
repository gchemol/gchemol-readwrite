// [[file:../../gchemol-readwrite.note::*header][header:1]]
// parses the following record types in a PDB file:
//
// CRYST1
// ATOM or HETATM
// TER
// END or ENDMDL
// CONECT
// header:1 ends here

// [[file:../../gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use super::parser::*;
// imports:1 ends here

// [[file:../../gchemol-readwrite.note::*crystal][crystal:1]]
fn read_lattice(s: &str) -> IResult<&str, Lattice> {
    let mut cryst1 = tag("CRYST1");
    let mut read_length = map_res(take_s(9), |x| x.trim().parse::<f64>());
    let mut read_angle = map_res(take_s(7), |x| x.trim().parse::<f64>());
    let mut take1 = take_s(1);
    let mut take11 = take_s(11);
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

// [[file:../../gchemol-readwrite.note::*element][element:1]]
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

// [[file:../../gchemol-readwrite.note::*atom records][atom records:1]]
// Return Atom index (sn) and Atom object
fn read_atom_record(s: &str) -> IResult<&str, (usize, Atom)> {
    let mut tag_atom = alt((tag("ATOM  "), tag("HETATM")));
    let mut take1 = take_s(1);
    let mut take3 = take_s(3);
    let mut take4 = take_s(4);
    let mut read_coord = map_res(take_s(8), |x| x.trim().parse::<f64>());
    let mut read_sn = map_res(take_s(5), |x| x.trim().parse::<usize>());
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
    let mut read_atom_list = many0(read_atom_record);
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

// [[file:../../gchemol-readwrite.note::*bond records][bond records:1]]
fn read_bond_record(s: &str) -> IResult<&str, Vec<(usize, usize)>> {
    let mut tag_conect = tag("CONECT");
    let mut atom_sn = map_res(take_s(5), |x| x.trim().parse::<usize>());
    let mut atom_sn2 = map_res(take_s(5), |x| x.trim().parse::<usize>());
    let mut bonded_atoms = many1(atom_sn2);
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
    let mut bond_list = many1(read_bond_record);
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

// [[file:../../gchemol-readwrite.note::*parse][parse:1]]
// quick jump to starting position
fn jump1(s: &str) -> IResult<&str, ()> {
    let possible_tags = alt((tag("CRYST1"), tag("ATOM  "), tag("HETATM")));
    let (r, _) = many_till(read_line, peek(possible_tags))(s)?;

    Ok((r, ()))
}

fn read_molecule(s: &str) -> IResult<&str, Molecule> {
    let mut read_lattice = opt(read_lattice);
    let mut read_bonds = opt(read_bonds);
    // recognize optional record between Atom and Bond
    let mut sep_atoms_bonds = opt(alt((preceded(tag("TER"), read_line), tag("END\n"))));
    do_parse!(
        s,
        jump1 >>             // seeking
        lat: read_lattice >> // crystal info, optional
        atoms: read_atoms >> // atoms, required
        sep_atoms_bonds   >> // separator, optinal
        bonds: read_bonds >> // bonds, optional
        ({
            // assign atoms
            let mut mol = Molecule::new("for pdb");
            mol.add_atoms_from(atoms);

            // assign lattice
            if let Some(lat) = lat {
                mol.set_lattice(lat);
            }

            // assign bonds
            if let Some(bonds) = bonds {
                // FIXME: bond type
                let bonds = bonds.into_iter().map(|(u, v)| (u, v, Bond::single()));
                mol.add_bonds_from(bonds);
            }
            mol
        })
    )
}

#[test]
fn test_pdb_molecule() {
    let lines = "\
SCALE3      0.000000  0.000000  0.132153        0.00000
ATOM      1  O2  MOL     2      -4.808   4.768   2.469  1.00  0.00           O
ATOM      2  O3  MOL     2      -6.684   6.549   1.983  1.00  0.00           O
ATOM      3 T1   MOL     2      -5.234   6.009   1.536  1.00  0.00          Si1+
ATOM      4  O1  MOL     2      -4.152  10.936   1.688  1.00  0.00           O
ATOM      5  O1  MOL     2      -4.150  10.935   1.688  1.00  0.00           O
ATOM      6  O2  MOL     2      -1.725  11.578   2.469  1.00  0.00           O
ATOM      7  O2  MOL     2      -9.164  10.843   2.469  1.00  0.00           O
ATOM      8 T1   MOL     2      -2.587  10.589   1.536  1.00  0.00          Si1+
ATOM      9 T1   MOL     2      -7.877  10.591   1.536  1.00  0.00          Si1+
ATOM     10  O2  MOL     2      -1.725  -6.548   2.469  1.00  0.00           O
ATOM     11  O3  MOL     2      -2.330  -9.063   1.983  1.00  0.00           O
ATOM     12 T1   MOL     2      -2.587  -7.537   1.536  1.00  0.00          Si1+
ATOM     13  O1  MOL     2      -7.395  -9.064   1.688  1.00  0.00           O
TER     367
CONECT    2    4
CONECT    3    4
END\n\n";

    let (r, v) = read_molecule(lines).expect("pdb molecule");
    assert_eq!(13, v.natoms());
    assert_eq!(2, v.nbonds());

    let lines = "\
REMARK   Created:  2018-10-22T12:36:28Z
SCALE3      0.000000  0.000000  0.132153        0.00000
ATOM      1  O2  MOL     2      -4.808   4.768   2.469  1.00  0.00           O
ATOM      2  O3  MOL     2      -6.684   6.549   1.983  1.00  0.00           O
ATOM      3 T1   MOL     2      -5.234   6.009   1.536  1.00  0.00          Si1+
ATOM      4  O1  MOL     2      -4.152  10.936   1.688  1.00  0.00           O
ATOM      5  O1  MOL     2      -4.150  10.935   1.688  1.00  0.00           O
ATOM      6  O2  MOL     2      -1.725  11.578   2.469  1.00  0.00           O
ATOM      7  O2  MOL     2      -9.164  10.843   2.469  1.00  0.00           O
ATOM      8 T1   MOL     2      -2.587  10.589   1.536  1.00  0.00          Si1+
ATOM      9 T1   MOL     2      -7.877  10.591   1.536  1.00  0.00          Si1+
ATOM     10  O2  MOL     2      -1.725  -6.548   2.469  1.00  0.00           O
ATOM     11  O3  MOL     2      -2.330  -9.063   1.983  1.00  0.00           O
ATOM     12 T1   MOL     2      -2.587  -7.537   1.536  1.00  0.00          Si1+
ATOM     13  O1  MOL     2      -7.395  -9.064   1.688  1.00  0.00           O
\n\n\n";

    let (r, v) = read_molecule(&lines).expect("pdb molecule no bonds");
    assert_eq!(13, v.natoms());
    assert_eq!(0, v.nbonds());
    let txt = "\
CRYST1   54.758   54.758   55.584  90.00  90.00  90.00 P 1           1
ATOM      1  SI1 SIO2X   1       1.494   1.494   0.000  1.00  0.00      UC1 SI
ATOM      2  O11 SIO2X   1       1.194   0.514   1.240  1.00  0.00      UC1  O
ATOM      3  SI2 SIO2X   1       3.484   3.484   3.474  1.00  0.00      UC1 SI
ATOM      4  O12 SIO2X   1       3.784   4.464   4.714  1.00  0.00      UC1  O
ATOM      5  SI3 SIO2X   1       0.995   3.983   1.737  1.00  0.00      UC1 SI
ATOM      6  O13 SIO2X   1       1.975   3.683   2.977  1.00  0.00      UC1  O
ATOM      7  SI4 SIO2X   1       3.983   0.995   5.211  1.00  0.00      UC1 SI
ATOM      8  O14 SIO2X   1       3.003   1.295   6.451  1.00  0.00      UC1  O
ATOM      9  O21 SIO2X   1       1.295   3.003   0.497  1.00  0.00      UC1  O
ATOM     10  O22 SIO2X   1       3.683   1.975   3.971  1.00  0.00      UC1  O
ATOM     11  O23 SIO2X   1       0.514   1.194   5.708  1.00  0.00      UC1  O
ATOM     12  O24 SIO2X   1       4.464   3.784   2.234  1.00  0.00      UC1  O
END\n
";
    let (_, v) = read_molecule(&txt).expect("pdb crystal");
    assert_eq!(12, v.natoms());
    assert!(v.lattice.is_some());
}
// parse:1 ends here

// [[file:../../gchemol-readwrite.note::ccd72c38][ccd72c38]]
fn format_molecule(mol: &Molecule) -> String {
    if mol.natoms() > 9999 {
        warn!("PDB format is incapable for large molecule (natoms < 9999)");
    }

    // atoms
    let mut lines = String::from("REMARK Created by gchemol\n");
    // write crystal info
    if let Some(lat) = mol.get_lattice() {
        let [a, b, c] = lat.lengths();
        let [alpha, beta, gamma] = lat.angles();
        lines.push_str(&format!("CRYST1{a:9.4}{b:9.4}{c:9.4}{alpha:7.2}{beta:7.2}{gamma:7.2} P1            1\n"))
    }
    for (i, a) in mol.atoms() {
        let line = format_atom(i, a);
        lines.push_str(&line);
    }

    // bonds
    if mol.nbonds() > 0 {
        lines.push_str(&format_bonds(&mol));
    }

    lines.push_str("END\n");

    lines
}
// ccd72c38 ends here

// [[file:../../gchemol-readwrite.note::*chemfile][chemfile:1]]
#[derive(Clone, Copy, Debug)]
pub struct PdbFile();

impl ChemicalFile for PdbFile {
    fn ftype(&self) -> &str {
        "text/pdb"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".pdb", ".ent"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        Ok(format_molecule(mol))
    }
}

impl ParseMolecule for PdbFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        let (_, mol) = read_molecule(input).map_err(|e| format_err!("parse PDB format failure: {:?}", e))?;
        Ok(mol)
    }
}
// chemfile:1 ends here

// [[file:../../gchemol-readwrite.note::cc0cbfc6][cc0cbfc6]]
impl ReadPart for PdbFile {
    // for multi-model records
    fn read_next(&self, context: ReadContext) -> ReadAction {
        Terminated(|line: &str| line == "ENDMDL\n").read_next(context)
    }
}

impl PdbFile {
    pub fn partitions<R: BufRead + Seek>(&self, mut r: TextReader<R>) -> Result<impl Iterator<Item = String>> {
        Ok(r.partitions(*self))
    }
}
// cc0cbfc6 ends here
