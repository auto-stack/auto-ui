"""Original MIT-licensed PLAN-838 triangles, no external input."""
import argparse,json,pathlib,struct,hashlib
blob=bytearray();views=[];accessors=[];meshes=[];nodes=[]
for i,x in enumerate([-1.5,1.5]):
    positions=[(-.7,0,0),(.7,0,0),(0,1.5,0)];normals=[(0,0,1)]*3;ids=[]
    for data,kind,component,count in [
        (b''.join(struct.pack('<3f',*v)for v in positions),'VEC3',5126,3),
        (b''.join(struct.pack('<3f',*v)for v in normals),'VEC3',5126,3),
        (struct.pack('<3H',0,1,2),'SCALAR',5123,3)]:
        while len(blob)%4:blob.append(0)
        vi=len(views);views.append({'buffer':0,'byteOffset':len(blob),'byteLength':len(data)});blob.extend(data)
        a={'bufferView':vi,'componentType':component,'count':count,'type':kind}
        if kind=='VEC3'and not ids:a.update(min=[-.7,0,0],max=[.7,1.5,0])
        ids.append(len(accessors));accessors.append(a)
    meshes.append({'name':'same-name','primitives':[{'attributes':{'POSITION':ids[0],'NORMAL':ids[1]},'indices':ids[2],'material':i}]})
    nodes.append({'mesh':i,'translation':[x,0,0]})
while len(blob)%4:blob.append(0)
j={'asset':{'version':'2.0'},'scene':0,'scenes':[{'nodes':[0,1]}],'nodes':nodes,'meshes':meshes,
   'materials':[{'pbrMetallicRoughness':{'baseColorFactor':c,'metallicFactor':0,'roughnessFactor':1}}for c in [[1,0,0,1],[0,1,0,1]]],
   'accessors':accessors,'bufferViews':views,'buffers':[{'byteLength':len(blob)}]}
b=json.dumps(j,separators=(',',':')).encode();b+=b' '*((-len(b))%4)
data=struct.pack('<III',0x46546c67,2,28+len(b)+len(blob))+struct.pack('<II',len(b),0x4e4f534a)+b+struct.pack('<II',len(blob),0x004e4942)+blob
p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');a=p.parse_args();path=pathlib.Path(__file__).with_name('two-parts.glb')
if a.check:assert path.read_bytes()==data,'fixture regeneration drift'
else:path.write_bytes(data)
print(hashlib.sha256(data).hexdigest())
