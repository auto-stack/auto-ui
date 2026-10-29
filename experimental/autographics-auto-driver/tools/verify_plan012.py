"""Complete PLAN-012 source closure; --controls restores every mutation."""
import json
import sys
import plan012_source_policy as p

IDENTITY=p.ROOT/'research/plan012-identity.json'
def verify(quiet=False):
    v=json.loads(IDENTITY.read_text())
    for field,expected in dict(plan='auto-ui/PLAN-012',base_commit=p.BASE_COMMIT,
        plan_pin=p.PLAN_PIN,identity_fix_pin=p.FIX_PIN,sdk_pin=p.SDK_PIN,
        text_suffixes=list(p.TEXT),raw_exact_rules=list(p.RAW_EXACT_RULES)).items():
        assert v[field]==expected,'wrong '+field
    for relative in v['files']:p.safe_path(relative)
    actual,folders=p.collect()
    assert actual.keys()==v['files'].keys(),'source set drift'
    assert folders==v['folders'],'source folder drift'
    for relative,expected in v['files'].items():
        assert actual[relative]==expected,'source hash mismatch: '+relative
    assert v['sdk_frozen_files']==681==sum(n.startswith('autographics-auto-driver/sdk/frozen/') for n in actual),'SDK snapshot size'
    assert all(n in actual for n in p.REQUIRED),'required source unlisted'
    assert all(actual[n]==h for n,h in p.LOCKS.items()),'frozen dependency lock changed'
    assert json.loads((p.ROOT/'research/baseline.json.txt').read_text())['sdk']['repository_commit']==p.SDK_PIN,'SDK baseline pin'
    if not quiet:print(f'PASS PLAN-012 complete source: {len(actual)} hashes; 006 delivery, SDK681, two locks, audio/legacy/renderer/runner/projects bound')
def reject():
    try:verify(True)
    except (AssertionError,KeyError,FileNotFoundError):return
    raise AssertionError('mutation unexpectedly accepted')
def controls():
    verify()
    for relative in ['api/public_audio_v1.at','src/authoring/public_audio/codec.rs','tests/public_audio_consumer.rs',
        'samples/public_audio/project2d/scene.agscene','samples/public_audio/project3d/assets/b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav',
        'research/public-audio-codec-reference.json','Cargo.lock']:
        path=p.ROOT/relative;raw=path.read_bytes()
        try:
            path.write_bytes(raw+b'\ncontrol\n');reject()
            path.unlink();reject()
        finally:path.write_bytes(raw)
    for relative in ['samples/public_audio/unlisted-control.bin','research/authoring-native-stable.agscene']:
        path=p.ROOT/relative;assert not path.exists()
        try:path.write_bytes(b'unlisted');reject()
        finally:path.unlink(missing_ok=True)
    raw=IDENTITY.read_bytes()
    try:
        for field in ['base_commit','plan_pin','identity_fix_pin','sdk_pin']:
            v=json.loads(raw);v[field]='0'*40;IDENTITY.write_text(json.dumps(v));reject()
        for relative in ['../outside.rs','autographics-auto-driver/target/hidden.rs']:
            v=json.loads(raw);v['files'][relative]='0'*64;IDENTITY.write_text(json.dumps(v));reject()
    finally:IDENTITY.write_bytes(raw)
    path=p.ROOT/'research/authoring-native-987654321.agscene';assert not path.exists()
    try:
        path.write_bytes(b'volatile');verify(True)
        v=json.loads(raw);v['files'][path.relative_to(p.BASE).as_posix()]=p.digest(path)
        IDENTITY.write_text(json.dumps(v));reject()
    finally:path.unlink(missing_ok=True);IDENTITY.write_bytes(raw)
    path=p.ROOT/'samples/public_audio/project2d/scripts'
    for path in path.glob('*.at'):
        raw=path.read_bytes();assert b'\n' in raw and b'\r\n' not in raw
        try:path.write_bytes(raw.replace(b'\n',b'\r\n'));reject()
        finally:path.write_bytes(raw)
    verify()
    print('PASS PLAN-012 controls: edit/delete/unlisted/stable-scene/wrong-pins/escape/target/PID-pin/exact-bound-source rejected; volatile PID output ignored')
if __name__=='__main__':
    controls() if '--controls' in sys.argv else verify()
