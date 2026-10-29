"""Build/check a relocatable public scene from unchanged fixed Kenney bytes."""
import hashlib
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent
PIN = "3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0"
MODELS = {
    "platform": "78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c",
    "coin": "a3fb8f779a6af1cd75e5a02da77a87546ca8a59f6fe52f79f25179c4a68fd0e5",
    "flag": "0ba84ff43f0fea0a9f0b080cfa1be54ebda36c8c35174d1ac2553ae6da9f65ae",
}
IMAGE = "106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3"


def sha(b):
    return hashlib.sha256(b).hexdigest()


def compact(v):
    return json.dumps(v, separators=(",", ":"), ensure_ascii=False).encode()


def project():
    image = (ROOT / "models/Textures/colormap.png").read_bytes()
    assert sha(image) == IMAGE
    edge = dict(source="uri:Textures/colormap.png", sha256=IMAGE,
                byte_length=len(image), width=512, height=512)
    blobs = {f"assets/{IMAGE}.png": image}
    assets = []
    for name, expected in MODELS.items():
        b = (ROOT / f"models/{name}.glb").read_bytes()
        assert sha(b) == expected, f"fixed source changed: {name} @ {PIN}"
        closure = sha(compact(["autographics.textured/1", expected, len(b), [edge]]))
        assets.append(dict(closure=dict(id=closure, images=[edge]), format="glb",
                           sha256=expected, byte_length=len(b), name=f"{name}.glb"))
        blobs[f"assets/{expected}.glb"] = b
    def obj(i, name, kind, translation, scale=(1, 1, 1), resource=None):
        return dict(id=i, name=name, kind=kind, translation=translation,
                    scale=scale, yaw=0, pitch=0, rgba=[0.9, 0.3, 0.2, 1.0],
                    visible=True, parent=None, resource=resource, sprite=None)
    objects = [obj(1, "Character", "cylinder", [0, 0.5, 0], [0.4, 0.5, 0.4]),
               obj(2, "Ground", "cylinder", [0, -0.5, 0], [10, 0.5, 10])]
    for i, (name, pos) in enumerate(zip(MODELS, [[3, 1, -3], [-3, 1.5, -5], [0, 1, -6]])):
        objects.append(obj(i + 3, name.title(), "model", pos,
                           resource="closure:" + assets[i]["closure"]["id"]))
    code=(ROOT / "main.at").read_bytes().replace(b"\r\n",b"\n")
    code_sha=sha(code)
    blobs[f"scripts/{code_sha}.at"]=code
    binding=dict(api="autographics.objects3d/1",source=dict(format="auto-source",sha256=code_sha,byte_length=len(code)),actors=dict(two_d=None,three_d=1))
    doc = dict(schema="autographics.scene/6", document_id="plan005-textured-models",
               resources=["circle", "quad", "cylinder", "player", "mob"],
               objects=objects, groups=[], assets=assets, behavior=binding)
    blobs["scene.agscene"] = (json.dumps(doc, indent=2) + "\n").encode()
    return blobs


if __name__ == "__main__":
    check = "--check" in sys.argv
    for relative, data in project().items():
        p = ROOT / relative
        if check:
            assert p.read_bytes().replace(b"\r\n", b"\n") == data if p.suffix == ".agscene" else p.read_bytes() == data, relative
        else:
            p.parent.mkdir(parents=True, exist_ok=True)
            if p.exists():
                assert p.read_bytes() == data, f"refuse to overwrite different bytes: {relative}"
            else:
                p.write_bytes(data)
    print("PASS PLAN-005 public project: 3 unchanged GLBs + 1 shared PNG + bound Auto blob + scene/6")
