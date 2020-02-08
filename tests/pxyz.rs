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
fn test_formats_plain_xyz() -> Result<()> {
    let f = "tests/files/xyz/c2h4.pxyz";
    let mols = read_all(f)?;
    assert_eq!(1, mols.len());
    assert_eq!(6, mols[0].natoms());

    // element numbers
    let f = "tests/files/xyz/ele-num.pxyz";
    let mols = read_all(f)?;
    assert_eq!(1, mols.len());
    assert_eq!(17, mols[0].natoms());
    let symbols: Vec<_> = mols[0].symbols().collect();
    assert_eq!("Si", symbols[0]);

    // parse multiple molecules
    let f = "tests/files/xyz/multi.pxyz";
    let mols = read_all(f)?;
    assert_eq!(6, mols.len());
    let natoms_expected = vec![16, 10, 16, 16, 16, 13];
    let natoms: Vec<_> = mols.iter().map(|m| m.natoms()).collect();
    assert_eq!(natoms_expected, natoms);

    // pbc
    let f = "tests/files/xyz/pbc.pxyz";
    let mols = read_all(f)?;
    assert_eq!(1, mols.len());
    assert_eq!(32, mols[0].natoms());
    assert!(mols[0].lattice.is_some());

    Ok(())
}
// tests:1 ends here
