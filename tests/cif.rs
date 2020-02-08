// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gchemol_gut::prelude::*;
// imports:1 ends here

// tests

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*tests][tests:1]]
#[test]
fn test_formats_cif() {
    let f = "tests/files/cif/babel.cif";
    let mols = read_all(f).expect("babel.cif");
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 34);

    let f = "tests/files/cif/MS-MOR.cif";
    let mols = read_all(f).expect("MS-MOR.cif");
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 144);

    let f = "tests/files/cif/IZA-LTL.cif";
    let mols = read_all(f).expect("cif IZA");
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 8);

    let f = "tests/files/cif/ccdc.cif";
    let mols = read_all(f).expect("cif ccdc");
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 41);

    let f = "tests/files/cif/quinone.cif";
    let mols = read_all(f).expect("cif quinone");
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 16);

    let f = "tests/files/cif/multi.cif";
    let mols = read_all(f).expect("cif multi Rh");
    assert_eq!(mols.len(), 10);
    assert_eq!(mols[0].natoms(), 32);
}
// tests:1 ends here
