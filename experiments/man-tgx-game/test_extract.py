"""Small checks for the prototype's consequential choices, no game required."""

import tempfile
import unittest
from pathlib import Path

from extract import blocks, field, paint_meshes, write_template


class ExtractTests(unittest.TestCase):
    def test_empty_piece_list_does_not_consume_next_field(self):
        self.assertEqual(field("     Pieces: \n     Locators: 0 1\n", "Pieces"), "")

    def test_uv_selection_uses_alias_and_variant(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "model.pim"
            path.write_text('''Material {
    Alias: "paint"
}
Piece {
    Index: 0
    Material: 0
    VertexCount: 3
    TriangleCount: 1
    Stream {
        Tag: "_UV7"
        Aliases: "_TEXCOORD1" "_TEXCOORD2"
        0 ( &00000000 &00000000 )
        1 ( &3f800000 &00000000 )
        2 ( &00000000 &3f800000 )
    }
    Triangles {
        0 ( 0 1 2 )
    }
}
Part {
    Name: "body"
    Pieces: 0
}
''')
            path.with_suffix(".pit").write_text('''Look {
    Name: "default"
    Material {
        Alias: "paint"
        Effect: "eut2.truckpaint.tsnmapuv"
    }
}
Variant {
    Name: "default"
    Part {
        Name: "body"
        Attribute {
            Tag: "visible"
            Value: ( 1 )
        }
    }
}
Variant {
    Name: "hidden"
    Part {
        Name: "body"
        Attribute {
            Tag: "visible"
            Value: ( 0 )
        }
    }
}
''')
            meshes = paint_meshes(path, "")
            self.assertEqual(meshes, [([(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)], [(0, 1, 2)])])
            self.assertEqual(paint_meshes(path, "variant: hidden\n"), [])
            # ConverterPIX already uses image coordinates; no vertical flip.
            svg = Path(directory) / "template.svg"
            self.assertEqual(write_template(svg, meshes, [2048, 1024]), 3)
            self.assertIn("0.000,1024.000", svg.read_text())
            traits = path.with_suffix(".pit")
            traits.write_text(traits.read_text().replace("truckpaint.tsnmapuv", "truckpaint.altuv"))
            with self.assertRaisesRegex(ValueError, "Alternate paint UVs"):
                paint_meshes(path, "")

    def test_unclosed_section_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Unclosed"):
            list(blocks("Piece {\n    Index: 0\n", "Piece"))


if __name__ == "__main__":
    unittest.main()
