"""Export the exact PLAN004+008 source identity from local Git into a new directory.
No existing consumer is overwritten and no linked directories are created.
"""
import argparse,hashlib,io,json,pathlib,subprocess,sys,tarfile
PIN="0261ae9dacf786beb6dceb630be6af267f242625"
def main():
 p=argparse.ArgumentParser();p.add_argument("--repository",type=pathlib.Path,required=True);p.add_argument("--out",type=pathlib.Path,required=True);a=p.parse_args();dest=a.out.resolve();assert not dest.exists(),"new directory required"
 manifest="experimental/autographics-auto-driver/research/plan004-identity.json"
 raw=subprocess.check_output(["git","-C",str(a.repository),"show",f"{PIN}:{manifest}"]);identity=json.loads(raw);required={"experimental/"+s for s in identity["files"]}|{manifest}
 archive=subprocess.check_output(["git","-C",str(a.repository),"archive",PIN,"experimental/autographics-auto-driver","experimental/autographics-prototype"])
 with tarfile.open(fileobj=io.BytesIO(archive)) as tf:
  seen=set()
  for m in tf:
   if m.name not in required:continue
   assert m.isfile() and not m.issym() and not m.islnk()
   target=dest/m.name;assert dest in target.resolve().parents;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(tf.extractfile(m).read());seen.add(m.name)
 assert seen==required,("Git export missing identity files",required-seen)
 driver=dest/"experimental/autographics-auto-driver"
 for tool,args in [("verify_plan004.py",[]),("prepare_sdk.py",["--from-snapshot"]),("verify_source.py",[])]:
  subprocess.run([sys.executable,str(driver/"tools"/tool),*args],check=True)
 print(f"PASS fixed consumer exported from {PIN}: {len(identity['files'])} identity entries")
if __name__=="__main__":main()
