// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
// imports:1 ends here

// tests

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*tests][tests:1]]
#[test]
fn test_formats_xyz() -> Result<()> {
    // FIXME: remove
    gut::cli::setup_logger();

    // read all molecules into a Vec
    let f = "tests/files/xyz/c2h4.xyz";
    let mols = read_all(f)?;
    assert_eq!(1, mols.len());
    assert_eq!(6, mols[0].natoms());

    // parse multiple molecules
    let f = "tests/files/xyz/multi.xyz";
    // read in an iterator over parsed molecules
    let mols = gchemol_readwrite::read(f)?;
    assert_eq!(6, mols.count());
    let mols = read_all(f)?;
    assert_eq!(6, mols.len());

    let natoms_expected = vec![16, 10, 16, 16, 16, 13];
    let natoms: Vec<_> = mols.iter().map(|m| m.natoms()).collect();
    assert_eq!(natoms_expected, natoms);

    // pbc
    let f = "tests/files/xyz/pbc.xyz";
    let mol = Molecule::from_file(f)?;
    assert_eq!(32, mol.natoms());
    assert!(mol.lattice.is_some());

    // format_as <=> parse_from
    let s = mol.format_as("text/xyz")?;
    let mol2 = Molecule::from_str(&s, "text/xyz")?;
    assert_eq!(mol.natoms(), mol2.natoms());

    // trajectory
    let f = "./tests/files/xyz/rx-lst5.xyz";
    let mols = gchemol_readwrite::read_all(f)?;
    assert_eq!(mols.len(), 5);
    assert_eq!(mols[0].natoms(), 13);

    Ok(())
}
// tests:1 ends here
