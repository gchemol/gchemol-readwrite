// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use super::parser::*;
// imports:1 ends here

// header

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*header][header:1]]
// Gaussian input file
//
// Reference
// ---------
// http://gaussian.com/input/?tabid=0
// header:1 ends here

// link0 section
// Reference
// - https://gaussian.com/link0/

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*link0 section][link0 section:1]]
fn link0_cmd(s: &str) -> IResult<&str, &str> {
    let prefix = tag("%");
    do_parse!(s, prefix >> cmd: read_until_eol >> (cmd))
}

#[test]
fn test_link0_cmd() {
    let (_, cmd) = link0_cmd("%Mem=64MB\n").unwrap();
    assert_eq!("Mem=64MB", cmd);

    let (_, cmd) = link0_cmd("%save\n").unwrap();
    assert_eq!("save", cmd);
}

fn link0_section(s: &str) -> IResult<&str, Vec<&str>> {
    many0(link0_cmd)(s)
}

#[test]
fn test_link0_section() {
    let lines = "%chk=C5H12.chk
%nproc=8
%mem=5GB
#p opt freq=noraman nosymm B3LYP/6-31+G** test geom=connect
";

    let (_, link0_cmds) = link0_section(lines).expect("gjf link0 section");
    assert_eq!(3, link0_cmds.len());
}
// link0 section:1 ends here

// route section
// The route section of a Gaussian job is initiated by a pound sign (#) as the
// first non-blank character of a line. The remainder of the section is in
// free-field format. For most jobs, all of the information can be placed on this
// first line, but overflow to other lines (which may but need not begin with a #
// symbol) is permissible. The route section must be terminated by a blank line.

// References
// - https://gaussian.com/route/


// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*route section][route section:1]]
fn blank_line(s: &str) -> IResult<&str, ()> {
    do_parse!(s, space0 >> eol >> (()))
}

fn route_section(s: &str) -> IResult<&str, String> {
    let pound = tag("#");
    let print_level = opt(alt((tag_no_case("N"), tag_no_case("P"), tag_no_case("T"))));
    let keywords = many_till(read_until_eol, blank_line);
    do_parse!(
        s,
        pound >> print_level >>      // #P, #N, #T, #
        space0 >> lines: keywords >> // opt B3LYP/6-31G* test
        ({
            lines.0.join(" ")
        })
    )
}

#[test]
fn test_route_section() {
    let lines = "#opt freq=noraman nosymm B3LYP/6-31+G** test geom=connect

";
    let x = route_section(lines).expect("gjf route section");

    let lines = "#p opt freq=noraman nosymm
B3LYP/6-31+G** test geom=connect

";
    let x = route_section(lines).expect("gjf route section multi-lines");
}
// route section:1 ends here

// title section
// Title section: Brief description of the calculation (blank line-terminated).


// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*title section][title section:1]]
fn title_section(s: &str) -> IResult<&str, String> {
    let title_lines = many_till(read_until_eol, blank_line);
    do_parse!(
        s,
        lines: title_lines >> // xx
        ({
            lines.0.join(" ")
        })
    )
}
// title section:1 ends here

// charge and spin multiplicity

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*charge and spin multiplicity][charge and spin multiplicity:1]]
// Specifies the net electric charge (a signed integer) and the spin
// multiplicity (usually a positive integer)
fn read_charge_and_spin_list(s: &str) -> IResult<&str, Vec<isize>> {
    let values = separated_nonempty_list(space1, signed_digit);
    do_parse!(s, space0 >> v: values >> eol >> (v))
}

#[test]
fn test_charge_and_spin() {
    let (_, x) = read_charge_and_spin_list("0	1\n").expect("gjf charge & spin");
    assert_eq!(2, x.len());

    let line = " 0 1 \t 0  1 -3 2 \n";
    let (r, x) = read_charge_and_spin_list(line).expect("gjf charge & spin");
    assert_eq!(6, x.len());
}
// charge and spin multiplicity:1 ends here

// element info

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*element info][element info:1]]
// single atom parameter entry
// Fragment=1, or Iso=13, or Spin=3
fn atom_param(s: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(alphanumeric1, tag("="), alphanumeric1)(s)
}

#[test]
fn test_gjf_atom_param() {
    let (_, (param, value)) = atom_param("fragment=1 ").unwrap();
    assert_eq!("fragment", param);
    assert_eq!("1", value);
}

// multiple property entries
// (fragment=1,iso=13,spin=3)
fn atom_params(s: &str) -> IResult<&str, Vec<(&str, &str)>> {
    let params = separated_nonempty_list(tag(","), atom_param);
    delimited(tag("("), params, tag(")"))(s)
}

