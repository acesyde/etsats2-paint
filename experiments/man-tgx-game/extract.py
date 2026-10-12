#!/usr/bin/env python3
"""Throwaway MAN TGX / DAF NGD probe: game models -> templates and manifest.

Usage: python3 extract.py GAME_DIRECTORY CONVERTER_PIX OUTPUT_DIRECTORY [man.tgx_2020|daf.2021]
Requires Pillow. No Carsmaniac or Paintjob Packer input.
"""

import collections
import json
import math
from pathlib import Path
import re
import struct
import subprocess
import sys

from PIL import Image, ImageDraw, ImageFont

VEHICLES = {
    "man.tgx_2020": ("MAN TGX", "MAN", "dlc_man_tgx_2020.scs", "man_tgx_2020", {"gx": "GX", "gm": "GM", "gn": "GN"}),
    "daf.2021": ("DAF NGD", "DAF", "dlc_daf_2021.scs", "daf_2021", {"xg_plus": "XG+", "xg": "XG", "xf": "XF"}),
}


def blocks(text, name):
    """Sections delimited by braces at the same indentation (ConverterPIX)."""
    pattern = re.compile(r"^( *)" + re.escape(name) + r" \{\n", re.M)
    for match in pattern.finditer(text):
        closing = re.search(r"^" + match[1] + r"\}", text[match.end():], re.M)
        if closing is None:
            raise ValueError(f"Unclosed {name} section")
        yield text[match.end():match.end() + closing.start()]


def field(text, name, default=None):
    match = re.search(r"^[ \t]*" + re.escape(name) + r':[ \t]*("[^"\n]*"|[^\n]*)', text, re.M)
    return match[1].strip().strip('"') if match else default


def number(value):
    return struct.unpack(">f", bytes.fromhex(value[1:]))[0] if value.startswith("&") else float(value)


def paint_meshes(pim, definition):
    text = pim.read_text()
    traits = pim.with_suffix(".pit").read_text()
    look_name = field(definition, "look", "default").lower()
    variant_name = field(definition, "variant", "default").lower()
    looks = [b for b in blocks(traits, "Look") if field(b, "Name").lower() == look_name]
    variants = [b for b in blocks(traits, "Variant") if field(b, "Name").lower() == variant_name]
    if len(looks) != 1 or len(variants) != 1:
        raise ValueError(f"Cannot select look/variant {look_name}/{variant_name}: {pim}")
    materials = {field(b, "Alias"): field(b, "Effect") for b in blocks(looks[0], "Material")}
    aliases = [field(b, "Alias") for b in blocks(text, "Material")]
    visible = set()
    for part in blocks(variants[0], "Part"):
        if re.search(r'Tag: "visible"\s+Value: \( 1 \)', part):
            visible.add(field(part, "Name"))
    allowed = set()
    for part in blocks(text, "Part"):
        if field(part, "Name") in visible:
            allowed.update(map(int, field(part, "Pieces", "").split()))
    result = []
    for piece in blocks(text, "Piece"):
        if int(field(piece, "Index")) not in allowed:
            continue
        effect = materials[aliases[int(field(piece, "Material"))]]
        if not effect.startswith("eut2.truckpaint"):
            continue
        # shortcut: regular truckpaint only; verify shader rules before supporting alternate UVs.
        if ".altuv" in effect:
            raise ValueError(f"Alternate paint UVs not implemented: {pim}")
        streams = [b for b in blocks(piece, "Stream") if '"_TEXCOORD1"' in b]
        if len(streams) != 1:
            raise ValueError(f"Ambiguous paint UV stream: {pim}")
        uv = [tuple(map(number, values.split())) for values in re.findall(r"^\s*\d+\s*\(([^)]+)\)", streams[0], re.M)]
        triangles = [tuple(map(int, values.split())) for values in re.findall(r"^\s*\d+\s*\(([^)]+)\)", next(blocks(piece, "Triangles")), re.M)]
        assert len(uv) == int(field(piece, "VertexCount"))
        assert len(triangles) == int(field(piece, "TriangleCount"))
        assert all(len(p) == 2 and all(math.isfinite(v) and -0.001 <= v <= 1.001 for v in p) for p in uv)
        assert all(len(t) == 3 and all(0 <= i < len(uv) for i in t) for t in triangles)
        result.append((uv, triangles))
    return result


