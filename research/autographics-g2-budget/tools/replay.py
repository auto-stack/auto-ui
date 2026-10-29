"""Materialize the same research harness against an independently exported consumer.
Copy source bytes and lock identities; only the generated dependency path changes.
"""
import argparse,os,pathlib,re,shutil,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[1]
def main():
 p=argparse.ArgumentParser();p.add_argument("--consumer-root",required=True,type=pathlib.Path);p.add_argument("--out",required=True,type=pathlib.Path);p.add_argument("--target",required=True,type=pathlib.Path);p.add_argument("--release",action="store_true");a=p.parse_args();check_env=os.environ.copy();check_env.pop("CARGO_TARGET_DIR",None);subprocess.run(["python",str(a.consumer_root.resolve()/"autographics-auto-driver/tools/verify_source.py")],env=check_env,check=True);out=a.out.resolve();assert not out.exists(),"new replay directory required";ws=out/"ws";ws.mkdir(parents=True)
 for f in ["Cargo.lock","Cargo.toml"]:shutil.copyfile(ROOT/f,ws/f)
 for folder in ["tests","fixtures"]:shutil.copytree(ROOT/folder,ws/folder)
 c=ws/"Cargo.toml";s=c.read_text();s=re.sub(r'path = "[^"]+"',lambda _: 'path = "'+(a.consumer_root.resolve()/"autographics-auto-driver").as_posix()+'"',s);c.write_text(s)
 env=os.environ.copy();env.update(CARGO_TARGET_DIR=str(a.target.resolve()),G2_ARTIFACTS=str(out/"reports"),RUST_MIN_STACK="16777216")
 command=["cargo","test","--manifest-path",str(c),"--locked","--offline","--test","full_loop"]+(["--release"]if a.release else[])+["--","--test-threads=1"]
 with (out/"replay.log").open("w") as log:subprocess.run(command,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 print("PASS fresh consumer replay",out)
if __name__=="__main__":main()
