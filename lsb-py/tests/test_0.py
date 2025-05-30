import unittest
import lsb_py
from pathlib import Path

class TestLsbPy(unittest.TestCase):
    def setUp(self):
        self.path = Path(__file__).parent
        self.input = self.path / ".." / ".." / "data" / "input.webp"
        self.container = self.path / ".." / ".." / "data" / "container.webp"
        self.embedded = self.path / ".." / ".." / "data" / "embedded.png"

    def test_embed(self):
        with self.input.open("rb") as f:
            input_data = f.read()
        with self.container.open("rb") as f:
            container_data = f.read()

        extension = self.input.suffix[1:]
        result = lsb_py.embed(input_data, extension, container_data)

        self.assertIsInstance(result, bytes)
        self.assertNotEqual(result, container_data)

    def test_extract(self):
        with self.embedded.open("rb") as f:
            embedded_data = f.read()

        result, format = lsb_py.extract(embedded_data)

        self.assertIsInstance(result, bytes)
        self.assertEqual(format, "webp")

    def test_embed_extract(self):
        with self.input.open("rb") as f:
            input_data = f.read()
        with self.container.open("rb") as f:
            container_data = f.read()

        extension = self.input.suffix[1:]
        embedded_data = lsb_py.embed(input_data, extension, container_data)
        result, format = lsb_py.extract(embedded_data)

        self.assertEqual(result, input_data)
        self.assertEqual(format, "webp")


if __name__ == "__main__":
    unittest.main()

