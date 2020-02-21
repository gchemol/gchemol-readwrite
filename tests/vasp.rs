// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
// imports:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*test][test:1]]
#[test]
fn test_format_vasp_input() -> Result<()> {
    let f = "tests/files/vasp/POSCAR";
    let mols = read_all(f)?;
    assert_eq!(1, mols.len());
    assert_eq!(365, mols[0].natoms());

    Ok(())
}
// test:1 ends here
