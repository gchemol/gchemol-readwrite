// [[file:../gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
// imports:1 ends here

// [[file:../gchemol-readwrite.note::d13f476e][d13f476e]]
#[test]
fn test_format_cml() -> Result<()> {
    let f = "tests/files/cml/1LJL_Cys10.cml";
    let mol = Molecule::from_file(f)?;
    assert_eq!(mol.natoms(), 13);

    let f = "tests/files/cml/Fe.cml";
    let mol = Molecule::from_file(f)?;
    assert!(mol.is_periodic());

    Ok(())
}
// d13f476e ends here
