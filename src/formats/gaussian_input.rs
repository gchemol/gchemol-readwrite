// [[file:../../gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use super::parser::*;
// imports:1 ends here

// [[file:../../gchemol-readwrite.note::*header][header:1]]
// Gaussian input file
//
// Reference
// ---------
// http://gaussian.com/input/?tabid=0
// header:1 ends here

// [[file:../../gchemol-readwrite.note::d068aeaf][d068aeaf]]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaussianAtomInfo {
    element_label: String,
    mm_type: Option<String>,
    mm_charge: Option<f64>,
    frozen_code: Option<isize>,
    position: [f64; 3],
    properties: HashMap<String, String>,
    oniom_info: OniomInfo,
}

impl Default for GaussianAtomInfo {
    fn default() -> Self {
        GaussianAtomInfo {
            element_label: "C".to_owned(),
            mm_type: None,
            mm_charge: None,
            frozen_code: None,
            properties: HashMap::new(),
            position: [0.0; 3],
            oniom_info: OniomInfo::default(),
        }
    }
}

impl GaussianAtomInfo {
    /// Set ONIOM layer for atom
    pub fn set_oniom_layer(&mut self, layer: &str) {
        self.oniom_info.layer = layer.to_owned().into();
    }

    /// Get ONIOM layer from atom
    pub fn get_oniom_layer(&self) -> Option<&str> {
        self.oniom_info.layer.as_ref().map(|x| x.as_str())
    }

    /// Set ONIOM link host atom
    pub fn set_oniom_link_host(&mut self, link_host: usize) {
        self.oniom_info.link_host = link_host.into();
    }

    /// Get ONIOM link host atom
    pub fn get_oniom_link_host(&mut self) -> Option<usize> {
        self.oniom_info.link_host
    }

    /// Set ONIOM link atom type
    pub fn set_oniom_link_atom(&mut self, link_atom: &str) {
        self.oniom_info.link_atom = link_atom.to_owned().into();
    }

    /// Get ONIOM link atom type
    pub fn get_oniom_link_atom(&mut self) -> Option<&str> {
        self.oniom_info.link_atom.as_ref().map(|x| x.as_str())
    }

    /// Attach/store extra properties to an `atom`.
    pub fn attach(&self, atom: &mut Atom) {
        let _ = atom.properties.store(ATOM_KEY_ONIOM_LAYER, &self);
    }

    /// Extract extra properties from `atom`.
    pub fn extract(atom: &Atom) -> Result<Self> {
        let x = atom.properties.load(ATOM_KEY_ONIOM_LAYER)?;
        Ok(x)
    }
}
// d068aeaf ends here

// [[file:../../gchemol-readwrite.note::*link0 section][link0 section:1]]
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

// [[file:../../gchemol-readwrite.note::*route section][route section:1]]
fn blank_line(s: &str) -> IResult<&str, ()> {
    do_parse!(s, space0 >> eol >> (()))
}

fn route_section(s: &str) -> IResult<&str, String> {
    let mut pound = tag("#");
    let mut print_level = opt(alt((tag_no_case("N"), tag_no_case("P"), tag_no_case("T"))));
    let mut keywords = many_till(read_until_eol, blank_line);
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

// [[file:../../gchemol-readwrite.note::*title section][title section:1]]
fn title_section(s: &str) -> IResult<&str, String> {
    let mut title_lines = many_till(read_until_eol, blank_line);
    do_parse!(
        s,
        lines: title_lines >> // xx
        ({
            lines.0.join(" ")
        })
    )
}
// title section:1 ends here

// [[file:../../gchemol-readwrite.note::766dccd9][766dccd9]]
// Specifies the net electric charge (a signed integer) and the spin
// multiplicity (usually a positive integer)
fn read_charge_and_spin_list(s: &str) -> IResult<&str, Vec<isize>> {
    let mut values = separated_list1(space1, signed_digit);
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
// 766dccd9 ends here

// [[file:../../gchemol-readwrite.note::*element info][element info:1]]
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
    let params = separated_list1(tag(","), atom_param);
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

// [[file:../../gchemol-readwrite.note::0d3a42da][0d3a42da]]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OniomInfo {
    layer: Option<String>,
    link_atom: Option<String>,
    link_host: Option<usize>,
    // TODO: scale factor
}

impl std::fmt::Display for OniomInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(layer) = &self.layer {
            write!(f, "{layer}")?;
            if let Some(link_atom) = &self.link_atom {
                write!(f, " {link_atom}")?;
                if let Some(link_host) = &self.link_host {
                    write!(f, " {link_host}")?;
                }
            }
        }
        Ok(())
    }
}

