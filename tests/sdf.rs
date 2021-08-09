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
fn test_formats_sdf() -> Result<()> {
    let f = "tests/files/sdf/dendrogram.sd";
    let mols = read_all(f)?;
    assert_eq!(mols.len(), 2);
    assert_eq!(mols[0].natoms(), 30);
    assert_eq!(mols[0].nbonds(), 31);

    let f = "tests/files/sdf/multi-babel.mol";
    let mols = read_all(f)?;
    assert_eq!(mols.len(), 2);
    assert_eq!(mols[0].natoms(), 30);
    assert_eq!(mols[0].nbonds(), 31);

    let f = "tests/files/sdf/thiadiazolyl.mol";
    let mols = read_all(f)?;
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 7);
    assert_eq!(mols[0].nbonds(), 7);

    Ok(())
}
// test:1 ends here
