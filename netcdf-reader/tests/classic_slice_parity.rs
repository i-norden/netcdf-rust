use std::path::Path;

use netcdf_reader::{Error, NcFile, NcSliceInfo, NcSliceInfoElem};

const CLASSIC_ROWS: usize = 32;
const CLASSIC_COLS: usize = 64;

fn create_classic_slice_fixture(path: &Path) {
    let mut file = netcdf::create_with(path, netcdf::Options::_64BIT_DATA).unwrap();
    file.add_dimension("row", CLASSIC_ROWS).unwrap();
    file.add_dimension("col", CLASSIC_COLS).unwrap();
    file.add_variable::<f32>("data", &["row", "col"]).unwrap();
    file.enddef().unwrap();

    let mut variable = file.variable_mut("data").unwrap();
    for row in 0..CLASSIC_ROWS {
        let values: Vec<f32> = (0..CLASSIC_COLS)
            .map(|col| ((row * 131 + col * 17) % 997) as f32 * 0.5 + row as f32 * 0.25)
            .collect();
        variable.put_values(&values, (row, ..)).unwrap();
    }
}

fn classic_strided_inner_slice() -> NcSliceInfo {
    NcSliceInfo {
        selections: vec![
            NcSliceInfoElem::Slice {
                start: 3,
                end: CLASSIC_ROWS as u64,
                step: 3,
            },
            NcSliceInfoElem::Slice {
                start: 5,
                end: 60,
                step: 4,
            },
        ],
    }
}

#[test]
fn test_classic_non_record_slice_matches_georust_for_strided_inner_selection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("classic_slice_parity.nc");
    create_classic_slice_fixture(&path);

    let file = NcFile::open(&path).unwrap();
    let selection = classic_strided_inner_slice();
    let actual: ndarray::ArrayD<f32> = file.read_variable_slice("data", &selection).unwrap();

    let reference = netcdf::open(&path).unwrap();
    let expected = reference
        .variable("data")
        .unwrap()
        .get_values::<f32, _>((&[3usize, 5usize], &[10usize, 14usize], &[3isize, 4isize]))
        .unwrap();

    assert_eq!(actual.shape(), &[10, 14]);
    assert_eq!(actual.iter().copied().collect::<Vec<_>>(), expected);

    let promoted = file.read_variable_slice_as_f64("data", &selection).unwrap();
    assert_eq!(promoted.shape(), &[10, 14]);
    let promoted_expected: Vec<f64> = expected.iter().map(|&value| value as f64).collect();
    assert_eq!(
        promoted.iter().copied().collect::<Vec<_>>(),
        promoted_expected
    );
}

#[test]
fn test_classic_non_record_slice_allows_empty_results() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("classic_slice_empty.nc");
    create_classic_slice_fixture(&path);

    let file = NcFile::open(&path).unwrap();
    let selection = NcSliceInfo {
        selections: vec![
            NcSliceInfoElem::Slice {
                start: CLASSIC_ROWS as u64,
                end: u64::MAX,
                step: 1,
            },
            NcSliceInfoElem::Slice {
                start: 0,
                end: u64::MAX,
                step: 1,
            },
        ],
    };

    let actual: ndarray::ArrayD<f32> = file.read_variable_slice("data", &selection).unwrap();
    assert_eq!(actual.shape(), &[0, CLASSIC_COLS]);
    assert!(actual.is_empty());
}

#[test]
fn test_classic_non_record_slice_rejects_start_past_dimension_end() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("classic_slice_oob.nc");
    create_classic_slice_fixture(&path);

    let file = NcFile::open(&path).unwrap();
    let selection = NcSliceInfo {
        selections: vec![
            NcSliceInfoElem::Slice {
                start: CLASSIC_ROWS as u64 + 1,
                end: u64::MAX,
                step: 1,
            },
            NcSliceInfoElem::Slice {
                start: 0,
                end: u64::MAX,
                step: 1,
            },
        ],
    };

    let err = file
        .read_variable_slice::<f32>("data", &selection)
        .unwrap_err();
    assert!(matches!(err, Error::InvalidData(message) if message.contains("slice start")));
}
