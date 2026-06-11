"""Parametrized tests for rehashes hash functions (MD5, SHA1, SHA256, SHA512)."""

import hashlib
from typing import Callable
import subprocess
import pytest
from rehashes import PyMd5, PySha1, PySha256, PySha512, PySsdeep


class SsdeepHasher:
    """Subprocess-based reference implementation for ssdeep hash with hashlib-like interface."""
    
    def __init__(self):
        self._data = b""
    
    def update(self, data: bytes) -> None:
        """Accumulate data to be hashed."""
        self._data += data
    
    def hexdigest(self) -> str:
        """Compute and return the ssdeep hash of accumulated data."""
        result = subprocess.run(
            ["ssdeep"],
            input=self._data,
            capture_output=True,
            check=True,
        )
        # ssdeep output format: "metadata\n1572864:hash:block,filename"
        ssdeep_output = result.stdout.decode().strip()
        
        # Extract the hash portion (everything before the filename comma) from line 2
        lines = ssdeep_output.splitlines()
        return lines[-1].split(",", 1)[0]


HASH_ALGOS = [
    ("md5", PyMd5, hashlib.md5),
    ("sha1", PySha1, hashlib.sha1),
    ("sha256", PySha256, hashlib.sha256),
    ("sha512", PySha512, hashlib.sha512),
    ("ssdeep", PySsdeep, SsdeepHasher),
]


@pytest.mark.parametrize("algo_name,tested_impl,reference_impl", HASH_ALGOS)
def test_hashers(algo_name: str, tested_impl: Callable, reference_impl: Callable, random_file: bytearray):
    """Test PyMd5/PySha1/PySha256/PySha512/PySsdeep produce same hash as reference using update() and finalize()."""
    hasher = tested_impl()
    chunk_size = 1024*1024
    for i in range(0, len(random_file), chunk_size):
        hasher.update(bytes(random_file[i : i + chunk_size]))
    py_hash_result = hasher.finalize()

    expected_hasher = reference_impl()
    for i in range(0, len(random_file), chunk_size):
        expected_hasher.update(bytes(random_file[i : i + chunk_size]))
    expected_hash_result = expected_hasher.hexdigest()

    assert py_hash_result == expected_hash_result


@pytest.mark.parametrize("algo_name,tested_impl,reference_impl", HASH_ALGOS)
def test_hashers_serialized(algo_name: str, tested_impl: Callable, reference_impl: Callable, random_file: bytearray):
    """Test PyMd5/PySha1/PySha256/PySha512/PySsdeep produce same hash as reference using update() and finalize()."""
    hasher = tested_impl()
    chunk_size = 1024*1024
    for i in range(0, len(random_file), chunk_size):
        hasher.update(bytes(random_file[i : i + chunk_size]))
        # Serialize and recover state every chunk
        state = hasher.serialize()
        hasher = tested_impl.deserialize(state)

    py_hash_result = hasher.finalize()

    expected_hasher = reference_impl()
    for i in range(0, len(random_file), chunk_size):
        expected_hasher.update(bytes(random_file[i : i + chunk_size]))
    expected_hash_result = expected_hasher.hexdigest()

    assert py_hash_result == expected_hash_result


@pytest.mark.parametrize("algo_name,tested_impl,reference_impl", HASH_ALGOS)
def test_streaming_pattern_variations(algo_name: str, tested_impl: Callable, reference_impl: Callable):
    """Test different streaming patterns: alternating small/large chunks."""
    data = b"Hello, World! This is a test message for hashing with various chunk sizes." * 100
    
    # Pattern 1: Alternating small (1 byte) and large (100 bytes) chunks
    hasher_alt = tested_impl()
    idx = 0
    while idx < len(data):
        if idx % 2 == 0:
            chunk_size = 1
        else:
            chunk_size = min(100, len(data) - idx)
        hasher_alt.update(data[idx:idx + chunk_size])
        idx += max(chunk_size, 1)
    py_hash_alt = hasher_alt.finalize()

    # Pattern 2: All small chunks (8 bytes)
    hasher_small = tested_impl()
    for i in range(0, len(data), 8):
        hasher_small.update(data[i:i+8])
    py_hash_small = hasher_small.finalize()

    # Pattern 3: All large chunks (65536 bytes)
    hasher_large = tested_impl()
    for i in range(0, len(data), 65536):
        hasher_large.update(data[i:i+65536])
    py_hash_large = hasher_large.finalize()

    # Pattern 4: Reference implementation
    expected_hasher = reference_impl()
    expected_hasher.update(data)
    expected_hash_result = expected_hasher.hexdigest()

    assert py_hash_alt == expected_hash_result, f"{algo_name}: alternating pattern mismatch"
    assert py_hash_small == expected_hash_result, f"{algo_name}: small chunk pattern mismatch"
    assert py_hash_large == expected_hash_result, f"{algo_name}: large chunk pattern mismatch"


@pytest.mark.parametrize("algo_name,tested_impl,reference_impl", HASH_ALGOS)
def test_empty_state_serialization(algo_name: str, tested_impl: Callable, reference_impl: Callable):
    """Test serialization of hasher before any updates (empty state)."""
    # Create fresh hasher and serialize immediately
    empty_hasher = tested_impl()
    serialized_empty = empty_hasher.serialize()

    # Deserialize the empty state
    restored_hasher = tested_impl.deserialize(serialized_empty)
    
    # Finalize without any data - should match hashing empty bytes
    py_hash_result = restored_hasher.finalize()

    expected_hasher = reference_impl()
    expected_hash_result = expected_hasher.hexdigest()

    assert py_hash_result == expected_hash_result, f"{algo_name}: empty state serialization mismatch"


@pytest.mark.parametrize("algo_name,tested_impl,reference_impl", HASH_ALGOS)
def test_invalid_serialization_data(algo_name: str, tested_impl: Callable, reference_impl: Callable):
    """Test that invalid serialization data raises appropriate errors."""
    # Test with completely random/invalid bytes
    invalid_data = b"\x00\xFF\xAA\xBB\xCC\xDD\xEE\xFF" * 16
    
    with pytest.raises(Exception):  # Should raise some error (PyValueError or similar)
        tested_impl.deserialize(invalid_data)
    
    # Test with truncated data (too short for valid state)
    truncated_data = b"\x00\x01\x02"
    
    with pytest.raises(Exception):
        tested_impl.deserialize(truncated_data)
    
    # Test with empty bytes
    empty_data = b""
    
    with pytest.raises(Exception):
        tested_impl.deserialize(empty_data)

def test_ssdeep_huge_data():
    tested_hasher = PySsdeep()
    reference_hasher = SsdeepHasher()
    chunk_size = 128 * 1024 * 1024
    chunk_part = bytes([i for i in range(256)])
    for i in range(64):
        chunk = chunk_part * (chunk_size // len(chunk_part))
        tested_hasher.update(chunk)
        reference_hasher.update(chunk)

    tested_result = tested_hasher.finalize()
    reference_result = reference_hasher.hexdigest()

    assert tested_result == reference_result