impl OniomInfo {
    fn from_str(s: &str) -> Self {
        let mut oniom = Self::default();
        let parts = s.split_whitespace().collect_vec();
        if parts.is_empty() {
            return oniom;
        }
        oniom.layer = parts[0].to_owned().into();
        if parts.len() >= 2 {
            oniom.link_atom = parts[1].to_owned().into();
            if parts.len() >= 3 {
                oniom.link_host = parts[2].parse().ok();
            }
        }
        return oniom;
    }
}

// C-CA--0.25   0   -4.703834   -1.841116   -0.779093 L
// C-CA--0.25   0   -3.331033   -1.841116   -0.779093 L H-HA-0.1  3
fn atom_oniom_info(s: &str) -> IResult<&str, OniomInfo> {
    do_parse!(
        s,
        s: read_line >> // ONIOM layer: H, M, L
        (OniomInfo::from_str(s))
    )
}

#[test]
fn test_gjf_atom_oniom_params() {
    let (_, oniom) = atom_oniom_info(" L H-Ha-0.1 3\n").unwrap();
    assert_eq!("L", oniom.layer.unwrap());
    assert_eq!("H-Ha-0.1", oniom.link_atom.unwrap());
    assert_eq!(Some(3), oniom.link_host);

    let (_, oniom) = atom_oniom_info(" L\n").unwrap();
    assert_eq!("L", oniom.layer.unwrap());
}
// 0d3a42da ends here

// [[file:../../gchemol-readwrite.note::dfcbceb2][dfcbceb2]]
use std::collections::HashMap;
use std::iter::FromIterator;

fn frozen_code(s: &str) -> IResult<&str, isize> {
    terminated(signed_digit, space1)(s)
}

// How about this: C-CA--0.25(fragment=1,iso=13,spin=3) 0 0.0 1.2 3.4 H H-H_
fn atom_line(s: &str) -> IResult<&str, GaussianAtomInfo> {
    let mut mm_info = opt(atom_mm_info);
    let mut params = opt(atom_params);
    let mut frozen = opt(frozen_code);
    do_parse!(
        s,
        space0 >> e: alphanumeric1      >> // element label
        m: mm_info                      >> // MM type and MM partial charge
        p: params                       >> // Optional atom info, such as fragment, nuclei props
        space1                          >> // xx
        f: frozen                       >> // frozen code: 0, -1
        c: xyz_array                    >> // x, y, z coords
        o: atom_oniom_info              >> // H H-H_
        ({
            GaussianAtomInfo {
                element_label: e.to_owned(),
                mm_type: m.and_then(|x| Some(x.0.to_owned())),
                mm_charge: m.and_then(|x| x.1),
                frozen_code: f,
                position: c,
                properties: if p.is_some() {p.unwrap().into_iter().map(|(k, v)| (k.to_owned(), v.to_owned())).collect()} else {HashMap::new()},
                oniom_info: o,
                ..Default::default()
            }
        })
    )
}
// dfcbceb2 ends here

// [[file:../../gchemol-readwrite.note::58144795][58144795]]
#[test]
fn test_gjf_atom_line() {
    let line = "C-CA--0.25(fragment=1,iso=13,spin=3)  -1 0.00   0.00   0.00 L H-HA-0.1  3\n";
    let (_, ga) = atom_line(line).expect("full oniom atom line");
    assert_eq!(ga.oniom_info.layer.unwrap(), "L");

    let line = "C-CA--0.25(fragment=1,iso=13,spin=3) -1 0.00   0.00   0.00 L \n";
    let (_, ga) = atom_line(line).expect("oniom without link atom");
    assert_eq!(ga.oniom_info.layer.unwrap(), "L");
    assert_eq!(ga.frozen_code, Some(-1));

    let line = "C-CA--0.25(fragment=1,iso=13,spin=3) -1 0.00   0.00   0.00\n";
    let (_, ga) = atom_line(line).expect("frozen + xyz");
    assert_eq!(ga.frozen_code, Some(-1));

    let line = "C-CA--0.25(fragment=1,iso=13,spin=3) 0.00   0.00   0.00\n";
    let (_, ga) = atom_line(line).expect("missing frozen code");
    assert_eq!(ga.mm_charge, Some(-0.25));
    assert_eq!(ga.mm_type.unwrap(), "CA");
    assert_eq!(ga.properties["fragment"], "1");

    let (_, ga) = atom_line("C-CA--0.25  0.00   0.00   0.00\n").expect("no key-value params");
    assert_eq!(ga.mm_charge, Some(-0.25));
    assert_eq!(ga.mm_type.unwrap(), "CA");

    let (_, ga) = atom_line("C-C_3 0.00 0.00 0.00\n").expect("no mm charge");
    assert_eq!(ga.mm_type.unwrap(), "C_3");
    assert_eq!(ga.mm_charge, None);

    let line = " C12(fragment=1)  0.00   0.00   0.00\n";
    let (_, ga) = atom_line(line).expect("key-value params only");
    assert_eq!(ga.properties["fragment"], "1");
    assert_eq!(ga.position[0], 0.0);

    let line = " C12  0.00   0.00   0.00\n";
    let (_, ga) = atom_line(line).expect("simple");
    assert_eq!(ga.position[0], 0.0);
}
// 58144795 ends here

