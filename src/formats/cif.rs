// header

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*header][header:1]]
// Data will be parsed:
// Lattice, Atoms, Bonds
// header:1 ends here

// imports

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use super::parser::*;
// imports:1 ends here

// base

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*base][base:1]]
/// Recognizes a float point number with uncertainty brackets
///
/// # Example
///
/// 10.154(2)
fn double_cif(s: &str) -> IResult<&str, f64> {
    use nom::number::complete::recognize_float;

    let xx = opt(delimited(tag("("), digit1, tag(")")));
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

// cell

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*cell][cell:1]]
// fn read_tagged_double<'a>(t: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, f64> {
//     use nom::sequence::tuple;

//     map(
//         tuple((space0, tag(t), space1, double, eol)),
//         |(_, _, _, e, _)| e,
//     )
// }

fn cell_params_xx(s: &str) -> IResult<&str, (&str, f64)> {
    let tag_cell = tag("_cell_");
    do_parse!(
        s,
        space0 >> tag_cell >> l: not_space >> space1 >> v: double_cif >> eol >> ((l, v))
    )
}

fn read_cell_params(s: &str) -> IResult<&str, Vec<(&str, f64)>> {
    let jump = take_until("_cell_length_a");
    let read_params = many1(cell_params_xx);
    do_parse!(
        s,
        jump >> // xx
        params: read_params >> // xx
        (params)
    )
}

fn read_cell(s: &str) -> Result<[f64; 6]> {
    let (_, values) = read_cell_params(s).map_err(|e| format_err!("read cell failure"))?;

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
loop_
";

    let param = read_cell(txt).context("cif cell")?;
    assert_eq!(param[1], 20.5160);

    Ok(())
}
// cell:1 ends here

// atoms
// # Example
// loop_
// _atom_site_label
// _atom_site_type_symbol
// _atom_site_fract_x
// _atom_site_fract_y
// _atom_site_fract_z
// Cu1 Cu 0.20761(4) 0.65105(3) 0.41306(4)
// O1 O 0.4125(2) 0.6749(2) 0.5651(3)
// O2 O 0.1662(2) 0.4540(2) 0.3821(3)
// O3 O 0.4141(4) 0.3916(3) 0.6360(4)
// N1 N 0.2759(3) 0.8588(2) 0.4883(3)
// ...

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*atoms][atoms:1]]
fn atom_site_column_name(s: &str) -> IResult<&str, &str> {
    let col_name = preceded(tag("_atom_site_"), not_space);
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
    preceded(tag("loop_\n"), many1(atom_site_column_name))(s)
}

#[test]
fn test_atom_site_column_names() -> Result<()> {
    let txt = "loop_
    _atom_site_label
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
    let read_items = separated_nonempty_list(space1, not_space);
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
fn read_atoms(s: &str) -> Result<Vec<Atom>> {
    // column header loopup table
    // Example
    // -------
    //   0        1         2       3      4            5          6         7
    // label type_symbol fract_x fract_y fract_z U_iso_or_equiv adp_type occupancy
    let (r, headers) = read_atom_site_column_names(&s).unwrap();
    let n_columns = headers.len();

    let table: std::collections::HashMap<_, _> = headers.iter().zip(0..).collect();
    let ifx = *table.get(&"fract_x").expect("missing fract x col");
    let ify = *table.get(&"fract_y").expect("missing fract y col");
    let ifz = *table.get(&"fract_z").expect("missing fract z col");
    // column index to atom label
    let ilbl = *table.get(&"label").expect("missing atom label col");
    // TODO: column index to element symbol, which is optional
    let isym = *table.get(&"type_symbol").expect("atom symbol col");

    let (_, rows) = read_atom_site_rows(r).unwrap();
    let mut atoms = vec![];
    for row in rows {
        // sanity check
        if row.len() != n_columns {
            bail!("malformed atom sites format!");
        }
        // parse fractional coordinates
        let sfx = row[ifx];
        let sfy = row[ify];
        let sfy = row[ify];
        let (_, fx) = double_cif(sfx).expect("invalid fcoords x");
        let (_, fy) = double_cif(sfx).expect("invalid fcoords y");
        let (_, fz) = double_cif(sfx).expect("invalid fcoords z");
        // parse atom symbol
        let lbl = row[ilbl];
        // TODO: assign atom label
        let sym = row[isym];
        let atom: Atom = (sym, [fx, fy, fz]).into();
        atoms.push(atom);
    }

    Ok(atoms)
}

#[test]
fn test_read_cif_atoms() -> Result<()> {
    let txt = "loop_
  _atom_site_label
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
    let v = read_atoms(txt)?;
    assert_eq!(12, v.len());
    Ok(())
}
// atoms:1 ends here

// molecule

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*molecule][molecule:1]]
fn cif_title(s: &str) -> IResult<&str, &str> {
    preceded(tag("data_"), not_space)(s)
}

/// Create Molecule object from cif stream
fn read_molecule(input: &str) -> Result<Molecule> {
    // let jump1 = take_until("_cell_");
    // let jump2 = take_until("_atom_site_");
    // do_parse!(s,
    //           jump >>           // ignore
    //           params: read_cell_params >> // a, b, c, alpha, beta, gamma
    //           jump2 >>                    // to _atom_site_xx
    // )
    todo!()
}
// molecule:1 ends here

// test

// [[file:~/Workspace/Programming/gchemol-rs/gchemol-readwrite/gchemol-readwrite.note::*test][test:1]]
use guts::fs::*;
use guts::prelude::*;

#[test]
fn test_cif() -> Result<()> {
    let f = "../gchemol/readwrite/tests/files/cif/babel.cif";
    let reader = TextReader::from_path(f)?;
    let cif_data_record_label = |line: &str| line.starts_with("loop_");
    let bunches = reader.bunches(cif_data_record_label);

    let mut parts = vec![];
    let mut buf = String::new();
    for lines in bunches {
        // collect each part for further parsing
        for line in lines {
            if !line.is_empty() {
                buf.push_str(&line);
                buf.push_str("\n");
            }
        }
        parts.push(buf.clone());
        buf.clear();
    }
    let mut mol = Molecule::new("cif");
    for part in parts {
        // cell parameters
        if part.contains("_atom_site_fract_x") {
            let atoms = read_atoms(&part)?;
            let atoms = (1..).zip(atoms.into_iter());
            mol.add_atoms_from(atoms);
        } else if part.contains("_cell_length_a") {
            let [a, b, c, alpha, beta, gamma] = read_cell(&part)?;
            let cell = Lattice::from_params(a, b, c, alpha, beta, gamma);
            mol.set_lattice(cell);
        }
    }

    Ok(())
}
// test:1 ends here
