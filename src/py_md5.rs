use crate::Hasher;
use md5::digest::array::Array;
use md5::digest::common::hazmat::SerializableState;
use md5::{Digest, Md5};
use pyo3::exceptions::PyValueError;
use pyo3::{PyResult, pyclass, pymethods};

impl From<Md5> for PyMd5 {
    fn from(hasher: Md5) -> Self {
        Self { hasher }
    }
}

impl Hasher for PyMd5 {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    fn finalize(&self) -> String {
        let cloned = self.hasher.clone();
        let result = cloned.finalize();
        hex::encode(result)
    }

    fn serialize(&self) -> Vec<u8> {
        self.hasher.serialize().to_vec()
    }

    fn deserialize(data: &[u8]) -> PyResult<Self> {
        let hasher = Md5::deserialize(&Array::try_from(data)?)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyMd5::from(hasher))
    }
}

#[pyclass]
#[derive(Default)]
pub struct PyMd5 {
    hasher: Md5,
}

#[pymethods]
impl PyMd5 {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    fn update(&mut self, data: &[u8]) {
        <Self as Hasher>::update(self, data);
    }

    pub fn finalize(&self) -> String {
        <Self as Hasher>::finalize(self)
    }

    pub fn serialize(&self) -> Vec<u8> {
        <Self as Hasher>::serialize(self)
    }

    #[staticmethod]
    fn deserialize(data: &[u8]) -> PyResult<Self> {
        <Self as Hasher>::deserialize(data)
    }
}
