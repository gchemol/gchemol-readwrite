// [[file:../../gchemol-readwrite.note::fd421c85][fd421c85]]
use super::parser::*;
use super::{Atom, Lattice, Molecule};
use gut::prelude::*;
// fd421c85 ends here

// [[file:../../gchemol-readwrite.note::68eb9728][68eb9728]]
fn read_head_lines(s: &str) -> IResult<&str, bool> {
    // !BIOSYM archive 2
    let (s, _) = tuple((tag("!BIOSYM archive "), one_of("23"), eol))(s)?;
    // PBC=ON
    let (s, (_, pbc, _)) = tuple((tag("PBC="), alt((tag("ON"), tag("OFF"))), eol))(s)?;
    //                Energy         8          0.0000       -302.460678
    let (s, _) = read_line(s)?;
    // !DATE
    let (s, _) = read_line(s)?;

    Ok((s, pbc == "ON"))
}

#[test]
fn test_parse_car_head() -> Result<()> {
    let line = "!BIOSYM archive 2
PBC=ON
                      Energy         8          0.0000       -302.460678
!DATE
PBC    8.86010000   21.50090000   20.05100000   90.00000000   90.00000000   93.93820000
H        5.640710263    9.141640724   11.792332013 CORE    1 H  H    0.0000    1
H        4.298792550    8.077310644   11.223689193 CORE    2 H  H    0.0000    2
";
    let (_, pbc) = read_head_lines(line)?;
    assert!(pbc);

    Ok(())
}
// 68eb9728 ends here

// [[file:../../gchemol-readwrite.note::26310c9a][26310c9a]]
fn read_pbc_line(s: &str) -> IResult<&str, [f64; 6]> {
    let (s, (_, _, [a, b, c])) = tuple((tag("PBC"), space1, xyz_array))(s)?;
    let (s, (_, [alpha, beta, gamma])) = tuple((space1, xyz_array))(s)?;
    let (s, r) = read_until_eol(s)?;

    if !r.trim().is_empty() && !r.contains("P1") {
        warn!("symmetry other than P1 is not handled:\n{r}");
    }
    Ok((s, [a, b, c, alpha, beta, gamma]))
}

#[test]
fn test_car_pbc() {
    let line = "PBC    8.86010000   21.50090000   20.05100000   90.00000000   90.00000000   93.93820000\n";
    let (_, params) = read_pbc_line(line).unwrap();
    assert_eq!(params[0], 8.86010);
    assert_eq!(params[5], 93.9382);
    let line = "PBC    5.0256    5.0256   13.6943   90.0000   90.0000  120.0000 (P1)\n";
    let (_, params) = read_pbc_line(line).unwrap();
    assert_eq!(params[0], 5.0256);
    assert_eq!(params[5], 120.0);
}
// 26310c9a ends here

// [[file:../../gchemol-readwrite.note::2fa00fc9][2fa00fc9]]
fn read_atom_line(s: &str) -> IResult<&str, Atom> {
    let (s, (name, _, coords, _)) = tuple((not_space, space1, xyz_array, read_until_eol))(s)?;
    // take only element symbol, ignoroing remaining part
    let symbol:String = name.chars().take_while(|x| x.is_ascii_alphabetic()).collect();
    let atom = Atom::new(symbol, coords);
    Ok((s, atom))
}
// 2fa00fc9 ends here

// [[file:../../gchemol-readwrite.note::6712e171][6712e171]]
fn parse_molecule(s: &str) -> IResult<&str, Molecule> {
    let (mut s, pbc) = read_head_lines(s)?;
    let lat = if pbc {
        let (r, [a, b, c, alpha, beta, gamma]) = read_pbc_line(s)?;
        s = r;
        Lattice::from_params(a, b, c, alpha, beta, gamma).into()
    } else {
        None
    };
    let (s, atoms) = many1(read_atom_line)(s)?;
    let mut mol = Molecule::from_atoms(atoms);
    mol.lattice = lat;

    Ok((s, mol))
}

#[test]
fn test_parse_car() -> Result<()> {
    let s = "!BIOSYM archive 2
PBC=ON
                      Energy         8          0.0000       -302.460678
!DATE
PBC    8.86010000   21.50090000   20.05100000   90.00000000   90.00000000   93.93820000
H        5.640710263    9.141640724   11.792332013 CORE    1 H  H    0.0000    1
H        4.298792550    8.077310644   11.223689193 CORE    2 H  H    0.0000    2
H        4.411039146    9.735539719   10.622981817 CORE    3 H  H    0.0000    3
H        1.948949336   10.464151540   11.012380457 CORE    4 H  H    0.0000    4
C        5.023072367    8.864354752   10.918038294 CORE    5 C  C    0.0000    5
O        5.868880616    8.419678268    9.893083809 CORE    6 O  O    0.0000    6
O        2.016197825    8.894412569    9.983383717 CORE    7 O  O    0.0000    7
O        2.362868195   10.279383280   10.139250843 CORE    8 O  O    0.0000    8
";
    let (_, mol) = parse_molecule(s)?;
    assert_eq!(mol.natoms(), 8);

    Ok(())
}
// 6712e171 ends here

// [[file:../../gchemol-readwrite.note::97511bc0][97511bc0]]
use super::ChemicalFile;
use super::ParseMolecule;

/// CAR Accelrys/MSI Biosym/Insight II format
#[derive(Clone, Copy, Debug)]
pub struct CarFile();

impl ChemicalFile for CarFile {
    fn ftype(&self) -> &str {
        "text/car"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".car", ".arc"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        bail!("not implemented yet")
    }
}

impl ParseMolecule for CarFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        let (_, mol) = parse_molecule(input).map_err(|e| anyhow!("parse car/arc format failure: {:?}", e))?;
        Ok(mol)
    }
}
// 97511bc0 ends here

// [[file:../../gchemol-readwrite.note::1f9aa2f3][1f9aa2f3]]
use super::*;

// read all available stream at once
impl super::parser::ReadPart for CarFile {}

impl CarFile {
    pub fn partitions<R: BufRead + Seek>(&self, mut r: TextReader<R>) -> Result<impl Iterator<Item = String>> {
        Ok(r.partitions(*self))
    }
}
// 1f9aa2f3 ends here
