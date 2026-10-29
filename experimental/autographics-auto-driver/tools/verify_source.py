"""Check locked metadata's actual SDK dependency closure and pinned local inputs."""
import json,pathlib,subprocess,sys,tomllib
root=pathlib.Path(__file__).resolve().parents[1]
current012=(root/'research/plan012-identity.json').exists()or(root/'tools/verify_plan012.py').exists()
if current012 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan012.py')],check=True)
current006=(root/'research/plan006-identity.json').exists()or(root/'tools/verify_plan006.py').exists()
if current006 and not current012 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan006.py')],check=True)
current005=(root/'research/plan005-identity.json').exists()or(root/'tools/verify_plan005.py').exists()
if current005 and not current006 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan005.py')],check=True)
current004=(root/'research/plan004-identity.json').exists()or(root/'tools/verify_plan004.py').exists()
if current004 and not current005 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan004.py')],check=True)
current002=(root/'research/plan002-identity.json').exists()or(root/'tools/verify_plan002.py').exists()
if current002 and not current004 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan002.py')],check=True)
current001=(root/'research/plan001-identity.json').exists()or(root/'tools/verify_plan001.py').exists()
if current001 and not current002 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan001.py')],check=True)
current879=(root/'research/plan879-identity.json').exists()or(root/'tools/verify_plan879.py').exists()
if current879 and not current001 and not current002 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan879.py')],check=True)
current875=(root/'research/plan875-identity.json').exists()or(root/'tools/verify_plan875.py').exists()
if current875 and not current879 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan875.py')],check=True)
current874=(root/'research/plan874-identity.json').exists()or(root/'tools/verify_plan874.py').exists()
if current874 and not current875 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan874.py')],check=True)
current861=(root/'research/plan861-identity.json').exists()or(root/'tools/verify_plan861.py').exists()
if current861 and not current874 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan861.py')],check=True)
current860=(root/'research/plan860-identity.json').exists()or(root/'tools/verify_plan860.py').exists()
if current860 and not current861 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan860.py')],check=True)
current857=(root/'research/plan857-identity.json').exists()or(root/'tools/verify_plan857.py').exists()
if current857 and not current860 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan857.py')],check=True)
current856=(root/'research/plan856-identity.json').exists()or(root/'tools/verify_plan856.py').exists()
if current856 and not current857 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan856.py')],check=True)
current855=(root/'research/plan855-identity.json').exists()or(root/'tools/verify_plan855.py').exists()
if current855 and not current856 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan855.py')],check=True)
current854=(root/'research/plan854-identity.json').exists()or(root/'tools/verify_plan854.py').exists()
if current854 and not current855 and not current856 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan854.py')],check=True)
current853=(root/'research/plan853-identity.json').exists()or(root/'tools/verify_plan853.py').exists()
# Reject undeclared build/config inputs before Cargo inspects this workspace.
if current853 and not current854 and not current855 and not current856 and '--sdk-only' not in sys.argv:
 subprocess.run([sys.executable,str(root/'tools/verify_plan853.py')],check=True)
subprocess.run([sys.executable,str(root/'tools/prepare_sdk.py'),'--verify-only'],check=True)
m=json.loads(subprocess.check_output(['cargo','metadata','--manifest-path',str(root/'Cargo.toml'),'--locked','--offline','--format-version','1'],text=True))
assert pathlib.Path(m['target_directory']).resolve()==(root/'target').resolve(),'consumer target escapes its workspace'
packages={p['id']:p for p in m['packages']};nodes={n['id']:n for n in m['resolve']['nodes']}
start=next(k for k,p in packages.items() if p['name']=='auto-lang-core');todo=[start];seen=set()
forbidden={'auto-lang','auto-man','auto-ui-dialect','auto-ui-compiler','auto-ui-runtime','auto-ui-tools'}
while todo:
 k=todo.pop()
 if k in seen:continue
 seen.add(k);p=packages[k];assert p['name']not in forbidden,p['name']
 if p['source'] is None:assert (root/'.sdk-source').resolve() in pathlib.Path(p['manifest_path']).resolve().parents,p['manifest_path']
 todo.extend(e['pkg']for e in nodes[k]['deps'])
b=json.loads((root/'research/baseline.json.txt').read_text());paths={p['path']for p in b['sdk']['path_sources']};edges=[]
for relative in paths:
 manifest=root/'.sdk-source'/relative/'Cargo.toml';data=tomllib.loads(manifest.read_text())
 assert data['package']['name']not in forbidden
 sections=[data]+list(data.get('target',{}).values())
 for section in sections:
  for kind in ['dependencies','build-dependencies','dev-dependencies']:
   for name,dep in section.get(kind,{}).items():
    if isinstance(dep,dict)and'path'in dep:
     dest=(manifest.parent/dep['path']).resolve().relative_to((root/'.sdk-source').resolve()).as_posix();assert dest in paths,(relative,name,dest);edges.append((relative,kind,name,dest))
assert len(paths)==17 and len(edges)==29,(len(paths),len(edges))
# AGP registry package identities must all survive the consumer lock.
a=tomllib.loads((root.parent/'autographics-prototype/Cargo.lock').read_text());c=tomllib.loads((root/'Cargo.lock').read_text());ids=lambda x:{(p['name'],p['version'],p.get('checksum'))for p in x['package']if'source'in p}
assert ids(a)<=ids(c),'AGP frozen registry dependency changed'
print(f'PASS SDK metadata closure: {len(seen)} packages; 17 declared local packages, 29 path edges; forbidden=0; frozen AGP lock retained')

if '--sdk-only' in sys.argv:
 sys.exit(0)

if current012 or current006 or current005 or current004:
 pass # current complete closure already checked
elif current860 or current857:
 pass # current complete closure already checked
elif current856:
 pass # historical closure is retained but superseded
elif current855:
 pass # current complete closure already checked
elif current854:
 pass # already admitted before metadata
elif (root/'research/plan853-identity.json').exists():
 pass # already admitted before cargo metadata
elif (root/'research/plan851-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan851.py')],check=True)
elif (root/'research/plan849-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan849.py')],check=True)
elif (root/'research/plan848-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan848.py')],check=True)
elif (root/'research/plan846-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan846.py')],check=True)
elif (root/'research/plan845-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan845.py')],check=True)
elif (root/'research/plan840-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan840.py')],check=True)
elif (root/'research/plan839-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan839.py')],check=True)
elif (root/'research/plan838-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan838.py')],check=True)
elif (root/'research/plan837-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan837.py')],check=True)
elif (root/'research/plan836-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan836.py')],check=True)
elif (root/'research/plan835-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan835.py')],check=True)
elif (root/'research/plan834-identity.json').exists():
 subprocess.run([sys.executable,str(root/'tools/verify_plan834.py')],check=True)
