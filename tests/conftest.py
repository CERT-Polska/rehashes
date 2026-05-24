import random
import pytest


@pytest.fixture(scope="session")
def random_file():
    SEED = 0xDEADBEEF
    SIZE_BYTES = 50 * 1024 * 1024  # 50MB in bytes
    rng = random.Random(SEED)
    data = bytearray(rng.randbytes(SIZE_BYTES))    
    yield data