def write_template(path, meshes, size):
    edges = set()
    for uv, triangles in meshes:
        for triangle in triangles:
            points = [tuple(round(c, 7) for c in uv[i]) for i in triangle]
            for a, b in zip(points, points[1:] + points[:1]):
                if a != b:
                    edges.add(tuple(sorted((a, b))))
    if not edges:
        raise ValueError(f"Empty paint template: {path}")
    width, height = size
    lines = [f'M{a[0] * width:.3f},{a[1] * height:.3f}L{b[0] * width:.3f},{b[1] * height:.3f}' for a, b in sorted(edges)]
    path.write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">\n'
                    f'<path fill="none" stroke="#444" stroke-width="0.6" d="{" ".join(lines)}"/>\n</svg>\n')
    return len(edges)


def control_image(path, size, index):
    width, height = size
    colors = [(225, 90, 90), (90, 180, 230), (100, 200, 110), (230, 170, 70), (185, 125, 220)]
    image = Image.new("RGB", size, colors[index % len(colors)])
    draw = ImageDraw.Draw(image)
    step = max(64, min(size) // 4)
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", max(12, step // 10))
    for y in range(0, height, step):
        for x in range(0, width, step):
            if (x // step + y // step) % 2:
                draw.rectangle((x, y, x + step - 1, y + step - 1), fill=tuple(min(255, c + 25) for c in colors[index % len(colors)]))
            draw.rectangle((x, y, x + step - 1, y + step - 1), outline="black", width=max(1, step // 100))
            draw.text((x + 5, y + 5), f"{index:02}\n{x // step},{y // step}\n^ TOP >", font=font, fill="black")
    image.save(path)


def main(game, converter, output, vehicle="man.tgx_2020"):
    name, brand, dlc, model_folder, cabins = VEHICLES[vehicle]
    game, converter, output = Path(game).resolve(), Path(converter).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    extracted = output / "extracted"
    source = output / "vehicle"
    templates = source / "templates"
    controls = output / "controls"
    if (source / "vehicle.json").exists():
        raise ValueError("Use a fresh output directory for another game extraction")
    templates.mkdir(parents=True, exist_ok=True)
    controls.mkdir(exist_ok=True)
    archives = ["base.scs", "base_share.scs", "base_vehicle.scs", "def.scs", dlc, "version.scs"]
    command = [str(converter)]
    for archive in archives:
        if not (game / archive).is_file():
            raise FileNotFoundError(game / archive)
        command.extend(["-b", str(game / archive)])
    command.extend(["-e", str(extracted)])

    def convert(*args):
        result = subprocess.run(command + list(args), check=True, capture_output=True, text=True)
        if any(marker in result.stdout + result.stderr for marker in ["Unable to", "[error]", "[err]", "[warn]"]):
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout

    version = field(convert("--show-file", "/version.sii"), "version")
    if not version:
        raise ValueError("No game version")
    convert("--extract-directory", f"/def/vehicle/truck/{vehicle}")
    root = extracted / f"def/vehicle/truck/{vehicle}"
    accessory_ids = set()
    texture_sizes = collections.defaultdict(set)
    for file in sorted((root / "paint_job/accessory").glob("*.sii")):
        for block in re.findall(r"simple_paint_job_data[^{}]*\{([^}]*)\}", file.read_text(), re.S):
            ids = re.findall(r'acc_list\[\]:\s*"([^"]+)"', block)
            accessory_ids.update(ids)
            crop = re.search(r"_size_(\d+)x(\d+)", field(block, "paint_job_mask", ""))
            if crop:
                for identifier in ids:
                    texture_sizes[identifier].add(tuple(map(int, crop.groups())))
    definitions = []
    models = set()
    for file in sorted(root.rglob("*.sii")):
        if file.relative_to(root).parts[0] not in {"cabin", "chassis", "accessory"}:
            continue
        text = file.read_text()
        unit = re.search(r"accessory_\w+\s*:\s*(\S+)\s*\{", text)
        if not unit:
            continue
        unit_name, group = unit[1].split(f".{vehicle}.")
        paths = re.findall(r'^\s*(?:model|exterior_model|exterior_model_uk):\s*"([^"]+\.pmd)"', text, re.M)
        paths = sorted(set(p for p in paths if f"/{model_folder}/" in p))
        if paths:
            definitions.append({"id": f"{group}.{unit_name}", "group": group, "name": unit_name, "file": str(file.relative_to(root)), "text": text, "models": paths})
            models.update(paths)
    print(f"ETS2 {version}: {len(definitions)} definitions, {len(models)} exterior models", flush=True)
    for index, model in enumerate(sorted(models)):
        pim = extracted / model.lstrip("/")
        if not (pim.with_suffix(".pim").is_file() and pim.with_suffix(".pit").is_file()):
            convert("-m", model[:-4])
        if (index + 1) % 25 == 0:
            print(f"Converted {index + 1}/{len(models)} models", flush=True)

    painted = []
    for definition in definitions:
        meshes = []
        for model in definition["models"]:
            pim = extracted / model.lstrip("/")
            meshes.extend(paint_meshes(pim.with_suffix(".pim"), definition["text"]))
        if meshes:
            painted.append((definition, meshes))
    by_id = {d["id"]: (d, m) for d, m in painted}
    missing = accessory_ids - by_id.keys()
    if missing:
        raise ValueError(f"Paint overrides without a painted exterior model: {sorted(missing)}")

    main_parts = []
    accessory_parts = []
    evidence = []
    for cabin, cabin_name in cabins.items():
        contributors = []
        meshes = []
        for definition, mesh in painted:
            if definition["id"] in accessory_ids:
                continue
            if definition["group"] == "cabin" and definition["name"] != cabin:
                continue
            suitable_cabins = re.findall(r'suitable_for\[\]:\s*"([^".]+)\.' + re.escape(vehicle) + r'\.cabin"', definition["text"])
            if suitable_cabins and cabin not in suitable_cabins:
                continue
            contributors.append(definition["id"])
            meshes.extend(mesh)
        assert f"cabin.{cabin}" in contributors
        size = [4096, 4096]
        filename = f"templates/{cabin}.svg"
        edges = write_template(source / filename, meshes, size)
        main_parts.append({"id": cabin, "name": cabin_name, "game_ids": [cabin], "texture": {"size": size, "template": filename, "layout_version": 1}})
        evidence.append({"part": cabin, "contributors": contributors, "edges": edges})

    for identifier in sorted(accessory_ids):
        definition, meshes = by_id[identifier]
        sizes = texture_sizes[identifier]
        if len(sizes) != 1:
            raise ValueError(f"Ambiguous/missing texture size for {identifier}: {sizes}")
        size = list(next(iter(sizes)))
        part_id = identifier.replace(".", "_")
        filename = f"templates/{part_id}.svg"
        edges = write_template(source / filename, meshes, size)
        accessory_parts.append({"id": part_id, "name": identifier, "game_ids": [identifier], "texture": {"size": size, "template": filename, "layout_version": 1}})
        evidence.append({"part": part_id, "contributors": [identifier], "models": definition["models"], "edges": edges})

    manifest = {"format": 1, "id": f"experimental.game.{vehicle}", "version": "0.1.0", "name": f"{name} — Game extraction probe", "brand": brand, "kind": "truck", "description": f"Experimental UV templates generated locally from ETS2 {version}. Not yet validated in game.", "game": {"id": "ets2", "versions": f">={'.'.join(version.split('.')[:2])}, <1.{int(version.split('.')[1]) + 1}", "path": vehicle}, "paint_job": {"main": main_parts, "accessories": accessory_parts}}
    (source / "vehicle.json").write_text(json.dumps(manifest, indent=2) + "\n")
    for index, part in enumerate(main_parts + accessory_parts):
        control_image(controls / f"{part['id']}.png", part["texture"]["size"], index)
    report = {"vehicle": vehicle, "game_version": version, "archives": archives, "models_converted": len(models), "painted_definitions": len(painted), "main": len(main_parts), "accessories": len(accessory_parts), "evidence": evidence, "limits": ["Regular truckpaint, exterior models only", "Variants selected from definitions; additional DLC/mod overrides not mounted", "Main templates union all compatible painted options, not one configured truck", "4096-square main texture is an output choice; accessory sizes come from game paintjob crop names", "No in-game validation yet"]}
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"Generated {len(main_parts)} cabin templates and {len(accessory_parts)} accessory templates in {source}", flush=True)


if __name__ == "__main__":
    if len(sys.argv) not in (4, 5):
        raise SystemExit(__doc__)
    main(*sys.argv[1:])
