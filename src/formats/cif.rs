// [[file:../../gchemol-readwrite.note::*header][header:1]]
// The following data will be parsed:
// Lattice, Atoms, Bonds
// header:1 ends here

// [[file:../../gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use super::parser::*;
// imports:1 ends here

// [[file:../../gchemol-readwrite.note::*base][base:1]]
/// Recognizes a float point number with uncertainty brackets
///
/// # Example
///
/// 10.154(2)
fn double_cif(s: &str) -> IResult<&str, f64> {
    use nom::number::complete::recognize_float;

    let mut xx = opt(delimited(tag("("), digit1, tag(")")));
    do_parse!(
        s,
        v: recognize_float >> // xx
        o: xx              >> // xx
        ({
            let s = if let Some(o) = o {
                v.to_owned() + o
            } else {
                v.to_owned()
            };
            s.parse().expect("cif uncertainty number")
        })
    )
}

#[test]
fn test_cif_float_number() {
    let (_, v) = double_cif("0.3916\n").expect("cif float1");
    assert_eq!(v, 0.3916);
    let (_, v) = double_cif("0.391(6)\n").expect("cif float2");
    assert_eq!(v, 0.3916);
}
// base:1 ends here

// [[file:../../gchemol-readwrite.note::*cell][cell:1]]
fn cell_params_xx(s: &str) -> IResult<&str, (&str, f64)> {
    let tag_cell = tag("_cell_");
    do_parse!(
        s,
        space0 >> tag_cell >> l: not_space >> space1 >> v: double_cif >> eol >> ((l, v))
    )
}

fn read_cell_params(s: &str) -> IResult<&str, Vec<(&str, f64)>> {
    let mut jump = take_until("_cell_");
    let mut read_params = many1(cell_params_xx);
    do_parse!(s, jump >> params: read_params >> (params))
}

fn parse_cell(s: &str) -> Result<[f64; 6]> {
    let (_, values) = read_cell_params(s).map_err(|e| format_err!("read cell failure: {:?}", s))?;

    let d: std::collections::HashMap<_, _> = values.into_iter().collect();
    debug_assert!(d.len() >= 6, "missing cell params: {:?}", d);

    let params = [
        d["length_a"],
        d["length_b"],
        d["length_c"],
        d["angle_alpha"],
        d["angle_beta"],
        d["angle_gamma"],
    ];

    Ok(params)
}

#[test]
fn test_cif_cell_loop() -> Result<()> {
    // Allow data in random order and with blank line
    let txt = "_cell_length_a                    18.094(0)
_cell_length_c                    7.5240
_cell_length_b                    20.5160
_cell_angle_alpha                 90.0000
_cell_angle_beta                  90.0000
_cell_angle_gamma                 90.0000
";

    let param = parse_cell(txt).context("cif cell")?;
    assert_eq!(param[1], 20.5160);

    Ok(())
}
// cell:1 ends here

// [[file:../../gchemol-readwrite.note::*atoms][atoms:1]]
fn atom_site_column_name(s: &str) -> IResult<&str, &str> {
    let mut col_name = preceded(tag("_atom_site_"), not_space);
    do_parse!(s, space0 >> x: col_name >> eol >> (x))
}

#[test]
fn test_atom_site_column_name() -> Result<()> {
    let line = "     _atom_site_fract_z\n";
    let (_, name) = atom_site_column_name(line)?;
    assert_eq!(name, "fract_z");
    Ok(())
}

fn read_atom_site_column_names(s: &str) -> IResult<&str, Vec<&str>> {
    many1(atom_site_column_name)(s)
}

#[test]
fn test_atom_site_column_names() -> Result<()> {
    let txt = "_atom_site_label
_atom_site_type_symbol
_atom_site_fract_x
_atom_site_fract_y
_atom_site_fract_z
_atom_site_occupancy
";
    let (_, col_names) = read_atom_site_column_names(txt)?;
    assert_eq!(col_names.len(), 6);

    Ok(())
}

