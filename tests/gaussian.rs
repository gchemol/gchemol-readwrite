// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gchemol_gut::prelude::*;
// imports:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*test][test:1]]
#[test]
fn test_format_gaussian_input() -> Result<()> {
    // simple xyz
    let f = "./tests/files/gaussian/test1044.com";
    let mols = read_all(f)?;
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 19);

    // comma separated
    let f = "./tests/files/gaussian/test1036.com";
    let mols = read_all(f)?;
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 26);

    // ONIOM style
    let f = "./tests/files/gaussian/test0769.com";
    let mols = read_all(f)?;
    assert_eq!(mols.len(), 1);
    assert_eq!(mols[0].natoms(), 38);

    // multiple jobs
    let f = "./tests/files/gaussian/multi.gjf";
    let mols = read_all(f)?;
    assert_eq!(mols.len(), 6);

    Ok(())
}
// test:1 ends here
