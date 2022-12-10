// [[file:../../gchemol-readwrite.note::*imports][imports:1]]
use super::*;
use super::parser::*;
// imports:1 ends here

// [[file:../../gchemol-readwrite.note::*data type][data type:1]]
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
enum DataType {
    Integer,
    Real,
    Logical,
    Character1,
    Character2,
}

impl DataType {
    fn width(&self) -> usize {
        use self::DataType::*;

        match self {
            // I, fortran format: 6I12
            Integer    => 12,
            // R, fortran format: 5E16.8
            Real       => 16,
            // L, fortran format: 72L1
            Logical    => 1,
            // C, fortran format: 5A12
            Character1 => 12,
            // H, fortran format: 9A8
            Character2 => 8,
        }
    }
}

impl FromStr for DataType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let dt = match s.trim() {
            "I" => DataType::Integer,
            "R" => DataType::Real,
            "C" => DataType::Character1,
            "H" => DataType::Character2,
            _  => {
                bail!("unkown data type: {}", s.trim());
            }
        };

        Ok(dt)
    }
}

#[test]
fn test_fchk_data_type() {
    let s = "  I";
    let dt = s.parse().expect("fchk data type: I");
    assert_eq!(DataType::Integer, dt);
    assert_eq!(dt.width(), 12);

    let s = " R ";
    let dt = s.parse().expect("fchk data type: R");
    assert_eq!(DataType::Real, dt);
    assert_eq!(dt.width(), 16);
}
// data type:1 ends here

// [[file:../../gchemol-readwrite.note::*data section][data section:1]]
/// Represents a section of data in formatted checkpoint file (fchk)
#[derive(Debug, Clone)]
struct Section<'a> {
    /// An informative section name
    label: &'a str,
    /// Data type: R, I, C, L, H
    data_type: DataType,
    /// if there is array data followed by one or more succeeding lines
    is_array: bool,
    /// The last item in section header representing section value or array size
    value: &'a str,
    /// Members of data array
    data_array: Option<Vec<&'a str>>,
}

// Number of alpha electrons                  I              225
// Nuclear charges                            R   N=         261
// Mulliken Charges                           R   N=          11
fn read_section_header(s: &str) -> IResult<&str, Section> {
    let take40 = take_s(40);
    let take7 = take_s(7);
    let take2 = take_s(2);
    do_parse!(
        s,
        label     : take40  >>      // xx
        data_type : take7   >>      // xx
        array     : take2   >>      // xx
        value     : read_line  >>   // xx
        ({
            Section {
                value: value.trim(),
                label: label.trim(),
                data_type: data_type.parse().expect("dt"),
                is_array: array.trim() == "N=",
                data_array: None,
            }
        })
    )
}

#[test]
fn test_fchk_section_header() {
    let line = "Nuclear charges                            R   N=          11 \n";
    let (_, s) = read_section_header(line).expect("fchk section header");
    assert_eq!("Nuclear charges", s.label);
    assert_eq!(DataType::Real, s.data_type);
    assert_eq!("11", s.value);
    assert!(s.is_array);

    let line = "Number of alpha electrons                  I              225\n";
    let (_, s) = read_section_header(line).expect("fchk section header");
    assert!(!s.is_array);

    let line = "Total Energy                               R     -1.177266205968928E+02\n";
    let (_, s) = read_section_header(line).expect("fchk section header");
    assert!(!s.is_array);
}
// data section:1 ends here
