# rehashes

Provides Python hashers with serializable internal state that can be persisted and recovered to continue hashing in another context.

Supported algorithms:

- MD5 (`rehashes.PyMd5`)
- SHA1 (`rehashes.PySha1`)
- SHA256 (`rehashes.PySha256`)
- SHA512 (`rehashes.PySha512`)
- ssdeep ((`rehashes.PySsdeep`, basic fuzzy hash evaluation only)

All hashers share the same minimal interface (update/finalize/serialize/deserialize).

This library is powered by Rust and hash implementations are provided by:
- https://github.com/rustcrypto/hashes
- https://github.com/rustysec/fuzzyhash-rs (embedded into library and modified to be serializable)

## Motivation

Unlike `hashlib`, rehashes hashers support state serialization, allowing you to persist and resume hashing across processes or sessions. Standard `hashlib` objects cannot be pickled or serialized.

Existing libraries are trying to achieve that by serializing OpenSSL opaque structures, which is very unsafe because the internal structures of OpenSSL are not stable and may vary across versions and platforms. rehashes is a Rust wrapper around [rustcrypto/hashes](https://github.com/RustCrypto/hashes) that natively support serialization.

The intended use case of rehashes is to support chunked upload in [MWDB Core](https://github.com/CERT-Polska/mwdb-core) and [Drakvuf Sandbox](https://github.com/CERT-Polska/drakvuf-sandbox) projects. The idea is to be able to stream uploaded chunks to S3 storage and compute hashes of the whole file without the need to re-read it afterward. This is achieved by serializing the internal state of the hasher and storing it in the shared database (e.g. Redis). Then for each chunk, we can recover the state and update/finalize the hash computation.

As rehashes was made for use in MWDB Core, it supports ssdeep (libfuzzy) computation by embedding the [fuzzyhash-rs](https://github.com/rustysec/fuzzyhash-rs) implementation, that was slightly modified to support serialization.

## Installation

```bash
pip install rehashes
```

Pre-built wheels are available for:
- **Linux x86_64** and **aarch64** (manylinux2014 / glibc 2.17+)
- **Python 3.10+** (abi3 stable ABI — one wheel covers all Python versions)

## Usage

### Basic hashing

```python
from rehashes import PySha256

hasher = PySha256()
hasher.update(b"Hello, ")
hasher.update(b"World!")
print(hasher.finalize())
# "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
```

All hashers follow the same interface:

```python
from rehashes import PyMd5, PySha1, PySha256, PySha512, PySsdeep

hasher = PySsdeep()        # or PySha1(), PySha256(), PySha512(), PySsdeep()
hasher.update(data)     # Feed data (bytes) into the hasher
result = hasher.finalize()  # Get the hex digest as a string
```

### Serializable state

> **Warning!**
> Ensure that you're using the same version of library for serializing and deserializing state. Serialized state should be kept server-side and not be exposed e.g. in JWT tokens. Under the hood we use:
> - for MD5/SHA-1/SHA-2: https://docs.rs/crypto-common/0.2.1/crypto_common/hazmat/trait.SerializableState.html
> - for ssdeep: https://docs.rs/serde_cbor/0.11.2/serde_cbor/


The key feature of **rehashes** is the ability to serialize and restore the internal hasher state. This enables use cases like chunked file uploads where you need to compute hashes across multiple sessions without re-reading the entire file.


```python
from rehashes import PySha256

# Session 1: Process first chunk of data
hasher = PySha256()
hasher.update(b"chunk 1 data")

# Serialize and persist the state (e.g., to Redis, database, etc.)
state = hasher.serialize()

# Session 2: Recover state and continue hashing
hasher = PySha256.deserialize(state)
hasher.update(b"chunk 2 data")

# Finalize when all data has been processed
print(hasher.finalize())
```

This works for all supported algorithms including ssdeep:

```python
from rehashes import PySsdeep

hasher = PySsdeep()
hasher.update(b"chunk 1 data")
state = hasher.serialize()  # Persist state to shared storage

# ... later, in another process ...
hasher = PySsdeep.deserialize(state)
hasher.update(b"chunk 2 data")
ssdeep_hash = hasher.finalize()
```

## API Reference

Each hasher class (`PyMd5`, `PySha1`, `PySha256`, `PySha512`, `PySsdeep`) exposes:

| Method | Description |
|--------|-------------|
| `__init__()` | Create a new hasher instance |
| `update(data: bytes)` | Feed data into the hasher |
| `finalize() -> str` | Return the hash digest as a hex string |
| `serialize() -> bytes` | Serialize the internal state to bytes |
| `deserialize(data: bytes) -> Self` | Restore a hasher from serialized state (staticmethod) |

## Development

### Setup

```bash
# Create virtual environment and install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Run tests
pip install pytest
pytest tests/
```

### Building wheels

```bash
# Build wheel for current platform
maturin build --release

# Build manylinux wheel
maturin build --release --manylinux manylinux2014 -i python3.10
```
