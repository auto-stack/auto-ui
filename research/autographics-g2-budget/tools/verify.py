"""Validate UI-011 measured research receipts and deliberate invalid reports.
The program exits nonzero for incomplete evidence. It never fabricates a core pass.
"""
import argparse,copy,hashlib,json,pathlib,subprocess,sys
import evidence as bound
import accounting
ROOT=pathlib.Path(__file__).resolve().parents[1]
ENGINE=pathlib.Path("D:/autostack/plan-artifacts/auto-ui/plan-011/consumer/experimental")
DEFAULT=pathlib.Path("D:/autostack/plan-artifacts/auto-ui/plan-011")
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def content_sha(p):
 b=p.read_bytes();b=b if p.suffix.lower()==".png"else b.replace(b"\r\n",b"\n");return hashlib.sha256(b).hexdigest()
def receipts(d):
 by_kind={k:[]for k in ["core","branches","performance","intents"]}
 for e in d["evidence"]:
  f=pathlib.Path(e["path"]);assert f.is_file() and sha(f)==e["sha256"]
  raw=json.loads(f.read_text())
  if e["kind"]=="core":
   for row in raw["rows"]:
    source=f.parent/"sources"/f"{row['source_sha256']}.at"
    assert sha(source)==row["source_sha256"]
    assert content_sha(source)==content_sha(ROOT/f"fixtures/candidate-{row['candidate']}.at")
  by_kind[e["kind"]].append(raw)
 assert len(by_kind["intents"])==2
 expected={"jump-initial":[0,0,0],"jump-accepted":[1,0,0],"air-rejump-no-intent":[1,0,0],"single-N":[0,0,0],"single-Nplus1":[0,1,1],"single-no-duplicate":[0,1,1],"triple-N":[0,0,0],"triple-goal-Nplus1":[0,3,4],"ended-no-repeat":[0,3,4],"enter-reset-intents":[0,0,0],"duplicate-coin-intent-control":[0,6,4],"dropped-coin-intent-control":[0,0,4],"intent-source-restored":[0,3,4]}
 for obs in by_kind["intents"]:
  assert obs["schema"]=="ui011-intents/1" and obs["full_core_intents"]=="not_established_budget_fault"
  assert len(obs["rows"])==len(expected)
  assert {r["case"]:r["intent_totals"]for r in obs["rows"]}==expected
  assert all(r["mutant_detected"]is True for r in obs["rows"]if r["case"].endswith("-control"))
 assert len(by_kind["core"])==3 and len(by_kind["branches"])==3 and len(by_kind["performance"])==5
 for core in by_kind["core"]:
  assert core["schema"]=="ui011-core/1" and {r["candidate"]for r in core["rows"]}=={"a","b"}
  for r in core["rows"]:
   assert r["source_sha256"]==d["candidates"][r["candidate"]]["source_sha256"]
   summary=d["candidates"][r["candidate"]]
   assert summary["first_fault_tick"]==r["trace"][-1]["logical_tick"]
   assert summary["published_tick"]==r["trace"][-1]["frame"]["tick"]
   assert len(r["trace"])==1 and r["trace"][0]["logical_tick"]==1
   assert r["trace"][0]["frame"]["tick"]==0 and "QueryBudget"in r["trace"][0]["frame"]["error"]
   assert r["initial_capture_sha256"]==r["fault_capture_sha256"]
 names={"boundary-32","boundary-33","b-cached-idle32","b-first-run-frame33","move-right-run","move-left-flip","jump-rise","fall","air-rejump-and-land","pause-neutral-step","three-coins-goal-Nplus1","triple-win-enter-fresh-run","dropped-third-event-control","restored-triple-control","full-a-three-coins-peak-budget-blocked","full-b-three-coins-peak-budget-blocked","isolated-three-coins-enemy-goal-priority","bad-sprite-last-good","old-run-handle-rejected","exact-source-restored","twenty-short-runs","image-owner-editor-external-final","isolated-patrol-moving-control","frozen-enemy-mutant-detected","patrol-east","patrol-west"}
 for n in ["wall","ceiling","edge","pit","single","upper-coin","goal-without-coins","enemy-contact"]:
  names.update({f"full-a-{n}",f"full-b-{n}",f"isolated-{n}"})
 stable=[]
 for branches in by_kind["branches"]:
  assert branches["schema"]=="ui011-branches/1"
  rows={r["case"]:r for r in branches["rows"]};assert set(rows)==names and len(rows)==len(branches["rows"])
  for name in ["boundary-32","b-cached-idle32","move-right-run","move-left-flip","jump-rise","fall","three-coins-goal-Nplus1","exact-source-restored"]:assert rows[name]["frame"]["error"] is None
  for name in ["boundary-33","b-first-run-frame33"]:assert "QueryBudget"in rows[name]["frame"]["error"]
  for name in ["move-right-run","move-left-flip","jump-rise","fall"]:assert rows[name]["pixels"]["wrong_frame_rejected"]is True and len(rows[name]["pixels"]["samples"])==5
  assert rows["three-coins-goal-Nplus1"]["frame"]["live_count"]==1
  assert rows["three-coins-goal-Nplus1"]["frame"]["diagnostic"][:2]==[3,1]
  assert rows["isolated-three-coins-enemy-goal-priority"]["frame"]["diagnostic"][:2]==[3,2]
  assert rows["dropped-third-event-control"]["frame"]["live_count"]==2
  assert rows["restored-triple-control"]["frame"]["live_count"]==1
  cycles=rows["twenty-short-runs"]["cycles"];assert len(cycles)==20 and all(c["owner_released"]is True for c in cycles)
  assert rows["image-owner-editor-external-final"]["last_owner_released"]is True
  stable.append({n:(r.get("frame",{}).get("live_count"),r.get("frame",{}).get("diagnostic"),bool(r.get("frame",{}).get("error")))for n,r in rows.items()})
 assert stable[0]==stable[1]==stable[2],"original/debug/release/fresh stable outcomes drift"
 for perf in by_kind["performance"]:
  assert perf["schema"]=="ui011-performance/1" and len(perf["rows"])==3
  assert len(perf["rows"][2]["host_whole_tick_us"])==60
  assert all("readback_assert_us"in r or r["case"]=="original-failing-prefix"for r in perf["rows"])