#[test]
fn test_gjf_atom_params() {
    let (_, d) = atom_params("(fragment=1,iso=13,spin=3) ").expect("gjf atom properties");
    assert_eq!(3, d.len())
}

// MM parameters, such as atom type and partial charge
// -CA--0.25
fn atom_mm_info(s: &str) -> IResult<&str, (&str, Option<f64>)> {
    let mm_type = preceded(tag("-"), is_not("- \t"));
    let mm_charge = preceded(tag("-"), double);
    pair(mm_type, opt(mm_charge))(s)
}

#[test]
fn test_gjf_atom_mm_info() {
    let (_, (mm_type, mm_charge)) = atom_mm_info("-CA--0.25").unwrap();
    assert_eq!("CA", mm_type);
    assert_eq!(Some(-0.25), mm_charge);

    let (_, (mm_type, mm_charge)) = atom_mm_info("-CA").unwrap();
    assert_eq!("CA", mm_type);
    assert_eq!(None, mm_charge);
}
// element info:1 ends here

// oniom info
// atom [freeze-code] coordinate-spec layer [link-atom [bonded-to [scale-fac1 [scale-fac2 [scale-fac3]]]]]

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*oniom info][oniom info:1]]
// C-CA--0.25   0   -4.703834   -1.841116   -0.779093 L
// C-CA--0.25   0   -3.331033   -1.841116   -0.779093 L H-HA-0.1  3
fn atom_oniom_params(s: &str) -> IResult<&str, &str> {
    do_parse!(
        s,
        space1 >>
        layer: alpha1 >> // ONIOM layer: H, M, L
        (layer)
    )
}

#[test]
fn test_gjf_atom_oniom_params() {
    let (_, layer) = atom_oniom_params(" L H-Ha-0.1 3\n").unwrap();
    assert_eq!("L", layer);
    let (_, layer) = atom_oniom_params(" L\n").unwrap();
    assert_eq!("L", layer);
}
// oniom info:1 ends here

// atom line

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*atom line][atom line:1]]
use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Debug)]
struct GaussianAtom<'a> {
    element_label: &'a str,
    mm_type: Option<&'a str>,
    mm_charge: Option<f64>,
    frozen_code: Option<isize>,
    position: [f64; 3],
    properties: HashMap<&'a str, &'a str>,
    oniom_layer: Option<&'a str>,
}

impl<'a> Default for GaussianAtom<'a> {
    fn default() -> Self {
        GaussianAtom {
            element_label: "C",
            mm_type: None,
            mm_charge: None,
            frozen_code: None,
            properties: HashMap::new(),
            position: [0.0; 3],
            oniom_layer: None,
        }
    }
}

fn frozen_code(s: &str) -> IResult<&str, isize> {
    terminated(signed_digit, space1)(s)
}

// How about this: C-CA--0.25(fragment=1,iso=13,spin=3) 0 0.0 1.2 3.4 H H-H_
fn atom_line(s: &str) -> IResult<&str, GaussianAtom> {
    let mm_info = opt(atom_mm_info);
    let params = opt(atom_params);
    let oniom_info = opt(atom_oniom_params);
    let frozen = opt(frozen_code);
    do_parse!(
        s,
        space0 >> e: alphanumeric1      >> // element label
        m: mm_info                      >> // MM type and MM partial charge
        p: params                       >> // Optional atom info, such as fragment, nuclei props
        space1                          >> // xx
        f: frozen                       >> // frozen code: 0, -1
        c: xyz_array                    >> // x, y, z coords
        o: oniom_info                   >> // H H-H_
        read_line >>                       // ignore remaining part
        ({
            GaussianAtom {
                element_label: e,
                mm_type: m.and_then(|x| Some(x.0)),
                mm_charge: m.and_then(|x| x.1),
                frozen_code: f,
                position: c,
                properties: if p.is_some() {p.unwrap().into_iter().collect()} else {HashMap::new()},
                oniom_layer: o,
                ..Default::default()
            }
        })
    )
}
// atom line:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*test][test:1]]
#[test]
fn test_gjf_atom_line() {
    let line = "C-CA--0.25(fragment=1,iso=13,spin=3)  -1 0.00   0.00   0.00 L H-HA-0.1  3\n";
    let (_, ga) = atom_line(line).expect("full oniom atom line");
    assert_eq!(ga.oniom_layer, Some("L"));

    let line = "C-CA--0.25(fragment=1,iso=13,spin=3) -1 0.00   0.00   0.00 L \n";
    let (_, ga) = atom_line(line).expect("oniom without link atom");
    assert_eq!(ga.oniom_layer, Some("L"));
    assert_eq!(ga.frozen_code, Some(-1));

    let line = "C-CA--0.25(fragment=1,iso=13,spin=3) -1 0.00   0.00   0.00\n";
    let (_, ga) = atom_line(line).expect("frozen + xyz");
    assert_eq!(ga.frozen_code, Some(-1));

    let line = "C-CA--0.25(fragment=1,iso=13,spin=3) 0.00   0.00   0.00\n";
    let (_, ga) = atom_line(line).expect("missing frozen code");
    assert_eq!(ga.mm_charge, Some(-0.25));
    assert_eq!(ga.mm_type, Some("CA"));
    assert_eq!(ga.properties["fragment"], "1");

    let (_, ga) = atom_line("C-CA--0.25  0.00   0.00   0.00\n").expect("no key-value params");
    assert_eq!(ga.mm_charge, Some(-0.25));
    assert_eq!(ga.mm_type, Some("CA"));

    let (_, ga) = atom_line("C-C_3 0.00 0.00 0.00\n").expect("no mm charge");
    assert_eq!(ga.mm_type, Some("C_3"));
    assert_eq!(ga.mm_charge, None);

    let line = " C12(fragment=1)  0.00   0.00   0.00\n";
    let (_, ga) = atom_line(line).expect("key-value params only");
    assert_eq!(ga.properties["fragment"], "1");
    assert_eq!(ga.position[0], 0.0);

    let line = " C12  0.00   0.00   0.00\n";
    let (_, ga) = atom_line(line).expect("simple");
    assert_eq!(ga.position[0], 0.0);
}
// test:1 ends here

