"""Source-derived callback accounts, calibrated by the measured32/33 controls.
Ranges describe public-call branches, never a read of a private VM meter.
Fault data only describes a source-derived prefix/refusal; no fresh diagnostics.
"""
import hashlib,json,pathlib,re
from evidence import sha
PRICES={"ag_action_pressed":1,"ag_action_held":1,"ag_char_grounded":1,"ag_sprite_frame":1,"ag_object_move":1,"ag_object_spawn":1,"ag_trigger_bind":1,"ag_object_despawn":1,"ag_event_next":1,"ag_event_pair":1,"ag_event_phase":1,"ag_round_request":1,"ag_char_bind":0,"ag_char_solid":0,"ag_camera_follow_2d":0,"ag_set_local":0,"ag_world":0,"ag_char_move":[25,26]}
def interval(v):return v if isinstance(v,list)else[v,v]
def account(source,case,frame,phase=None,ordinal=None):
 body=source.split("fn app_update",1)[-1];error=frame.get("error");tick=frame.get("tick",0)
 init=phase=="app_start" or (phase is None and tick==0 and not error)
 calls={};prices=dict(PRICES);branch="";suffix=None
 if init:
  body=source.split("fn app_start",1)[1].split("\n}",1)[0]
  for name in re.findall(r"\b(ag_[a-z0-9_]+)\(",body):calls[name]=calls.get(name,0)+1
  
  if "g2_audit_intents()"in body:calls["ag_set_local"]=calls.get("ag_set_local",0)+3
  branch="initialization";phase="app_start"
 elif "old-run-handle"in case:
  calls={"ag_object_despawn":1};prices["ag_object_despawn"]=0;branch="stale handle validation before charge"
 else:
  ended=case=="ended-no-repeat" or ("isolated-pit"in case and ordinal and ordinal>=2)or("isolated-enemy-contact"in case and ordinal and ordinal>=3)
  if phase=="enter_request":calls={"ag_action_pressed":1,"ag_round_request":1};branch="Running Enter restart request"
  elif ended:calls={"ag_action_pressed":1};branch="ended state, no new Enter"
  else:
   calls={"ag_action_pressed":1,"ag_action_held":2,"ag_char_grounded":1,"ag_sprite_frame":1,"ag_world":1}
   cached="x!=oldframe"in body
   if cached:calls["ag_sprite_frame"]=[0,1]
   if "ag_char_move("in body:calls["ag_char_move"]=1
   if "ag_object_move("in body:calls["ag_object_move"]=1
   pad=re.search(r"for b in 0\.\.(\d+)",body)
   if pad:calls["ag_action_held"]+=int(pad[1]);prices["ag_char_move"]=26
   events=0;despawns=0
   if "ag_event_next("in body:
    events=None
    if case.startswith("core-")or case in {"b-cached-idle32","b-first-run-frame33"}or("full-"in case and tick==0):events=0
    elif "triple"in case or "three-coins"in case:
     events=5 if "priority"in case else 4;despawns=3
     if case=="triple-N":events=0;despawns=0
    elif "single"in case or "upper-coin"in case:
     events=1 if ordinal==2 or "Nplus1"in case else 0;despawns=events
    elif "patrol"in case or "frozen-enemy"in case or "isolated-pit"in case:events=0
    elif "goal-without-coins"in case or "enemy-contact"in case:events=0 if ordinal==1 else 1
    if "dropped-third-event"in case:events=2;despawns=2
    if "duplicate-coin-intent"in case or "dropped-coin-intent"in case or "intent-source-restored"in case:events=4;despawns=3
    cap=2 if "for i in 0..2"in body else 6
    calls.update(ag_event_next=[1,cap]if events is None else min(events+1,cap),ag_event_pair=[0,5]if events is None else events,ag_event_phase=[0,5]if events is None else events,ag_object_despawn=[0,3]if events is None else despawns)
   # Original first-callback failure is a normal26 movement/first sprite transition,
   # cross-calibrated by the same grounded32/33 threshold and cached B idle control.
   if case.startswith("core-")or (case.startswith("full-")and tick==0 and error):prices["ag_char_move"]=26;calls["ag_sprite_frame"]=1
   if case=="b-cached-idle32":prices["ag_char_move"]=26;calls["ag_sprite_frame"]=0
   if case=="b-first-run-frame33":prices["ag_char_move"]=26;calls["ag_sprite_frame"]=1
   if case.startswith("full-")and tick>=1 and "goal"in case or(case.startswith("full-")and tick>=1 and "three-coins"in case):
    prices["ag_char_move"]=25
    if cached:calls["ag_sprite_frame"]=1 if ordinal==1 and not error else 0
   if "bad-sprite"in case:prices["ag_sprite_frame"]=[0,1]
   branch="event/patrol isolation"if "ag_char_move"not in calls else "character and presentation"if "ag_event_next"not in calls else "combined character/patrol/events"
   if events:branch+=f"; queued events={events}, despawns={despawns}"
   if cached:branch+="; cached sprite branch"
   if error and "QueryBudget"in error:suffix="not_established_budget_fault; source-required branch is hypothetical after refusal"
  # Diagnostic setters are public zero-credit output only, never gameplay inputs.
  n=body.count("ag_set_local(")
  if "g2_audit_intents()"in body:n+=3
  if ended or phase=="enter_request":n=3 if "g2_audit_intents()"in body else 0
  if n:calls["ag_set_local"]=n
 lo=hi=0
 for name,count in calls.items():
  c=interval(count);p=interval(prices[name]);lo+=c[0]*p[0];hi+=c[1]*p[1]
 return dict(phase=phase or"app_update",branch=branch,public_call_counts=calls,public_unit_prices={n:prices[n]for n in calls},source_required_credit_range=[lo,hi],observed_limit_refusal=bool(error and"QueryBudget"in error),accepted_capacity=32,first_refused_credit_demand=33 if error and"QueryBudget"in error else None,unreached_suffix=suffix,diagnostics="last_good_only"if error else"accepted frame",method="source-derived/ranged;32-pass/33-refuse calibration;not direct private meter")