// [[file:../../gchemol-readwrite.note::*connectivity specs][connectivity specs:1]]
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
    let mut read_bond_items = many0(bond_item);
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

// [[file:../../gchemol-readwrite.note::fc56d173][fc56d173]]
const ATOM_KEY_ONIOM_LAYER: &'static str = "gaussian-oniom-layer";

fn parse_molecule_meta(s: &str) -> IResult<&str, String> {
    let mut link0 = opt(link0_section);
    do_parse!(
        s,
        link0 >>             // Link0 commands, which is optional section
        route_section >>     // route section contains job keywords
        t: title_section >>  // molecular title
        (t)
    )
}

fn parse_molecule_specs(s: &str) -> IResult<&str, Molecule> {
    let mut read_atoms = many1(atom_line);
    let mut read_bonds = opt(many1(read_connect_line));
    do_parse!(
        s,
        read_charge_and_spin_list >> // charge and spin multipy
        atoms: read_atoms         >> // atom symbol, coordinates, atom type, ...
        blank_line                >>
        bonds: read_bonds         >> // connectivity section
        ({
            let mut mol = Molecule::default();
            let mut lat_vectors = vec![];
            let atoms = atoms.into_iter().filter_map(|x| {
                // Handle dummy TV atoms (transitional vector)
                if &x.element_label.to_uppercase() == "TV" {
                    debug!("found TV dummy atom");
                    lat_vectors.push(x.position.clone());
                    None
                } else {
                    // set atom freezing flag
                    let mut a = Atom::new(x.element_label.clone(), x.position.clone());
                    if let Some(f) = x.frozen_code {
                        if f == -1 {
                            a.set_freezing([true; 3]);
                        }
                    }
                    // store extra atom properties such as ONIOM layer or fragment index.
                    x.attach(&mut a);
                    Some(a)
                }
            });
            mol.add_atoms_from((1..).zip(atoms));
            if lat_vectors.len() == 3 {
                let lat = Lattice::new([lat_vectors[0], lat_vectors[1], lat_vectors[2]]);
                mol.set_lattice(lat);
            } else if !lat_vectors.is_empty() {
                error!("Expect 3, but found {} TV atoms.", lat_vectors.len());
            }

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
    let (r, mut mol) =
        parse_molecule_specs(&r).map_err(|e| format_err!("parsing gaussian input molecule specification failure: {:}.\n Input stream: {}", e, r))?;
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
 C(Fragment=1)   -1         -1.29639700         -0.54790000         -0.04565800 L
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
// fc56d173 ends here

// [[file:../../gchemol-readwrite.note::5605d45c][5605d45c]]
// TODO: atom properties
fn format_atom(a: &Atom) -> String {
    let [x, y, z] = a.position();
    let symbol = a.symbol();
    let fcode = if a.freezing() == [true; 3] { -1 } else { 0 };
    let part = format!(" {symbol:15} {fcode:2} {x:14.8} {y:14.8} {z:14.8}");

    // format ONIOM layer, link atom, link host
    if let Ok(extra) = GaussianAtomInfo::extract(&a) {
        let oniom = extra.oniom_info;
        format!("{part} {oniom}\n")
    } else {
        format!("{part}\n")
    }
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
// 5605d45c ends here

// [[file:../../gchemol-readwrite.note::d13a6041][d13a6041]]
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
// d13a6041 ends here

// [[file:../../gchemol-readwrite.note::2530a669][2530a669]]
impl ReadPart for GaussianInputFile {
    fn read_next(&self, context: ReadContext) -> ReadAction {
        Terminated(|line: &str| {
            let link1 = "--link1--\n";
            line.to_lowercase() == link1
        })
        .read_next(context)
    }
}
// 2530a669 ends here

// [[file:../../gchemol-readwrite.note::457b015d][457b015d]]
#[test]
fn test_gaussian_input_file() -> Result<()> {
    let s = gut::fs::read_file("tests/files/gaussian/test0769.com")?;
    let mol = GaussianInputFile().parse_molecule(&s)?;
    let s = format_molecule(&mol);
    println!("{s}\n****");
    Ok(())
}
// 457b015d ends here