// connectivity specs

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*connectivity specs][connectivity specs:1]]
// Connectivity specs example:
//
// 1 2 1.0 3 1.0 4 1.0 5 1.0
// 2
// 3
fn bond_item(s: &str) -> IResult<&str, (usize, f64)> {
    do_parse!(
        s,
        space1 >> n: unsigned_digit >> // bond atom number: 2
        space1 >> o: double         >> // bond order 1.0
        (n, o)
    )
}

fn read_connect_line(s: &str) -> IResult<&str, Vec<(usize, usize, Bond)>> {
    let read_bond_items = many0(bond_item);
    do_parse!(
        s,
        space0 >> n: unsigned_digit    >> // host atom number
        others: read_bond_items >> eol >> // bond atoms and bond orders
        ({
            let mut bonds = vec![];
            for (index2, order) in others {
                bonds.push((n, index2, Bond::single()));
            }

            bonds
        })
    )
}

#[test]
fn test_gjf_connectivity() {
    let (_, x) = read_connect_line(" 1 2 1.0 3 1.0 4 1.0 5 1.0\n").expect("gjf connectivity");
    assert_eq!(4, x.len());
}
// connectivity specs:1 ends here

// parse

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*parse][parse:1]]
fn parse_molecule_meta(s: &str) -> IResult<&str, String> {
    let link0 = opt(link0_section);
    do_parse!(
        s,
        link0 >>             // Link0 commands, which is optional section
        route_section >>     // route section contains job keywords
        t: title_section >>  // molecular title
        (t)
    )
}

fn parse_molecule_specs(s: &str) -> IResult<&str, Molecule> {
    let read_atoms = many1(atom_line);
    let read_bonds = opt(many1(read_connect_line));
    do_parse!(
        s,
        read_charge_and_spin_list >> // charge and spin multipy
        atoms: read_atoms  >>        // atom symbol, coordinates, atom type, ...
        blank_line         >>
        bonds: read_bonds  >>        // connectivity section
        ({
            let mut mol = Molecule::default();
            let atoms = atoms.into_iter().map(|x| Atom::new(x.element_label, x.position));
            mol.add_atoms_from((1..).zip(atoms));

            // handle bonds
            if let Some(bonds) = bonds {
                for (u, v, b) in bonds.into_iter().flatten() {
                    mol.add_bond(u, v, b);
                }
            }
            mol
        })
    )
}

// FIXME: how about gaussian extra input
pub fn parse_molecule(s: &str) -> Result<Molecule> {
    let (r, title) = parse_molecule_meta(s).map_err(|e| format_err!("{}", e))?;
    // We replace comma with space in molecular specification part for easy
    // parsing.
    let r = r.replace(",", " ");
    let (r, mut mol) = parse_molecule_specs(&r).map_err(|e| {
        format_err!(
            "parsing gaussian input molecule specification failure: {:}.\n Input stream: {}",
            e,
            r
        )
    })?;
    mol.set_title(&title);

    Ok(mol)
}

