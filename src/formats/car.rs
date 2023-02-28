// [[file:../../gchemol-readwrite.note::fd421c85][fd421c85]]
use super::parser::*;
use gut::prelude::*;
// fd421c85 ends here

// [[file:../../gchemol-readwrite.note::a5f1e12d][a5f1e12d]]
fn read_pbc_params(s: &str) -> IResult<&str, [f64; 6]> {
    let head = terminated(tag("PBC"), space1);
    let (s, _) = context("pbc line", head)(s)?;
    let (r, p) = separated_list1(space1, not_space)(s)?;
    let params: Vec<f64> = p.iter().take(6).filter_map(|x| x.parse().ok()).collect();
    if params.len() < 6 {
        error!("invalid pbc params: {params:?}");
        return parse_error(s);
    }

    if p.len() > 6 {
        let space_group_name = p[6];
        if space_group_name != "(P1)" {
            warn!("symmetry other than P1 is not handled:\n{space_group_name}");
        }
    }

    Ok((r, params.try_into().unwrap()))
}

#[test]
fn test_arc_pbc() {
    gut::cli::setup_logger_for_test();

    let line = "PBC    8.86010000   21.50090000   20.05100000   90.00000000   90.00000000   93.93820000";
    let (_, params) = read_pbc_params(line).unwrap();
    assert_eq!(params[0], 8.86010);
    assert_eq!(params[5], 93.9382);
    let line = "PBC    5.0256    5.0256   13.6943   90.0000   90.0000  120.0000 (P1)";
    let (_, params) = read_pbc_params(line).unwrap();
    assert_eq!(params[0], 5.0256);
    assert_eq!(params[5], 120.0);
}
// a5f1e12d ends here

// [[file:../../gchemol-readwrite.note::2fa00fc9][2fa00fc9]]
use gchemol_core::Atom;

fn read_atom_line(s: &str) -> IResult<&str, Atom> {
    let (s, (name, _, coords)) = tuple((not_space, space1, xyz_array))(s)?;
    // take only element symbol, ignoroing remaining part
    let symbol:String = name.chars().take_while(|x| x.is_ascii_alphabetic()).collect();
    let atom = Atom::new(symbol, coords);
    Ok((s, atom))
}
// 2fa00fc9 ends here
