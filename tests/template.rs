// [[file:../gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
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