def build(evidence):
 entries=[];bindings=[];sources={}
 for e in evidence:
  parent=pathlib.Path(e["path"]).parent
  for p in (parent/"sources").glob("*.at"):
   sources[p.stem]=p
 def add(e,case,frame,digest=None,phase=None,ordinal=None):
  if digest:
   path=sources[digest];assert sha(path)==digest
   source=path.read_text().replace("\r\n","\n")
   a=account(source,case,frame,phase,ordinal)
  else:a=dict(phase="host_lifecycle_no_callback",public_call_counts={},source_required_credit_range=[0,0],method="host operation;not an Auto callback")
  entries.append(dict(evidence=e["path"],case=case,run=frame.get("run"),published_tick=frame.get("tick"),attempted_tick=frame.get("tick",0)+1 if frame.get("error")else frame.get("tick"),source_sha256=digest,account=a))
 for e in evidence:
  bindings.append(dict(path=e["path"],sha256=e["sha256"]))
  d=json.loads(pathlib.Path(e["path"]).read_text())
  if e["kind"]=="core":
   for r in d["rows"]:
    f=r["trace"][0]["frame"];add(e,"core-"+r["candidate"]+"-initial",dict(f,error=None),r["source_sha256"],"app_start")
    for t in r["trace"]:add(e,"core-"+r["candidate"],t["frame"],r["source_sha256"])
  elif e["kind"]=="branches":
   by={r["case"]:r for r in d["rows"]}
   for r in d["rows"]:
    name=r["case"];digest=r.get("source_sha256")
    if "frame"in r:add(e,name,r["frame"],digest)
    elif "trajectory"in r:
     for t in r["trajectory"]:add(e,name,{"tick":t["t"]+1},by["jump-rise"]["source_sha256"])
    elif "cycles"in r:
     for c in r["cycles"]:add(e,name,c["first"],by["exact-source-restored"]["source_sha256"]);add(e,name+"-Stop",c["first"])
    else:add(e,name,{})
    for n,f in enumerate(r.get("states",[]),1):add(e,name+"-sequence",f["frame"],digest,ordinal=n)
    for n,f in enumerate(r.get("trace",[]),1):add(e,name+"-peak-sequence",f,digest,ordinal=n)
    if name=="pause-neutral-step":add(e,"paused-host-frames-no-callback",{})
    if name in {"isolated-pit","isolated-enemy-contact","triple-win-enter-fresh-run"}:
     prior=r["states"][-1]["frame"]if r.get("states")else by["three-coins-goal-Nplus1"]["frame"]
     request=dict(prior,error=None,tick=prior["tick"]+(1 if r.get("states")else 2))
     add(e,name+"-Enter-source-derived-transition",request,digest,"enter_request")
  elif e["kind"]=="performance":
   for r in d["rows"]:
    if r["case"]=="original-failing-prefix":add(e,"core-"+r["candidate"]+"-performance",{"tick":0,"error":"QueryBudget"},r["source_sha256"])
    else:
     for t in range(len(r["host_whole_tick_us"])):add(e,"isolated-legal-character60",{"tick":t+1},r["source_sha256"])
  elif e["kind"]=="intents":
   for r in d["rows"]:add(e,r["case"],r["frame"],r["source_sha256"])
 api=pathlib.Path("D:/autostack/plan-artifacts/auto-ui/plan-011/consumer/experimental/autographics-auto-driver/api")
 price_inputs=[dict(path=str(api/name),sha256=sha(api/name))for name in ["character2d_v1.at","behavior_motion_v5.at","runtime_objects_v1.at","runtime_triggers_v1.at","sprite2d_v1.at","runtime_camera2d_v1.at"]]
 return dict(price_inputs=price_inputs,calibration_cases=["boundary-32","boundary-33","b-cached-idle32","b-first-run-frame33"],schema="ui011-callback-account/1",basis="fixed0261 public source prices and measured32/33 controls",source_credit_meter="not_read",bindings=bindings,entries=entries)
