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
fn test_template_render() -> Result<()> {
    let f = "./tests/files/mol2/LTL-crysin-ds.mol2";
    let mol = Molecule::from_file(f)?;

    let tpl = "./tests/files/templates/xyz.hbs";
    let s = mol.render_with(tpl.as_ref())?;

    let tpl = "./tests/files/templates/xyz.tera";
    let s = mol.render_with(tpl.as_ref())?;
    let m = Molecule::from_str(&s, "text/xyz")?;
    assert_eq!(mol.natoms(), m.natoms());

    Ok(())
}
// test:1 ends here