#[test]
fn test_parse_gaussian_molecule() {
    let txt = "%chk=C5H12.chk
%nproc=8
%mem=5GB
#p opt freq=noraman nosymm B3LYP/6-31+G** test geom=connect

Title Card
Required

0 1
 C(Fragment=1)    0         -1.29639700         -0.54790000         -0.04565800 L
 H(Fragment=1)    0         -0.94903500         -1.58509500         -0.09306500 L
 H(Fragment=1)    0         -0.93491200         -0.03582400         -0.94211800 L
 H(Fragment=1)    0         -2.39017900         -0.56423400         -0.09090100 L
 C(Fragment=2)    0         -0.80594100          0.14211700          1.23074100 L
 H(Fragment=2)    0         -1.13863100          1.18750400          1.23095600 L
 H(Fragment=2)    0          0.29065200          0.17299300          1.23044100 L
 C(Fragment=3)    0         -1.29047900         -0.54051400          2.51480900 L
 H(Fragment=3)    0         -0.95681000         -1.58684300          2.51564600 L
 H(Fragment=3)    0         -2.38822700         -0.57344700          2.51397600 L
 C(Fragment=4)    0         -0.80793200          0.14352700          3.79887800 L
 H(Fragment=4)    0         -1.14052400          1.18894500          3.79694800 L
 H(Fragment=4)    0          0.28866200          0.17430200          3.80089400 L
 C(Fragment=5)    0         -1.30048800         -0.54500600          5.07527000 L
 H(Fragment=5)    0         -0.95334000         -1.58219500          5.12438500 L
 H(Fragment=5)    0         -0.94034400         -0.03198400          5.97172800 L
 H(Fragment=5)    0         -2.39434000         -0.56114800          5.11880700 L

 1 2 1.0 3 1.0 4 1.0 5 1.0
 2
 3
 4
 5 6 1.0 7 1.0 8 1.0
 6
 7
 8 9 1.0 10 1.0 11 1.0
 9
 10
 11 12 1.0 13 1.0 14 1.0
 12
 13
 14 15 1.0 16 1.0 17 1.0
 15
 16
 17
";

    let mol = parse_molecule(txt).expect("gjf molecule");
    assert_eq!(17, mol.natoms());
    assert_eq!(16, mol.nbonds());
}
// parse:1 ends here

// format

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*format][format:1]]
// TODO: atom properties
fn format_atom(a: &Atom) -> String {
    let [x, y, z] = a.position();
    format!(
        " {symbol:15} {x:14.8} {y:14.8} {z:14.8}\n",
        symbol = a.symbol(),
        x = x,
        y = y,
        z = z,
    )
}

// string representation in gaussian input file format
fn format_molecule(mol: &Molecule) -> String {
    let mut lines = String::new();

    let link0 = "%nproc=1\n%mem=20MW";
    let route = "#p sp scf=tight HF/3-21G* geom=connect test";
    lines.push_str(&format!("{}\n{}\n", link0, route));
    lines.push_str("\n");

    // title section
    lines.push_str("Title Card Required\n");
    lines.push_str("\n");

    // TODO: take from molecule
    lines.push_str("0 1\n");
    for (_, a) in mol.atoms() {
        let line = format_atom(&a);
        lines.push_str(&line);
    }

    // crystal vectors
    if let Some(lattice) = mol.lattice {
        // let va = lattice.vector_a();
        // let vb = lattice.vector_b();
        // let vc = lattice.vector_c();
        for l in lattice.vectors().iter() {
            lines.push_str(&format!(" TV              {:14.8}{:14.8}{:14.8}\n", l.x, l.y, l.z));
        }
    }

    // connectivity
    lines.push_str("\n");
    let mut map = HashMap::new();
    for (i, j, b) in mol.bonds() {
        let mut neighbors = map.entry(i).or_insert(vec![]);
        neighbors.push((j, b.order()));
    }
    for (i, a) in mol.atoms() {
        let mut line = format!("{:<5}", i);
        if let Some(neighbors) = map.get(&i) {
            for (j, o) in neighbors {
                line.push_str(&format!(" {:<} {:<.1}", j, o));
            }
        }
        lines.push_str(&format!("{}\n", line));
    }

    lines.push_str("\n");
    lines
}
// format:1 ends here

// impl chemfile

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*impl chemfile][impl chemfile:1]]
#[derive(Clone, Copy, Debug)]
/// plain xyz coordinates with atom symbols
pub struct GaussianInputFile();

/// References
/// http://gaussian.com/input/?tabid=0
impl ChemicalFile for GaussianInputFile {
    fn ftype(&self) -> &str {
        "gaussian/input"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".gjf", ".com", ".gau"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        Ok(format_molecule(mol))
    }
}

impl ParseMolecule for GaussianInputFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        parse_molecule(input)
    }
}

impl Partition for GaussianInputFile {
    fn read_next(&self, context: ReadContext) -> bool {
        let line = context.this_line();
        let link1 = "--link1--\n";
        line.to_lowercase() != link1
    }
}
// impl chemfile:1 ends here
