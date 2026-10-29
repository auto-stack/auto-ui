"""Freeze PLAN-006 after implementation/tests/delivery inputs are stable."""
import json
import plan006_source_policy as p

files, folders = p.collect()
assert all(files[n]==value for n,value in p.LOCKS.items()),"frozen dependency lock changed"
for required in p.REQUIRED:
    assert required in files, f"required source absent: {required}"
identity = dict(plan="auto-ui/PLAN-006", base_commit=p.BASE_COMMIT,
                plan_pin=p.PLAN_PIN, identity_fix_pin=p.FIX_PIN, sdk_pin=p.SDK_PIN,
                sdk_frozen_files=sum(n.startswith("autographics-auto-driver/sdk/frozen/") for n in files),
                text_suffixes=list(p.TEXT), folders=folders, files=files)
assert identity["sdk_frozen_files"] == 681
out = p.ROOT / "research/plan006-identity.json"
out.write_text(json.dumps(identity, indent=1, sort_keys=True) + "\n", encoding="utf-8")
print(f"frozen {len(files)} files -> {out}")
