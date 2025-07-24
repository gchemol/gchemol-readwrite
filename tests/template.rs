// [[file:../gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
// use std::fs;
// imports:1 ends here

// [[file:../gchemol-readwrite.note::fe64490a][fe64490a]]
#[test]
fn test_template_render() -> Result<()> {
    let f = "./tests/files/mol2/LTL-crysin-ds.mol2";
    let mol = Molecule::from_file(f)?;
    assert!(mol.is_periodic());

    let tpl = "./tests/files/templates/xyz.hbs";
    let s = mol.render_with(tpl.as_ref())?;
    let m = Molecule::from_str(&s, "text/xyz")?;
    assert_eq!(mol.natoms(), m.natoms());
    assert!(!m.is_periodic());

    let tpl = "./tests/files/templates/xyz.tera";
    let s = mol.render_with(tpl.as_ref())?;
    let m = Molecule::from_str(&s, "text/xyz")?;
    assert_eq!(mol.natoms(), m.natoms());
    assert!(m.is_periodic());

    Ok(())
}
// fe64490a ends here

// [[file:../gchemol-readwrite.note::*abacus_stru][abacus_stru:1]]
#[test]
fn test_template_render_abacus_stru() -> Result<()> {
    let f = "./tests/files/mol2/LTL-crysin-ds.mol2";
    let mol = Molecule::from_file(f)?;
    assert!(mol.is_periodic());

    let tpl = "./tests/files/templates/abacus_stru.jinja";
    let s = mol.render_with(tpl.as_ref())?;

    // Save the rendered template to a file for verification
    // std::fs::write("abacus_stru_test.stru", &s)?;

    // Print the rendered template for verification
    println!("{}", s);

    // Basic checks to ensure the template rendered correctly
    assert!(s.contains("ATOMIC_SPECIES"));
    assert!(s.contains("LATTICE_CONSTANT"));
    assert!(s.contains("ATOMIC_POSITIONS"));
    assert!(s.contains("Si"));  // LTL-crysin-ds.mol2 contains silicon atoms

    Ok(())
}
// abacus_stru:1 ends here
