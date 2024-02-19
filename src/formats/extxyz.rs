// [[file:../../gchemol-readwrite.note::574001a8][574001a8]]
use super::*;
// 574001a8 ends here

// [[file:../../gchemol-readwrite.note::7636c8a8][7636c8a8]]
/// The extended XYZ format
#[derive(Copy, Clone, Debug)]
pub struct ExtxyzFile();
// 7636c8a8 ends here

// [[file:../../gchemol-readwrite.note::*read][read:1]]

// read:1 ends here

// [[file:../../gchemol-readwrite.note::ec30581c][ec30581c]]

// ec30581c ends here

// [[file:../../gchemol-readwrite.note::9078bfde][9078bfde]]
impl ExtxyzFile {
    /// Return Lattice object by reading data from `line` in ase [extxyz](https://wiki.fysik.dtu.dk/ase/ase/io/formatoptions.html#xyz) format.
    ///
    /// Lattice="13.5142 0.0 0.0 0.0 14.9833 0.0 0.0 0.0 20.0" Properties=species:S:1:pos:R:3 pbc="T T T"
    pub fn read_lattice(line: &str) -> Option<Lattice> {
        if line.starts_with("Lattice=") {
            if let Some(pos) = &line[9..].find("\"") {
                let lattice_str = &line[9..pos + 9];
                let lattice_numbers: Vec<f64> = lattice_str.split_ascii_whitespace().filter_map(|value| value.parse().ok()).collect();
                if lattice_numbers.len() != 9 {
                    return None;
                }
                let va: [f64; 3] = lattice_numbers[..3].try_into().ok()?;
                let vb: [f64; 3] = lattice_numbers[3..6].try_into().ok()?;
                let vc: [f64; 3] = lattice_numbers[6..9].try_into().ok()?;
                let lat = Lattice::new([va, vb, vc]);
                return Some(lat);
            }
        }
        None
    }
}

#[test]
fn test_lattice_extxyz() {
    let line = "Lattice=\"13.5142 0.0 0.0 0.0 14.9833 0.0 0.0 0.0 20.0\" Properties=species:S:1:pos:R:3 pbc=\"T T T\"";
    let lat = ExtxyzFile::read_lattice(line);
    assert!(lat.is_some());
    assert_eq!(lat.unwrap().lengths(), [13.5142, 14.9833, 20.0]);
}
// 9078bfde ends here
