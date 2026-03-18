# Changelog

## 0.1.0 - 2026-03-18

Initial public release.

- Added a pure-Rust, read-only HDF5 decoder in `hdf5-reader`
- Added a pure-Rust NetCDF reader in `netcdf-reader` covering CDF-1/2/5 and NetCDF-4
- Added chunked I/O, filter support, parallel read paths, and cache configuration
- Added Criterion benchmarks against the C-backed `netcdf` crate plus CI benchmark regression checks