fn atom_site_row(s: &str) -> IResult<&str, Vec<&str>> {
    let mut read_items = separated_list1(space1, not_space);
    do_parse!(s, space0 >> items: read_items >> eol >> (items))
}

fn read_atom_site_rows(s: &str) -> IResult<&str, Vec<Vec<&str>>> {
    many1(atom_site_row)(s)
}

#[test]
fn test_read_cif_rows() -> Result<()> {
    let txt = "    O1      O    0.60052   0.66147   0.10552   1.000
    N2      N    0.85128   0.69162   0.94732   1.000
    C3      C    0.76514   0.76592   0.04263   1.000
    C4      C    0.02513   0.84479   0.88902   1.000
    C5      C    0.07600   0.01890   0.95060   1.000
";
    let (_, rows) = read_atom_site_rows(txt)?;
    assert_eq!(rows.len(), 5);

    Ok(())
}

/// Read atoms in cif _atom_site loop
fn parse_atoms(s: &str) -> Result<Vec<Atom>> {
    // column header loopup table
    // Example
    // -------
    //   0        1         2       3      4            5          6         7
    // label type_symbol fract_x fract_y fract_z U_iso_or_equiv adp_type occupancy
    let (r, headers) = read_atom_site_column_names(&s).map_err(|e| format_err!("{}", e))?;
    let n_columns = headers.len();

    let table: std::collections::HashMap<_, _> = headers.iter().zip(0..).collect();
    let ifx = *table.get(&"fract_x").expect("missing fract x col");
    let ify = *table.get(&"fract_y").expect("missing fract y col");
    let ifz = *table.get(&"fract_z").expect("missing fract z col");
    // column index to atom label
    let ilbl = *table.get(&"label").expect("missing atom label col");
    // column index to element symbol, which is optional
    let isym_opt = table.get(&"type_symbol");

    let (_, rows) = read_atom_site_rows(r).map_err(|e| format_err!("{}", e))?;
    let mut atoms = vec![];
    for row in rows {
        // sanity check
        if row.len() != n_columns {
            warn!("malformed atom sites format:\n{:?}", row);
            continue;
        }
        // parse fractional coordinates
        let sfx = row[ifx];
        let sfy = row[ify];
        let sfz = row[ifz];
        let (_, fx) = double_cif(sfx).expect("invalid fcoords x");
        let (_, fy) = double_cif(sfy).expect("invalid fcoords y");
        let (_, fz) = double_cif(sfz).expect("invalid fcoords z");
        // parse atom symbol from type_symbol column or atom label column
        let sym: String = if let Some(&isym) = isym_opt {
            row[isym].to_string()
        } else {
            // take only element symbol
            row[ilbl].chars().take_while(|x| x.is_ascii_alphabetic()).collect()
        };
        let mut atom: Atom = (sym, [fx, fy, fz]).into();
        //assign atom label
        atom.set_label(row[ilbl]);
        atoms.push(atom);
    }

    Ok(atoms)
}

#[test]
fn test_read_cif_atoms() -> Result<()> {
    let txt = " _atom_site_label
  _atom_site_type_symbol
  _atom_site_fract_x
  _atom_site_fract_y
  _atom_site_fract_z
  _atom_site_U_iso_or_equiv
  _atom_site_adp_type
  _atom_site_occupancy
  Si1    Si    0.30070   0.07240   0.04120   0.00000  Uiso   1.00
  Si2    Si    0.30370   0.30880   0.04610   0.00000  Uiso   1.00
  O3     O     0.12430   0.41700   0.42870   0.00000  Uiso   1.00
  O4     O     0.12260   0.19540   0.42540   0.00000  Uiso   1.00
  O5     O     0.23620   0.12240   0.98650   0.00000  Uiso   1.00
  Si6    Si    0.80070   0.57240   0.04120   0.00000  Uiso   1.00
  Si7    Si    0.80370   0.80880   0.04610   0.00000  Uiso   1.00
  O8     O     0.62430   0.91700   0.42870   0.00000  Uiso   1.00
  O9     O     0.62260   0.69540   0.42540   0.00000  Uiso   1.00
  O10    O     0.73620   0.62240   0.98650   0.00000  Uiso   1.00
  Si11   Si    0.69930   0.92760   0.54120   0.00000  Uiso   1.00
  Si12   Si    0.69630   0.69120   0.54610   0.00000  Uiso   1.00
  ";
    let v = parse_atoms(txt)?;
    assert_eq!(12, v.len());
    assert_eq!(v[0].position(), [0.30070, 0.07240, 0.04120]);

    Ok(())
}
// atoms:1 ends here

