"""Assemble a bounded decision from actual, complete original/fresh receipts."""
import argparse,hashlib,json,pathlib,platform,subprocess
import accounting
ROOT=pathlib.Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def content_sha(p):
 b=p.read_bytes();b=b if p.suffix.lower()==".png"else b.replace(b"\r\n",b"\n");return hashlib.sha256(b).hexdigest()
def read(p):return json.loads(p.read_text())
def main():
 p=argparse.ArgumentParser();p.add_argument("--artifacts",type=pathlib.Path,required=True);a=p.parse_args();base=a.artifacts.resolve();debug=base/"debug";release=base/"release-1";fresh=base/"fresh-replay/reports"
 evidence=[]
 for d,names in [(debug,["core.json","branches.json","performance.json"]),(release,["core.json","branches.json","performance.json"]),(base/"release-2",["performance.json"]),(base/"release-3",["performance.json"]),(fresh,["core.json","branches.json","performance.json"])]:
  for n in names:
   f=d/n;assert f.is_file(),f;evidence.append(dict(path=str(f),sha256=sha(f),kind=n.removesuffix(".json")))
 for folder in ["repair-r2-intents","fresh-r2-intents/reports"]:
  f=base/folder/"intents.json";assert f.is_file();evidence.append(dict(path=str(f),sha256=sha(f),kind="intents"))
 ledger=base/"callback-ledger.json";ledger.write_text(json.dumps(accounting.build(evidence),indent=2))
 core=read(release/"core.json");assert len(core["rows"])==2
 candidates={}
 for r in core["rows"]:
  assert r["trace"][-1]["frame"]["error"] and "QueryBudget"in r["trace"][-1]["frame"]["error"]
  candidates[r["candidate"]]=dict(source_sha256=r["source_sha256"],source_content_sha256=content_sha(ROOT/f"fixtures/candidate-{r['candidate']}.at"),first_fault_tick=r["trace"][-1]["logical_tick"],published_tick=r["trace"][-1]["frame"]["tick"])
 metrics=[]
 for d in [debug,release,base/"release-2",base/"release-3"]:
  perf=read(d/"performance.json");times=[]
  for r in perf["rows"]:
   v=r.get("host_whole_tick_us",[]);times.extend(v if isinstance(v,list)else[v])
  metrics.append(dict(window=d.name,profile=perf["profile"],count=len(times),max_host_whole_tick_us=max(times),mean_host_whole_tick_us=sum(times)/len(times)))
 worst=max(r["max_host_whole_tick_us"]for r in metrics if r["profile"]=="release")
 inputs=subprocess.check_output(["git","ls-files","-z","."],cwd=ROOT).decode().split("\0");inputs={p:content_sha(ROOT/p)for p in inputs if p}
 d=dict(callback_ledger=dict(path=str(ledger),sha256=sha(ledger)),source_inputs=inputs,schema="ui011-decision/1",engine_commit="0261ae9dacf786beb6dceb630be6af267f242625",research_commit=subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip(),machine=dict(platform=platform.platform(),processor=platform.processor()),decision="no_go_current_profile",core_semantics="not_established_budget_fault",hard_budgets="fail",target_8ms="missed"if worst>8000 else"met_for_measured_prefix_only",g2_09_public_services="missing_public_capability",full_m2_proven=False,research_limit="two_candidates_current_profile_only",denominator={f"G2-{n:02d}":"not_established_budget_fault"if n<9 else"missing_public_capability"for n in range(1,10)},candidates=candidates,performance_windows=metrics,observability=dict(credits="32/33-calibrated source prices; not direct private meter",steps="actual maxima not exposed by public Play; frozen cumulative bound100000 enforced",resumes="actual maxima not exposed by public Play; frozen120/250ms enforced",gpu="CPU Host pipeline and separate synchronous readback; no GPU timestamps",successful_full_route=False),evidence=evidence)
 (base/"decision.json").write_text(json.dumps(d,indent=2));print(json.dumps({k:d[k]for k in ["decision","target_8ms","candidates","performance_windows"]},indent=2))
if __name__=="__main__":main()
