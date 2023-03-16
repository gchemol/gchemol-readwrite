// [[file:../gchemol-readwrite.note::f7a5df2a][f7a5df2a]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
// f7a5df2a ends here

// [[file:../gchemol-readwrite.note::0fdc2fea][0fdc2fea]]
#[test]
fn test_format_cjson() -> Result<()> {
    use vecfx::Vector3f;

    let f = "tests/files/cjson/ethane.cjson";
    let mol = Molecule::from_file(f)?;
    assert_eq!(mol.natoms(), 8);

    let f = "tests/files/cjson/pbc.cjson";
    let mol = Molecule::from_file(f)?;
    assert_eq!(mol.natoms(), 6);
    assert!(mol.is_periodic());
    let frac: Vector3f = mol.get_scaled_positions().unwrap().last().unwrap().into();
    let expected: Vector3f = [0.5, 0.80530, 0.19470].into();
    vecfx::approx::assert_relative_eq!(frac, expected, epsilon = 1E-4);

    Ok(())
}
// 0fdc2fea ends here
