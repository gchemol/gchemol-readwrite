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
fn test_formats_pdb() -> Result<()> {
    // multiple molecules generated by babel
    let fname = "tests/files/pdb/multi-babel.pdb";
    let mols = read_all(fname)?;

    assert_eq!(6, mols.len());
    assert_eq!(mols[5].natoms(), 13);

    // single molecule with a lattice
    let fname = "tests/files/pdb/sio2.pdb";
    let mols = read_all(fname).expect("parse pdb molecules");
    assert_eq!(1, mols.len());
    assert!(mols[0].lattice.is_some());

    // ASE generated pdb file
    let fname = "tests/files/pdb/ase.pdb";
    let mols = read_all(fname).expect("parse pdb molecules");
    assert_eq!(2, mols.len());
    assert_eq!(mols[0].natoms(), 16);
    assert_eq!(mols[1].natoms(), 10);

    // zeolite with connectivity
    let fname = "tests/files/pdb/LTL-zeolite-ms.pdb";
    let mols = read_all(fname).expect("parse pdb molecules");
    assert_eq!(1, mols.len());

    let fname = "tests/files/pdb/tyr-33-conf1.pdb";
    let mols = read_all(fname).expect("parse pdb tyr33");
    assert_eq!(1, mols.len());
    assert_eq!(mols[0].natoms(), 103);

    let fname = "tests/files/pdb/1PPC_ligand.pdb";
    let mols = read_all(fname).expect("parse pdb 1ppc");
    assert_eq!(1, mols.len());
    assert_eq!(mols[0].natoms(), 37);

    let filenames = vec![
        // GaussView generated pdb file
        "tests/files/pdb/gview.pdb",
        // Chem3D generated pdb file
        "tests/files/pdb/chem3d.pdb",
        // Discovery Studio generated pdb file
        "tests/files/pdb/ds.pdb",
        // Material Studio generated pdb file
        "tests/files/pdb/ms.pdb",
    ];
    for fname in filenames {
        let mols = read_all(fname).expect("parse pdb molecules");
        assert_eq!(1, mols.len());
        assert_eq!(mols[0].natoms(), 16, "{}", fname);
        assert_eq!(mols[0].nbonds(), 14, "{}", fname);
    }

    Ok(())
}
// test:1 ends here
