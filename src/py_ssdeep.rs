use crate::Hasher;
use crate::fuzzyhash::constants::Modes;
use crate::fuzzyhash::hasher::Hasher as Ssdeep;

use pyo3::exceptions::PyValueError;
use pyo3::{PyResult, pyclass, pymethods};

impl From<Ssdeep> for PySsdeep {
    fn from(hasher: Ssdeep) -> Self {
        Self { hasher }
    }
}

impl Hasher for PySsdeep {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data, data.len());
    }

    fn finalize(&self) -> String {
        let mut cloned = self.hasher.clone();
        cloned.digest(Modes::None).unwrap()
    }

    fn serialize(&self) -> Vec<u8> {
        serde_cbor::ser::to_vec_packed(&self.hasher).unwrap()
    }

    fn deserialize(data: &[u8]) -> PyResult<Self> {
        let hasher = serde_cbor::from_slice::<Ssdeep>(data)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PySsdeep::from(hasher))
    }
}

#[pyclass]
#[derive(Default)]
pub struct PySsdeep {
    hasher: Ssdeep,
}

#[pymethods]
impl PySsdeep {
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