def validate(d):
 assert d["schema"]=="ui011-decision/1"
 bound.verify_source_revision(d)
 receipts(d)
 assert set(d["denominator"])=={f"G2-{n:02d}" for n in range(1,10)}
 assert set(d["candidates"])=={"a","b"}
 assert d["full_m2_proven"] is False
 assert d["g2_09_public_services"]=="missing_public_capability"
 assert d["core_semantics"]=="not_established_budget_fault"
 assert d["hard_budgets"]=="fail" and d["decision"]=="no_go_current_profile"
 assert d["performance_windows"]==bound.performance_windows(d["evidence"]),"performance summary contradicts raw"
 assert d["target_8ms"]==bound.target_status(d["performance_windows"]),"8ms classification contradicts raw"
 ledger=d["callback_ledger"];p=pathlib.Path(ledger["path"]);assert sha(p)==ledger["sha256"]
 accounting.validate_semantics(json.loads(p.read_text()),d["evidence"])
 assert json.loads(p.read_text())==accounting.build(d["evidence"]),"callback account contradicts source/raw"
 assert all(d["denominator"][f"G2-{n:02d}"]=="not_established_budget_fault" for n in range(1,9))
 assert d["denominator"]["G2-09"]=="missing_public_capability"
 for c in ["a","b"]:
  assert d["candidates"][c]["source_content_sha256"]==content_sha(ROOT/f"fixtures/candidate-{c}.at")
  assert d["candidates"][c]["first_fault_tick"]>=1
 assert d["engine_commit"]=="0261ae9dacf786beb6dceb630be6af267f242625"
 assert d["research_limit"]=="two_candidates_current_profile_only"
 assert len(d["evidence"])>=3
 for e in d["evidence"]:
  p=pathlib.Path(e["path"]);assert p.is_file() and sha(p)==e["sha256"],p

