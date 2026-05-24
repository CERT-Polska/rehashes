use crate::Hasher;
use pyo3::exceptions::PyValueError;
use pyo3::{PyResult, pyclass, pymethods};
use sha2::digest::array::Array;
use sha2::digest::common::hazmat::SerializableState;
use sha2::{Digest, Sha512};

impl From<Sha512> for PySha512 {
    fn from(hasher: Sha512) -> Self {
        Self { hasher }
    }
}

impl Hasher for PySha512 {
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
        let hasher = Sha512::deserialize(&Array::try_from(data)?)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PySha512::from(hasher))
    }
}

#[pyclass]
#[derive(Default)]
pub struct PySha512 {
    hasher: Sha512,
}

#[pymethods]
impl PySha512 {
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
