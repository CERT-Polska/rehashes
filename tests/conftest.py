import random
import pytest


@pytest.fixture(scope="session")
def random_file():
    SEED = 0xDEADBEEF
    SIZE_BYTES = 50 * 1024 * 1024  # 50MB in bytes
    rng = random.Random(SEED)
    data = bytearray(rng.randbytes(SIZE_BYTES))    
    yield data


def pytest_addoption(parser):
    parser.addoption("--slow", action="store_true",
                     help="run the slow tests")


def pytest_runtest_setup(item):
    if 'slow' in item.keywords and not item.config.getoption("--slow"):
        pytest.skip("need --slow option to run this test")