def main():
 p=argparse.ArgumentParser();p.add_argument("--controls",action="store_true");p.add_argument("--report",type=pathlib.Path,default=DEFAULT/"decision.json");p.add_argument("--consumer-root",type=pathlib.Path,default=ENGINE);a=p.parse_args()
 subprocess.run([sys.executable,str(a.consumer_root/"autographics-auto-driver/tools/verify_source.py")],check=True)
 d=json.loads(a.report.read_text());validate(d)
 n=0
 if a.controls:
  def rejected(m):
   nonlocal n
   try:validate(m)
   except (AssertionError,KeyError):n+=1;return
   raise AssertionError("invalid decision was accepted")
  m=copy.deepcopy(d);del m["denominator"]["G2-08"];rejected(m)
  for k,v in [("full_m2_proven",True),("decision","go_core"),("core_semantics","pass"),("g2_09_public_services","pass"),("engine_commit","wrong")]:
   m=copy.deepcopy(d);m[k]=v;rejected(m)
  m=copy.deepcopy(d);m["candidates"]["a"]["source_sha256"]="stale";rejected(m)
  m=copy.deepcopy(d);m["evidence"]=[];rejected(m)
 # Mutate actual bound report bytes, refresh their hash, then reject semantic holes.
  control_dir=a.report.parent/"decision-controls";control_dir.mkdir(exist_ok=True)
  for kind in ["branches","core"]:
   m=copy.deepcopy(d);entry=next(e for e in m["evidence"]if e["kind"]==kind)
   raw=json.loads(pathlib.Path(entry["path"]).read_text());raw["rows"].pop()
   path=pathlib.Path(entry["path"]).with_name(f"control-missing-{kind}-row.json");path.write_text(json.dumps(raw));entry.update(path=str(path),sha256=sha(path));rejected(m)
 if a.controls:
  for field,value in [("source_inputs",{}),("research_commit","0"*40),("performance_windows",[])]:
   m=copy.deepcopy(d);m[field]=value;rejected(m)
  m=copy.deepcopy(d);m["candidates"]["a"]["first_fault_tick"]=999;rejected(m)
  m=copy.deepcopy(d);m["candidates"]["a"]["published_tick"]=999;rejected(m)
  m=copy.deepcopy(d);m["performance_windows"][1]["max_host_whole_tick_us"]=999999999;rejected(m)
  m=copy.deepcopy(d);entry=[e for e in m["evidence"]if e["kind"]=="performance"][1]
  raw=json.loads(pathlib.Path(entry["path"]).read_text());raw["rows"][2]["host_whole_tick_us"][0]=9000000
  path=a.report.parent/"decision-controls/measured-9s-performance.json";path.write_text(json.dumps(raw));entry.update(path=str(path),sha256=sha(path))
  rejected(m)
  m["performance_windows"]=bound.performance_windows(m["evidence"]);m["target_8ms"]="met_for_measured_prefix_only";rejected(m)
 if a.controls:
  for mode in ["accepted-over-cap","shifted-air-tick"]:
   m=copy.deepcopy(d);raw=json.loads(pathlib.Path(m["callback_ledger"]["path"]).read_text())
   if mode=="accepted-over-cap":
    row=next(e for e in raw["entries"]if e["case"].endswith("-peak-sequence")and e["account"]["diagnostics"]=="accepted frame");row["account"]["source_required_credit_range"]=[47,47]
   else:
    row=next(e for e in raw["entries"]if e["case"]=="air-rejump-and-land");row["published_tick"]=1;row["run"]=None
   path=a.report.parent/"decision-controls"/f"{mode}.json";path.write_text(json.dumps(raw));m["callback_ledger"]={"path":str(path),"sha256":sha(path)};rejected(m)
 print(f"PASS honest bounded no-go decision; invalid reports rejected={n}")
if __name__=="__main__":main()
