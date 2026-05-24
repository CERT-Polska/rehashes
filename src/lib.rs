mod fuzzyhash;
pub mod py_md5;
pub mod py_sha1;
pub mod py_sha256;
pub mod py_sha512;
pub mod py_ssdeep;

use pyo3::prelude::*;

use py_md5::PyMd5;
use py_sha1::PySha1;
use py_sha256::PySha256;
use py_sha512::PySha512;
use py_ssdeep::PySsdeep;

pub trait Hasher {
    fn update(&mut self, data: &[u8]);
    fn finalize(&self) -> String;
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> PyResult<Self>
    where
        Self: Sized;
}

#[pymodule]
fn rehashes(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyMd5>()?;
    m.add_class::<PySha1>()?;
    m.add_class::<PySha256>()?;
    m.add_class::<PySha512>()?;
    m.add_class::<PySsdeep>()?;
    Ok(())
}
