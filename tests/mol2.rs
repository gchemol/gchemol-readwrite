// [[file:../gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
// imports:1 ends here

// [[file:../gchemol-readwrite.note::b028087c][b028087c]]
#[test]
fn test_formats_mol2() {
    let f = "tests/files/mol2/ch3f-dos.mol2";
    let mols = read_all(f).expect("ch3f");
    assert_eq!(1, mols.len());

    // if missing the final blank line: gaussview generated .mol2 file
    let f = "tests/files/mol2/alanine-gv.mol2";
    let mols = read_all(f).expect("alanine");
    assert_eq!(1, mols.len());
    let mol = &mols[0];
    assert_eq!(12, mol.natoms());
    assert_eq!(11, mol.nbonds());

    // serde_json: key must be a string
    let m = &mols[0];
    assert!(m.to_json().is_ok());

    // molecule trajectory: openbabel converted .mol2 file
    let f = "tests/files/mol2/multi-obabel.mol2";
    let mols = read_all(f).expect("multi-obabel");
    let natoms_expected = vec![16, 10, 16, 16, 16, 13];
    let natoms: Vec<_> = mols.iter().map(|m| m.natoms()).collect();
    assert_eq!(natoms_expected, natoms);

    let nbonds_expected = vec![14, 10, 14, 14, 14, 12];
    let nbonds: Vec<_> = mols.iter().map(|m| m.nbonds()).collect();
    assert_eq!(nbonds_expected, nbonds);
    assert_eq!(6, mols.len());

    // single molecule with a lattice
    // discovery studio generated .mol2 file
    let f = "tests/files/mol2/LTL-crysin-ds.mol2";
    let mols = read_all(f).expect("LTL");
    assert_eq!(1, mols.len());
    assert!(mols[0].lattice.is_some());
}
// b028087c ends here
