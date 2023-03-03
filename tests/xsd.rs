// [[file:../gchemol-readwrite.note::f7a5df2a][f7a5df2a]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
// f7a5df2a ends here

// [[file:../gchemol-readwrite.note::0fdc2fea][0fdc2fea]]
#[test]
fn test_format_xsd() -> Result<()> {
    let f = "tests/files/xsd/mol.xsd";
    let mol = Molecule::from_file(f)?;
    assert_eq!(mol.natoms(), 5);

    // let f = "tests/files/xsd/bisphenol.xsd";
    // let mol = Molecule::from_file(f)?;
    // assert_eq!(mol.natoms(), 18);

    let f = "tests/files/xsd/pbc.xsd";
    let mol = Molecule::from_file(f)?;
    assert_eq!(mol.natoms(), 127);
    assert_eq!(mol.atoms().filter(|(_, a)| a.is_fixed()).count(), 42);

    Ok(())
}
// 0fdc2fea ends here
