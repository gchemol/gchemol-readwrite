// [[file:../gchemol-readwrite.note::*imports][imports:1]]
use gchemol_core::Molecule;
use gchemol_readwrite::prelude::*;
use gchemol_readwrite::read_all;

use gut::prelude::*;
// imports:1 ends here

// [[file:../gchemol-readwrite.note::391edd88][391edd88]]
#[test]
fn test_readwrite() -> Result<()> {
    use tempfile::tempdir;

    let f = "./tests/files/mol2/LTL-crysin-ds.mol2";

    // read last molecule from file
    let mol = Molecule::from_file(f)?;

    // write molecule to file
    let dir = tempdir()?;
    let path = dir.path().join("test.mol2");
    mol.to_file(&path)?;

    // read all molecules into memory
    let mols = gchemol_readwrite::read_all(&path)?;
    assert_eq!(mols.len(), 1);

    // write all molecules into file
    gchemol_readwrite::write(&path, &mols)?;
    // force to write in xyz format
    gchemol_readwrite::write_format(&path, &mols, "text/xyz")?;
    let s = gut::fs::read_file(&path)?;
    assert_eq!(s.lines().count(), 99 + 3 + 2, "Failed to write in xyz format");

    // parse in specific format
    let s = gut::fs::read_file(&path)?;
    let b = std::io::Cursor::new(s.as_bytes());
    let mols = gchemol_readwrite::read_from(b, "text/xyz")?;
    assert_eq!(mols.count(), 1, "Failed to read in xyz format");
    let mol = Molecule::from_str(&s, "text/xyz")?;
    assert!(mol.is_periodic());

    // write in specific format
    let s = mol.format_as("text/xyz")?;
    assert!(!s.is_empty());

    // format molecule using user defined template
    let tpl = "./tests/files/templates/xyz.tera";
    let s = mol.render_with(tpl.as_ref())?;
    assert!(!s.is_empty());

    Ok(())
}
// 391edd88 ends here
