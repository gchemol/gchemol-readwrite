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

    // NOTE: x3 for bare molecule, xFract for periodic molecule
    let x = map.get("x3").or(map.get("xFract"))?.parse().ok()?;
    let y = map.get("y3").or(map.get("yFract"))?.parse().ok()?;
    let z = map.get("z3").or(map.get("zFract"))?.parse().ok()?;

    // get element symbol
    let symbol = map.get("elementType")?;
    let mut atom = Atom::new(*symbol, [x, y, z]);
    Some(atom)
}
// caf74f52 ends here

// [[file:../../gchemol-readwrite.note::d7bee6e3][d7bee6e3]]
fn parse_lattice_from(lattice: Node) -> Lattice {
    use std::collections::HashMap;

    let params: HashMap<_, f64> = lattice
        .descendants()
        .filter_map(|n| {
            if n.has_tag_name("scalar") && n.has_attribute("title") {
                if let Some(value) = n.text() {
                    let title = n.attribute("title")?;
                    let value = value.parse().ok()?;
                    return Some((title, value));
                }
            }
            None
        })
        .collect();
    Lattice::from_params(params["a"], params["b"], params["c"], params["alpha"], params["beta"], params["gamma"])
}
// d7bee6e3 ends here

// [[file:../../gchemol-readwrite.note::63fcee19][63fcee19]]
fn parse_molecule_from(molecule: Node) -> Molecule {
    // parse atoms
    let atoms = molecule.descendants().filter(|n| n.has_tag_name("atom"));
    let atoms = atoms.filter_map(|node| parse_atom_from(node));

    // molecular title
    let title = molecule.attribute("id").unwrap_or("untitled cml");
    let mut mol = Molecule::from_atoms(atoms);
    mol.set_title(title);

    // parse lattice object
    if let Some(node) = molecule.children().find(|n| n.has_tag_name("crystal")) {
        let lattice = parse_lattice_from(node);
        mol.set_lattice(lattice);
    }

    // parse bonds
    if let Some(node) = molecule.children().find(|n| n.has_tag_name("bondArray")) {
        // TODO
    }

    mol
}

pub(self) fn parse_molecules(s: &str) -> Result<Vec<Molecule>> {
    use roxmltree::Document;

    let doc = Document::parse(s)?;
    let nodes_mol = doc.root_element().descendants().filter(|n| n.has_tag_name("molecule"));
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

    // write lattice
    if let Some(lat) = mol.get_lattice() {
        writeln!(s, "  <crystal>");
        let [a, b, c] = lat.lengths();
        writeln!(s, "<scalar title='a' units='units:angstrom'>{a}</scalar>");
        writeln!(s, "<scalar title='b' units='units:angstrom'>{b}</scalar>");
        writeln!(s, "<scalar title='c' units='units:angstrom'>{c}</scalar>");
        let [alpha, beta, gamma] = lat.angles();
        writeln!(s, "<scalar title='alpha' units='units:degree'>{alpha}</scalar>");
        writeln!(s, "<scalar title='beta' units='units:degree'>{beta}</scalar>");
        writeln!(s, "<scalar title='gamma' units='units:degree'>{gamma}</scalar>");
        writeln!(s, "  </crystal>");
    }

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

// [[file:../../gchemol-readwrite.note::c4bf05cf][c4bf05cf]]
#[test]
fn test_parse_mol_from_cml() -> Result<()> {
    let f = "tests/files/cml/1LJL_Cys10.cml";
    let s = gut::fs::read_file(f)?;
    let mols = parse_molecules(&s)?;
    let s = format_molecules(&mols);
    // println!("{}", s);
    let mols = parse_molecules(&s)?;
    assert_eq!(mols.len(), 7);
    let natoms_list = vec![1, 3, 7, 3, 207, 33, 13];
    for i in 0..7 {
        assert_eq!(mols[i].natoms(), natoms_list[i]);
    }

    let f = "tests/files/cml/Fe.cml";
    let s = gut::fs::read_file(f)?;
    let mols = parse_molecules(&s)?;
    assert_eq!(mols.len(), 1);
    assert!(mols[0].is_periodic());

    Ok(())
}
// c4bf05cf ends here