// [[file:../../gchemol-readwrite.note::*parse][parse:1]]
fn cif_title(s: &str) -> IResult<&str, &str> {
    let tag_data = tag("data_");
    do_parse!(s, tag_data >> t: not_space >> eol >> (t))
}

/// Create Molecule object from cif stream
fn parse_molecule(s: &str) -> Result<Molecule> {
    let (r, title) = cif_title(s).map_err(|e| format_err!("{:}", e))?;
    let mut mol = Molecule::new(title);

    for part in r.split("loop_\n") {
        // cell parameters
        if part.contains("_cell_length_a") {
            let [a, b, c, alpha, beta, gamma] = parse_cell(&part.trim_start())?;
            let cell = Lattice::from_params(a, b, c, alpha, beta, gamma);
            mol.set_lattice(cell);
        }
        // atom sites
        if part.contains("_atom_site_fract_x") {
            let atoms = parse_atoms(&part.trim_start())?;
            let atoms = (1..).zip(atoms.into_iter());
            mol.add_atoms_from(atoms);
        }
    }

    Ok(mol)
}
// parse:1 ends here

// [[file:../../gchemol-readwrite.note::078643b6][078643b6]]
/// Represent molecule in .cif format
fn format_molecule(mol: &Molecule) -> Result<String> {
    use std::collections::HashMap;

    let mut lines = String::new();

    // 1. meta inforation
    lines.push_str("data_test\n");
    lines.push_str("_audit_creation_method            'gchemol'\n");
    lines.push_str("_symmetry_space_group_name_H-M    'P1'\n");
    lines.push_str("_symmetry_Int_Tables_number       1\n");
    lines.push_str("_symmetry_cell_setting            triclinic\n");
    lines.push_str("\n");

    // 2. cell parameters
    lines.push_str("loop_\n");
    lines.push_str("_symmetry_equiv_pos_as_xyz\n");
    lines.push_str(" x,y,z\n");

    let mut lat = mol.lattice.ok_or(format_err!("Not a periodic moelcule."))?;
    let [a, b, c] = lat.lengths();
    let [alpha, beta, gamma] = lat.angles();
    lines.push_str(&format!("_cell_length_a     {:10.4}\n", a));
    lines.push_str(&format!("_cell_length_b     {:10.4}\n", b));
    lines.push_str(&format!("_cell_length_c     {:10.4}\n", c));
    lines.push_str(&format!("_cell_angle_alpha  {:10.4}\n", alpha));
    lines.push_str(&format!("_cell_angle_beta   {:10.4}\n", beta));
    lines.push_str(&format!("_cell_angle_gamma  {:10.4}\n", gamma));
    lines.push_str("\n");

    // 3. atom fractional coordinates
    lines.push_str("loop_\n");
    lines.push_str("_atom_site_type_symbol\n");
    lines.push_str("_atom_site_label\n");
    lines.push_str("_atom_site_fract_x\n");
    lines.push_str("_atom_site_fract_y\n");
    lines.push_str("_atom_site_fract_z\n");

    let mut element_count = HashMap::new();
    for (_, a) in mol.atoms() {
        let position = a.position();
        let symbol = a.symbol();
        let c = element_count.entry(symbol).and_modify(|c| *c += 1).or_insert(1);
        // set site label as "Fe12" alike
        let name = a.get_label().map(|l| l.to_string()).unwrap_or(format!("{symbol}{c}"));
        let p = lat.to_frac(position);
        let s = format!("{:4}{:6}{:12.5}{:12.5}{:12.5}\n", symbol, name, p.x, p.y, p.z);
        lines.push_str(&s);
    }

    // 4. bonds
    // if mol.nbonds() > 0 {
    //     lines.push_str("loop_\n");
    //     lines.push_str("_geom_bond_atom_site_label_1\n");
    //     lines.push_str("_geom_bond_atom_site_label_2\n");
    //     lines.push_str("_geom_bond_distance\n");
    //     lines.push_str("_geom_bond_site_symmetry_2\n");
    //     lines.push_str("_ccdc_geom_bond_type\n");
    //     for bond in mol.bonds() {
    //         let symbol1 = frame.symbols.get(&current).unwrap();
    //         let name1 = format!("{}{}", symbol1, current);
    //         let p1 = frame.positions.get(&current).unwrap();
    //         let p1 = Point3::new(p1[0], p1[1], p1[2]) - cell_origin;

    //         let connected = frame.neighbors.get(&current).unwrap();
    //         for other in connected {
    //             if *other > current {
    //                 let symbol2 = frame.symbols.get(&other).unwrap();
    //                 let name2 = format!("{}{}", symbol2, other);
    //                 let p2 = frame.positions.get(&other).unwrap();
    //                 let p2 = Point3::new(p2[0], p2[1], p2[2]) - cell_origin;
    //                 let (image, distance) = get_nearest_image(cell, p1, p2);
    //                 if image.x == 0. && image.y == 0. && image.z == 0. {
    //                     lines.push_str(&format!("{:6} {:6} {:6.3} {:6} S\n", name1, name2, distance, "."));
    //                 } else {
    //                     let symcode = get_image_symcode(image);
    //                     lines.push_str(&format!("{:6} {:6} {:6.3} {:6} S\n", name1, name2, distance, symcode));
    //                     let (image, distance) = get_nearest_image(cell, p2, p1);
    //                     let symcode = get_image_symcode(image);
    //                     lines.push_str(&format!("{:6} {:6} {:6.3} {:6} S\n", name2, name1, distance, symcode));
    //                 }
    //             }
    //         }
    //     }
    // }

    Ok(lines)
}

