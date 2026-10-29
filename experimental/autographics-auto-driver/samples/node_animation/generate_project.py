"""Deterministic project from unchanged character/PNG; no animation stripping."""
import hashlib
import json
import pathlib
import struct
import sys

ROOT = pathlib.Path(__file__).resolve().parent
SOURCE = ROOT.parent / "textured_models/models"
MODEL = "7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1"
IMAGE = "106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3"

def sha(b): return hashlib.sha256(b).hexdigest()
def compact(v): return json.dumps(v, separators=(",", ":")).encode()

def bounds(raw):
    n = struct.unpack_from("<I", raw, 12)[0]
    j = json.loads(raw[20:20+n]); binary = raw[28+n:]
    points = []
    def visit(index, chain):
        node = j["nodes"][index]; chain = [node] + chain
        if "mesh" in node:
            for primitive in j["meshes"][node["mesh"]]["primitives"]:
                a = j["accessors"][primitive["attributes"]["POSITION"]]
                v = j["bufferViews"][a["bufferView"]]
                start = v.get("byteOffset",0) + a.get("byteOffset",0)
                for i in range(a["count"]):
                    p = struct.unpack_from("<3f",binary,start+i*v.get("byteStride",12))
                    for t in chain:
                        p = [p[k]*t.get("scale",[1,1,1])[k] for k in range(3)]
                        x,y,z,w = t.get("rotation",[0,0,0,1]); px,py,pz=p
                        tx,ty,tz = 2*(y*pz-z*py),2*(z*px-x*pz),2*(x*py-y*px)
                        p = [px+w*tx+y*tz-z*ty,py+w*ty+z*tx-x*tz,pz+w*tz+x*ty-y*tx]
                        p = [p[k]+t.get("translation",[0,0,0])[k] for k in range(3)]
                    points.append(p)
        for child in node.get("children",[]): visit(child,chain)
    for root in j["scenes"][j.get("scene",0)]["nodes"]: visit(root,[])
    return [min(p[k] for p in points) for k in range(3)], [max(p[k] for p in points) for k in range(3)]

def project():
    raw=(SOURCE/"character.glb").read_bytes(); image=(SOURCE/"Textures/colormap.png").read_bytes()
    assert sha(raw)==MODEL and sha(image)==IMAGE
    edge=dict(source="uri:Textures/colormap.png",sha256=IMAGE,byte_length=len(image),width=512,height=512)
    closure=sha(compact(["autographics.node_animation3d/1",MODEL,len(raw),[edge]]))
    identity=dict(closure=dict(node_animation=True,id=closure,images=[edge]),format="glb",sha256=MODEL,byte_length=len(raw),name="character.glb")
    lo,hi=bounds(raw); scale=[v/(hi[k]-lo[k]) for k,v in enumerate([.8,1.,.8])]
    def obj(i,name,kind,pos,scale,resource=None):
        return dict(id=i,name=name,kind=kind,translation=pos,scale=scale,yaw=0,pitch=0,rgba=[.9,.3,.2,1.],visible=True,parent=None,resource=resource,sprite=None)
    objects=[obj(1,"Original animated character","model",[0,-lo[1]*scale[1]+.002,0],scale,"closure:"+closure),obj(2,"Frozen collision floor","cylinder",[0,-.5,0],[10,.5,10])]
    code=(ROOT/"main.at").read_bytes().replace(b"\r\n",b"\n"); code_sha=sha(code)
    binding=dict(api="autographics.node_animation3d/1",source=dict(format="auto-source",sha256=code_sha,byte_length=len(code)),actors=dict(two_d=None,three_d=1))
    doc=dict(schema="autographics.scene/6",document_id="plan006-node-animation",resources=["circle","quad","cylinder","player","mob"],objects=objects,groups=[],assets=[identity],behavior=binding)
    return {f"assets/{MODEL}.glb":raw,f"assets/{IMAGE}.png":image,f"scripts/{code_sha}.at":code,"scene.agscene":(json.dumps(doc,indent=2)+"\n").encode()}

if __name__ == "__main__":
    check="--check" in sys.argv
    for name,data in project().items():
        p=ROOT/name
        if check:
            assert p.read_bytes().replace(b"\r\n",b"\n")==data if p.suffix == ".agscene" else p.read_bytes()==data,name
        else:
            p.parent.mkdir(parents=True,exist_ok=True)
            if p.exists(): assert p.read_bytes()==data,"refuse different existing output: "+name
            else: p.write_bytes(data)
    print("PASS PLAN-006 public bound scene: unchanged original four clips + PNG + Auto, schema6")
