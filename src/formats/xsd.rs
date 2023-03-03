// [[file:../../gchemol-readwrite.note::68a04e1c][68a04e1c]]
use gut::prelude::*;

use gchemol_core::{Atom, Lattice, Molecule};
use roxmltree::Node;
use std::collections::HashMap;
// 68a04e1c ends here

// [[file:../../gchemol-readwrite.note::6a022d3f][6a022d3f]]
fn find_child_node<'a>(node: Node<'a, 'a>, tag_name: &'static str) -> Option<Node<'a, 'a>> {
    node.children().find(|node| node.tag_name().name() == tag_name)
}

fn filter_child_node<'a>(node: Node<'a, 'a>, tag_name: &'static str) -> impl Iterator<Item = Node<'a, 'a>> + 'a {
    node.children().filter(move |node| node.tag_name().name() == tag_name)
}

fn find_atom3d_node<'a>(node: Node<'a, 'a>) -> Option<Node<'a, 'a>> {
    find_child_node(node, "Atom3d")
}
// 6a022d3f ends here

// [[file:../../gchemol-readwrite.note::3e500125][3e500125]]
/// construct Lattice struct from xsd SpaceGroup node attributes
fn parse_lattice_from(node: Node) -> Option<Lattice> {
    use vecfx::*;

    let sg: HashMap<_, _> = node.attributes().map(|attr| (attr.name(), attr.value())).collect();
    let sgn = sg.get("Name")?;
    if *sgn != "P1" {
        error!("xsd: SpaceGroup other than P1 is not supported.");
        return None;
    }

    let avector = sg.get("AVector")?.split(',');
    let bvector = sg.get("BVector")?.split(',');
    let cvector = sg.get("CVector")?.split(',');
    let vector: Vec<f64> = avector.chain(bvector).chain(cvector).filter_map(|x| x.parse().ok()).collect();
    assert_eq!(vector.len(), 9);
    // FIXME: to be better
    Lattice::new(vector.as_3d().try_into().unwrap()).into()
}
// 3e500125 ends here

// [[file:../../gchemol-readwrite.note::*atom][atom:1]]
/// Construct `Atom` from xsd `Atom3d` node attributes
// FIXME: set atom sn from ID attribute
fn parse_atom_from(node: Node) -> Option<Atom> {
    let map: HashMap<_, _> = node.attributes().map(|attr| (attr.name(), attr.value())).collect();

    // get fractional position
    let xyz: Vec<f64> = map.get("XYZ")?.split(',').filter_map(|x| x.parse().ok()).collect();
    assert_eq!(xyz.len(), 3, "invalid atom node: {map:?}");
    // get element symbol
    let symbol = map.get("Components")?;
    let mut atom = Atom::new(*symbol, [xyz[0], xyz[1], xyz[2]]);
    // get atom name
    if let Some(name) = map.get("Name") {
        atom.set_label(*name);
    }
    // freeze atom?
    if map.get("RestrictedProperties").is_some() {
        atom.set_freezing([true; 3]);
    }
    Some(atom)
}
// atom:1 ends here

// [[file:../../gchemol-readwrite.note::b2502161][b2502161]]
use std::path::Path;

pub(self) fn parse_molecule_from_xsd_file(f: &Path) -> Result<Molecule> {
    let s = gut::fs::read_file(f)?;
    let mol = parse_molecule_from_xsd(&s)?;
    Ok(mol)
}

pub(self) fn parse_molecule_from_xsd(s: &str) -> Result<Molecule> {
    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };
    let doc = roxmltree::Document::parse_with_options(&s, opt)?;

    let mut nodes = doc.descendants().filter(|node| node.is_element());
    let atomistic_tree_root = nodes.find(|node| node.tag_name().name() == "AtomisticTreeRoot");
    ensure!(atomistic_tree_root.is_some(), "not a valid Material Studio xsd file");

    // periodic or molecule system?
    let mol = if let Some(root) = find_identity_mapping_node(atomistic_tree_root.unwrap()) {
        let space_group = find_child_node(root, "SpaceGroup").ok_or(anyhow!("malformed xsd: no SpaceGroup"))?;
        let lattice = parse_lattice_from(space_group).ok_or(anyhow!("malformed xsd: no lattice parsed"))?;
        let atoms: Vec<_> = filter_child_node(root, "Atom3d").filter_map(|node| parse_atom_from(node)).collect();
        let mut mol = Molecule::from_atoms(atoms);
        mol.set_lattice(lattice);
        // reset positions to Cartesian coords
        let positions: Vec<_> = mol.positions().collect();
        mol.set_scaled_positions(positions);
        mol
    } else {
        let mut root = find_molecule_node(atomistic_tree_root.unwrap()).unwrap();
        // Molecule/RepeatUnit/Atom3d
        let atom3d = if find_child_node(root, "Atom3d").is_none() {
            root = find_child_node(root, "RepeatUnit").ok_or(anyhow!("cannot find Atom3d node for mol"))?;
        };
        let atoms: Vec<_> = filter_child_node(root, "Atom3d").filter_map(|node| parse_atom_from(node)).collect();
        Molecule::from_atoms(atoms)
    };

    Ok(mol)
}

// SymmetrySystem/MappingSet/MappingFamily/IdentityMapping
fn find_identity_mapping_node<'a>(root: Node<'a, 'a>) -> Option<Node<'a, 'a>> {
    let child = find_child_node(root, "SymmetrySystem")?;
    let child = find_child_node(child, "MappingSet")?;
    let child = find_child_node(child, "MappingFamily")?;
    let node = find_child_node(child, "IdentityMapping")?;
    Some(node)
}

// root/Molecule or root
fn find_molecule_node<'a>(root: Node<'a, 'a>) -> Option<Node<'a, 'a>> {
    find_child_node(root, "Molecule").or(root.into())
}
// b2502161 ends here

// [[file:../../gchemol-readwrite.note::19028850][19028850]]
use super::ChemicalFile;
use super::ParseMolecule;

#[derive(Clone, Copy, Debug)]
pub struct XsdFile();

impl ChemicalFile for XsdFile {
    fn ftype(&self) -> &str {
        "xml/xsd"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".xsd"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        bail!("not implemented yet")
    }
}

impl ParseMolecule for XsdFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        let mol = parse_molecule_from_xsd(input).map_err(|e| format_err!("parse XSD format failure: {:?}", e))?;
        Ok(mol)
    }
}
// 19028850 ends here

// [[file:../../gchemol-readwrite.note::299b30ff][299b30ff]]
use super::*;

// read all available stream at once
impl super::parser::ReadPart for XsdFile {}

impl XsdFile {
    pub fn partitions<R: BufRead + Seek>(&self, mut r: TextReader<R>) -> Result<impl Iterator<Item = String>> {
        Ok(r.partitions(*self))
    }
}
// 299b30ff ends here

// [[file:../../gchemol-readwrite.note::b2151caa][b2151caa]]
#[test]
fn test_xsd_tree() -> Result<()> {
    use crate::prelude::*;

    use roxmltree::Document;

    let f = "/home/ybyygu/Workspace/ToDo/ASAP/20210708 滕波涛rxe验证/1/P.xsd";
    let mol = parse_molecule_from_xsd_file(f.as_ref())?;
    // mol.to_file("/tmp/a.gjf");

    Ok(())
}
// b2151caa ends here