#[test]
#[ignore]
fn test_cif_format() {
    let mut mol = Molecule::from_database("CH4");
    mol.set_lattice_from_bounding_box(1.0);
    let s = format_molecule(&mol).unwrap();
    println!("{s}");
}
// 078643b6 ends here

// [[file:../../gchemol-readwrite.note::*impl chemfile][impl chemfile:1]]
#[derive(Clone, Copy, Debug)]
pub struct CifFile();

impl ChemicalFile for CifFile {
    fn ftype(&self) -> &str {
        "text/cif"
    }

    fn possible_extensions(&self) -> Vec<&str> {
        vec![".cif"]
    }

    fn format_molecule(&self, mol: &Molecule) -> Result<String> {
        format_molecule(mol)
    }
}

impl ParseMolecule for CifFile {
    fn parse_molecule(&self, input: &str) -> Result<Molecule> {
        let mut mol = parse_molecule(input)?;

        // fract coords => cartesian coords
        let frac_coords: Vec<_> = mol.positions().collect();
        mol.set_scaled_positions(frac_coords);

        Ok(mol)
    }

    /// Skip reading some lines.
    fn pre_read_hook<R: BufRead + Seek>(&self, mut r: TextReader<R>) -> TextReader<R>
    where
        Self: Sized,
    {
        r.seek_line(|line| line.starts_with("data_"));
        r
    }
}
// impl chemfile:1 ends here

// [[file:../../gchemol-readwrite.note::*new][new:1]]
impl ReadPart for CifFile {
    fn read_next(&self, context: ReadContext) -> ReadAction {
        Preceded(|line: &str| line.starts_with("data_")).read_next(context)
    }
}
// new:1 ends here
