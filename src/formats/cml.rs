// [[file:../../gchemol-readwrite.note::1dbbc8d8][1dbbc8d8]]
//! Chemical Markup Language
use gut::prelude::*;

use gchemol_core::{Atom, Lattice, Molecule};
use roxmltree::Node;
use std::collections::HashMap;
// 1dbbc8d8 ends here

// [[file:../../gchemol-readwrite.note::caf74f52][caf74f52]]
/// Construct `Atom` from child node of cml `atomArray`
// <atom id='a21' elementType='O' x3='9.37036730304' y3='1.29903952131' z3='0.237767735478'  />
fn parse_atom_from(node: Node) -> Option<Atom> {
    let map: HashMap<_, _> = node.attributes().map(|attr| (attr.name(), attr.value())).collect();

    let x = map.get("x3")?.parse().ok()?;
    let y = map.get("y3")?.parse().ok()?;
    let z = map.get("z3")?.parse().ok()?;

    // get element symbol
    let symbol = map.get("elementType")?;
    let mut atom = Atom::new(*symbol, [x, y, z]);
    Some(atom)
}
// caf74f52 ends here

// [[file:../../gchemol-readwrite.note::63fcee19][63fcee19]]
fn parse_molecule_from(molecule: Node) -> Molecule {
    let atoms = molecule.descendants().filter(|n| n.has_tag_name("atom"));
    // parse atoms
    let atoms = atoms.filter_map(|node| parse_atom_from(node));

    // molecular title
    let title = molecule.attribute("id").unwrap_or("untitled cml");
    let mut mol = Molecule::from_atoms(atoms);
    mol.set_title(title);

    // TODO: parse bonds
    // let bonds = molecule.descendants().filter(|n| n.has_tag_name("bonds"));
    mol
}

pub(self) fn parse_molecules(s: &str) -> Result<Vec<Molecule>> {
    use roxmltree::Document;

    let doc = Document::parse(s)?;
    // validation
    let node = doc.root_element().first_element_child();
    ensure!(node.is_some() && node.unwrap().tag_name().name() == "molecule", "invalid cml format");

    let nodes_mol = doc.root_element().children().filter(|n| n.has_tag_name("molecule"));
    let mols = nodes_mol.map(|node| parse_molecule_from(node)).collect();
    Ok(mols)
}

fn write_molecule(s: &mut String, mol: &Molecule) {
    let title = mol.title();
    writeln!(s, " <molecule id='{title}'>");
    // write atoms
    writeln!(s, "  <atomArray>");
    for (i, a) in mol.atoms() {
        let sym = a.symbol();
        let [x, y, z] = a.position();
        writeln!(s, "   <atom id='a{i}' elementType='sym' x3='{x}' y3='{y}' z3='{z}' />");
    }
    writeln!(s, "  </atomArray>");
    // write bonds
    writeln!(s, "  <bondArray>");
    for (u, v, _) in mol.bonds() {
        writeln!(s, "   <bond atomRefs2='a{u} a{v} />\n");
    }
    writeln!(s, "  </bondArray>");

    writeln!(s, "</molecule>");
}

/// Format a list of molecules in CML format.
pub(self) fn format_molecules<'a>(mols: impl IntoIterator<Item = &'a Molecule>) -> String {
    let mut s = String::new();
    writeln!(&mut s, "<?xml version='1.0'?>");
    writeln!(&mut s, "<list xmlns='http://www.xml-cml.org/schema'>");
    for mol in mols.into_iter() {
        write_molecule(&mut s, mol);
    }
    writeln!(&mut s, "</list>");
    s
}

#[test]
fn test_parse_mol_from_cml() -> Result<()> {
    let f = "tests/files/cml/1LJL_Cys10.cml";
    let s = gut::fs::read_file(f)?;
    let mols = parse_molecules(&s)?;
    let s = format_molecules(&mols);
    println!("{}", s);
    let mols = parse_molecules(&s)?;
    assert_eq!(mols.len(), 7);
    let natoms_list = vec![1, 3, 7, 3, 207, 33, 13];
    for i in 0..7 {
        assert_eq!(mols[i].natoms(), natoms_list[i]);
    }

    Ok(())
}
// 63fcee19 ends here

// [[file:../../gchemol-readwrite.note::b22e0379][b22e0379]]
use super::ChemicalFile;
use super::ParseMolecule;

#[derive(Clone, Copy, Debug)]
/// Basic support for the Chemical Markup Language (read-only)
pub struct CmlFile();

impl ChemicalFile for CmlFile {
    fn ftype(&self) -> &str {
        "xml/cml"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".cml"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        ensure!(!mol.is_periodic(), "cannot render Lattice in cml format!");
        Ok(format_molecules([mol]))
    }
}

impl ParseMolecule for CmlFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        // FIXME: dirty
        let mut mols = parse_molecules(input)?;
        ensure!(!mols.is_empty(), "parse cml failed");
        let i = mols.len() - 1;
        Ok(mols.remove(i))
    }
}
// b22e0379 ends here

// [[file:../../gchemol-readwrite.note::de0729da][de0729da]]
crate::cf_impl_partitions!(CmlFile);
// de0729da ends here
