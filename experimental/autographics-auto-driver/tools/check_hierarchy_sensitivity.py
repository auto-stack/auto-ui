"""Bounded, sequential actual-source controls for the hierarchy lifetime test.
Only run in PLAN-837's idle authoritative tree. Sources always restored byte-exact.
"""
import hashlib,json,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
assert subprocess.check_output(['git','branch','--show-current'],cwd=root,text=True).strip()=='plan-837-dev'
dest=root/'research/hierarchy-sensitivity';dest.mkdir(exist_ok=True)
session=root/'src/authoring/session.rs';host=root/'src/authoring/host.rs'
original={p:p.read_bytes() for p in [session,host]}
command=['cargo','test','--locked','--test','authoring_hierarchy','twenty_hierarchy_rounds_bound_vm_history_mesh_and_last_owner','--','--exact','--nocapture','--test-threads=1']
results=[]
try:
 for case,p,old,new,oracle in [
  ('retained-vm',session,'self.preview = p.preview.unwrap();','let old = std::mem::replace(&mut self.preview, p.preview.unwrap()); std::mem::forget(old);','assertion failed: !owner()'),
  ('fresh-mesh',host,'fn publish(&mut self, p: Prepared) {','fn publish(&mut self, p: Prepared) {\n let mut forced_warm=MeshStore::default(); for model in &self.models {for i in 0..model.meshes.len() {forced_warm.get_or_upload(self.rt.gpu(),model,i).unwrap();}}','Arc::ptr_eq'),
 ]:
  for path,data in original.items():path.write_bytes(data)
  text=p.read_text();assert text.count(old)==1,(case,'mutation target')
  text=text.replace(old,new)
  if case=='fresh-mesh':
   old2='c.instances = p.instances;';assert text.count(old2)==1
   text=text.replace(old2,'c.store=forced_warm; '+old2)
  p.write_text(text)
  out=subprocess.run(command,cwd=root,capture_output=True,timeout=120)
  log=out.stdout+out.stderr;(dest/(case+'.log')).write_bytes(log)
  assert out.returncode!=0 and b'twenty_hierarchy_rounds_bound_vm_history_mesh_and_last_owner ... FAILED' in log and oracle.encode() in log,(case,log[-4000:])
  results.append(dict(case=case,exit=out.returncode,oracle=oracle))
finally:
 for path,data in original.items():path.write_bytes(data)
for path,data in original.items():assert path.read_bytes()==data
out=subprocess.run(command,cwd=root,capture_output=True,timeout=120)
(dest/'restored.log').write_bytes(out.stdout+out.stderr);assert out.returncode==0
results.append(dict(case='restored',exit=out.returncode))
(dest/'results.json').write_text(json.dumps(dict(command=command,results=results,restored_hashes={str(p.relative_to(root)):hashlib.sha256(b).hexdigest() for p,b in original.items()}),indent=2)+'\n')
print(json.dumps(results))
