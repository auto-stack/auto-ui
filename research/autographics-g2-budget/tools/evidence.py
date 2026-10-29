"""Independent evidence reconstruction; no gameplay or consumer mutation."""
import hashlib,json,pathlib,re,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[1]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def content_sha(path):
 b=path.read_bytes();return hashlib.sha256(b if path.suffix.lower()==".png"else b.replace(b"\r\n",b"\n")).hexdigest()
def source_paths():
 return {p for p in subprocess.check_output(["git","ls-files","-z","."],cwd=ROOT).decode().split("\0")if p}
def source_manifest():return {p:content_sha(ROOT/p)for p in sorted(source_paths())}
def verify_source_revision(d):
 assert set(d["source_inputs"])==source_paths(),"incomplete source manifest"
 assert d["source_inputs"]==source_manifest(),"current source mismatch"
 commit=d["research_commit"];assert re.fullmatch(r"[0-9a-f]{40}",commit),"invalid research commit"
 prefix=ROOT.relative_to(pathlib.Path(subprocess.check_output(["git","rev-parse","--show-toplevel"],cwd=ROOT,text=True).strip())).as_posix()
 for rel,digest in d["source_inputs"].items():
  result=subprocess.run(["git","show",f"{commit}:{prefix}/{rel}"],cwd=ROOT,capture_output=True)
  assert result.returncode==0,"research commit source missing"
  b=result.stdout;b=b if pathlib.Path(rel).suffix.lower()==".png"else b.replace(b"\r\n",b"\n")
  assert hashlib.sha256(b).hexdigest()==digest,"research commit content mismatch"
def performance_windows(evidence):
 rows=[];raw=[json.loads(pathlib.Path(e["path"]).read_text())for e in evidence if e["kind"]=="performance"]
 assert len(raw)==5
 for name,profile,p in zip(["debug","release-1","release-2","release-3"],["debug","release","release","release"],raw[:4]):
  assert p["profile"]==profile
  times=[]
  for r in p["rows"]:
   v=r["host_whole_tick_us"];times.extend(v if isinstance(v,list)else[v])
  assert len(times)==62 and all(type(t)is int and t>=0 for t in times)
  rows.append(dict(window=name,profile=profile,count=len(times),max_host_whole_tick_us=max(times),mean_host_whole_tick_us=sum(times)/len(times)))
 return rows
def target_status(windows):
 return "missed"if max(w["max_host_whole_tick_us"]for w in windows if w["profile"]=="release")>8000 else"met_for_measured_prefix_only"
