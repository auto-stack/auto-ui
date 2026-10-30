"""Frozen directed PLAN-013 gate. --validate reuses revision-bound evidence.

Run only when implementation is frozen (review/merge owner). --controls rejects
stale/partial/empty/wrong-runner and changed observable proofs after a real run.
"""
import argparse, copy, hashlib, json, os, pathlib, re, secrets, struct, subprocess, sys, tempfile
import plan013_source_policy as p
SCHEMA='autographics.plan013.report/1'
RUNNERS={
 'public':('public_hud',17,0,None),
 'codec':('public_audio_codec',6,0,None),
 'resources':('public_audio_resources',9,0,None),
 'host':('public_audio_host',12,0,None),
 'legacy':('public_audio_compat',2,0,None),
 'consumer':('public_hud_consumer',2,0,None),
 'units':(None,5,33,'audio'),
}
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity():return digest(p.ROOT/'research/plan013-identity.json')
def stack(binary):
    raw=binary.read_bytes();assert raw[:2]==b'MZ','PE runner required'
    offset=struct.unpack_from('<I',raw,0x3c)[0]
    assert raw[offset:offset+4]==b'PE\0\0' and struct.unpack_from('<H',raw,offset+24)[0]==0x20b,'x64 PE required'
    return struct.unpack_from('<Q',raw,offset+24+72)[0]
def counts(name,output):
    _,passed,filtered,_=RUNNERS[name]
    matches=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',output)
    assert len(matches)==1 and tuple(map(int,matches[0]))==(passed,0,0,0,filtered),'partial/zero-selected runner '+name
    if name=='legacy':
        assert 'Verified15 legacy profiles against frozen exact-byte baseline' in output and 'Verified17 pre-HUD profiles against the pristine df39593 assembler' in output and 'capture mode is NOT verification' not in output,'legacy comparison missing/capture-only proof'
    if name=='units':
        for case in ['real_audio_wire_mutations_cannot_accept_partial_frame_or_insert_pcm',
          'observed_native_pcm_is_after_mono_mapping_and_signed_unsigned_quantization',
          'fixed_three_cue_bank_fits_real_five_visual_raws_and_a_reserved_font_slot',
          'round_restart_reopens_faulted_stream_and_invalidates_old_run_owner',
          'preaccept_audio_start_failure_restores_editing_view_and_keeps_host_idle']:
            assert case+' ...' in output,'wrong selected unit'
def guard():
    env=dict(os.environ);env.pop('CARGO_TARGET_DIR',None)
    result=subprocess.run([sys.executable,str(p.ROOT/'tools/verify_source.py')],cwd=p.ROOT,env=env,capture_output=True,text=True)
    assert result.returncode==0,(result.stdout+result.stderr)[-5000:]
    return result.stdout+result.stderr
def mask_distance(actual,expected):
    def points(value):
        assert isinstance(value,list) and value,'GPU glyph identity'
        assert all(isinstance(p,list) and len(p)==2 and all(type(n) is int and n>=0 for n in p) for p in value),'GPU glyph identity'
        result={tuple(p) for p in value};assert len(result)==len(value),'GPU glyph identity'
        return result
    actual,expected=points(actual),points(expected)
    return len(actual^expected)/len(actual|expected)
def data(directory,manifest_dir):
    c=json.loads((directory/'consumer.json').read_text());v=json.loads((directory/'performance.json').read_text())
    assert pathlib.Path(c['manifest_dir']).resolve()==manifest_dir and pathlib.Path(v['manifest_dir']).resolve()==manifest_dir,'foreign compiled consumer'
    assert [r['profile'] for r in c['projects']]==['autographics.hud2d/1','autographics.hud3d/1'],'partial dimension set'
    golden=json.loads((p.ROOT/'research/public-hud-gpu-golden.json').read_text())
    for r in c['projects']:
        assert all(r[k] is True for k in ['moved_project','cold_reopen','user_mutation_sensitive','cue_gain_waveform_sensitive','event_selects_cue1','licenses_moved','hud_dynamic']),'public consumer oracle'
        assert r['pcm_frames_compared']==12645 and re.fullmatch('[0-9a-f]{64}',r['source_sha256']),'consumer PCM/source oracle'
        assert len(r['gpu_oracles'])==3,'GPU glyph identity'
        for i,actual in enumerate(r['gpu_oracles']):
            expected=golden[r['profile']][str(i)]
            assert set(actual)==set(expected)=={'slot0','slot1','slot3'},'GPU glyph identity'
            assert all(mask_distance(actual[slot],expected[slot])<=0.02 for slot in expected),'GPU glyph identity'
    assert v['guards']==dict(credits=32,slice_steps=1000,callback_steps=100000,resumptions=120,callback_ms=250,init_ms=30000,tick_ms=2000,pump_target_us=8000),'guard changed'
    assert [r['project'] for r in v['projects']]==['project2d','project3d'],'partial performance set'
    for i,r in enumerate(v['projects']):
        assert r['user_sha256']==c['projects'][i]['source_sha256'],'wrong source consumer'
        assert 0<r['steps']<=100000 and 0<r['slices']<=121 and 0<r['managed_bytes']<=96*1024 and r['ticks']==12,'callback source/work budget'
        assert all(r[k]==0 for k in ['warm_decode_delta','warm_mesh_uploads','warm_texture_uploads','warm_atlas_builds_delta']),'warm resource invariant'
        for k in ['tick','vm_output','admission','host_prepare','host_publish','host_pump']:
            s=r[k];assert type(s['count']) is int and s['count']>=12 and 0<s['median_us']<=s['max_us'],'missing CPU observation'
        assert r['tick']['max_us']<2_000_000,'tick hard deadline'
        assert r['pump_target_us']==8000 and r['pump_target']==('met' if r['host_pump']['max_us']<=8000 else 'missed'),'pump target misreported'
        u=r['resources'];assert u['count']==[3,7][i] and 0<u['raw_bytes']<=32*1024**2 and 0<u['cpu_total_with_pose_bytes']<=64*1024**2 and 0<u['gpu_expected_bytes']<=64*1024**2,'resource union budget'
        assert u['cpu_total_with_pose_bytes']==u['cpu_geometry_bytes']+u['pose_reserved_cpu_bytes'],'pose missing from budget'
        a=r['native'];assert a['manager'] is True and a['fault'] is None and a['voices']==0 and a['callbacks']>0 and a['nonzero']>0 and a['peak']>0 and a['quantum_frames']>0 and a['max_us']>0 and a['reserved_cpu_bytes']>0,'native output oracle'
        assert type(a['deadline_misses']) is int and a['deadline_misses']>=0 and r['resume_to_first_pcm_upper_us']>0,'missing native timing'
def run(directory,fresh_directory=None):
    assert not any(directory.iterdir()),'reports must be fresh/empty'
    (directory/'source.log').write_text(guard(),encoding='utf-8')
    proofs={}
    for name,(target,passed,filtered,selection) in RUNNERS.items():
        env=dict(os.environ,UI013_REPORT_DIR=str(directory),CARGO_TERM_COLOR='never',RUST_MIN_STACK='16777216');env.pop('RUSTC_WRAPPER',None)
        env.pop('PLAN013_CAPTURE_LEGACY',None)
        command=['cargo','test','--locked','--offline']+(['--test',target] if target else ['--lib'])
        if selection:command.append(selection)
        command+=['--','--test-threads=1','--nocapture']
        print('RUN',name,flush=True)
        result=subprocess.run(command,cwd=p.ROOT,env=env,capture_output=True,text=True);out=result.stdout+result.stderr
        log=directory/(name+'.log');log.write_text(out,encoding='utf-8');assert result.returncode==0,out[-5000:];counts(name,out)
        m=re.search(r'Running [^\r\n]*\(([^\r\n()]+)\)',out);assert m,'actual runner missing'
        binary=pathlib.Path(m[1]);binary=(p.ROOT/binary).resolve() if not binary.is_absolute() else binary.resolve()
        assert stack(binary)==33554432,'main stack changed'
        proofs[name]=dict(target=target,passed=passed,filtered=filtered,binary=str(binary),binary_sha256=digest(binary),log_sha256=digest(log),worker_stack=16777216)
    import fresh_public_hud_consumer as fresh
    if fresh_directory is None:
        fresh_directory=directory.parent/(directory.name+'-fresh')
        fresh.run(fresh_directory)
    fresh.validate(fresh_directory)
    manifest=dict(schema=SCHEMA,source_identity=identity(),nonce=secrets.token_hex(16),runners=proofs,source_log_sha256=digest(directory/'source.log'),data_sha256={n:digest(directory/(n+'.json')) for n in ['consumer','performance']},fresh=str(fresh_directory.resolve()),fresh_manifest_sha256=digest(fresh_directory/'manifest.json'))
    (directory/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
def validate(directory,source_guard=True,check_fresh=True):
    if source_guard:guard()
    v=json.loads((directory/'manifest.json').read_text())
    assert v['schema']==SCHEMA and v['source_identity']==identity(),'stale/wrong source identity'
    assert re.fullmatch('[0-9a-f]{32}',v['nonce']),'run nonce missing'
    assert set(v['runners'])==set(RUNNERS),'partial runner set'
    assert digest(directory/'source.log')==v['source_log_sha256'],'source log changed'
    allowed=pathlib.Path(os.environ.get('CARGO_TARGET_DIR',str(p.ROOT/'target'))).resolve()
    for name,(target,passed,filtered,_) in RUNNERS.items():
        r=v['runners'][name];binary=pathlib.Path(r['binary']).resolve()
        assert (r['target'],r['passed'],r['filtered'])==(target,passed,filtered),'wrong runner/count'
        assert allowed in binary.parents and binary.name.startswith((target or 'autographics_auto_driver')+'-'),'foreign runner path'
        assert digest(binary)==r['binary_sha256'] and stack(binary)==33554432 and r['worker_stack']==16777216,'changed runner/stack'
        log=directory/(name+'.log');assert digest(log)==r['log_sha256'],'proof log changed';counts(name,log.read_text(encoding='utf-8'))
    for name in ['consumer','performance']:assert digest(directory/(name+'.json'))==v['data_sha256'][name],'data proof changed'
    data(directory,p.ROOT.resolve())
    fresh_dir=pathlib.Path(v['fresh']);assert digest(fresh_dir/'manifest.json')==v['fresh_manifest_sha256'],'fresh proof changed'
    if check_fresh:
        import fresh_public_hud_consumer as fresh
        fresh.validate(fresh_dir)
    print('PASS PLAN-013 bound source/runner/PCM/native/owner/consumer/budget proofs')
def controls(directory):
    original=json.loads((directory/'manifest.json').read_text())
    def rejected(change,needle):
        dest=pathlib.Path(tempfile.mkdtemp(prefix='ui013-control-',dir=directory.parent))
        for f in directory.iterdir():
            if f.is_file() and f.suffix in ['.json','.log']:(dest/f.name).write_bytes(f.read_bytes())
        v=copy.deepcopy(original);change(dest,v);(dest/'manifest.json').write_text(json.dumps(v),encoding='utf-8')
        try:validate(dest,False,False)
        except (AssertionError,KeyError,FileNotFoundError) as e:assert needle in str(e),str(e)
        else:raise AssertionError('accepted control: '+needle)
    rejected(lambda w,v:v.update(source_identity='0'*64),'source identity')
    rejected(lambda w,v:v['runners'].pop('host'),'partial runner')
    rejected(lambda w,v:v['runners']['public'].update(passed=0),'wrong runner/count')
    rejected(lambda w,v:v['runners']['host'].update(target='public_audio'),'wrong runner/count')
    def capture_only(w,v):
        f=w/'legacy.log';out=f.read_text(encoding='utf-8')
        marker='Verified15 legacy profiles against frozen exact-byte baseline'
        assert out.count(marker)==1,'legacy comparison marker not actually executed'
        out=out.replace(marker,'Captured15 legacy snapshots; capture mode is NOT verification')
        f.write_text(out,encoding='utf-8');v['runners']['legacy']['log_sha256']=digest(f)
    rejected(capture_only,'legacy comparison')
    def changed(name,mutate):
        def apply(w,v):
            f=w/(name+'.json');data=json.loads(f.read_text());mutate(data);f.write_text(json.dumps(data),encoding='utf-8');v['data_sha256'][name]=digest(f)
        return apply
    rejected(changed('consumer',lambda v:v['projects'][0].update(cue_gain_waveform_sensitive=False)),'consumer oracle')
    rejected(changed('consumer',lambda v:v['projects'][0].update(hud_dynamic=False)),'consumer oracle')
    rejected(changed('consumer',lambda v:v['projects'][0]['gpu_oracles'][0].update(slot0='0'*64)),'GPU glyph identity')
    rejected(changed('performance',lambda v:v['guards'].update(credits=33)),'guard changed')
    rejected(changed('performance',lambda v:v['projects'][0].update(warm_decode_delta=1)),'warm resource')
    rejected(changed('performance',lambda v:v['projects'][0].update(warm_atlas_builds_delta=1)),'warm resource')
    rejected(changed('performance',lambda v:v['projects'][0]['native'].update(nonzero=0)),'native output')
    rejected(changed('performance',lambda v:v['projects'][0]['resources'].update(cpu_total_with_pose_bytes=128*1024**2)),'resource union')
    binary=pathlib.Path(original['runners']['public']['binary'])
    result=subprocess.run([str(binary),'__ui013_no_such_test__','--test-threads=1'],cwd=p.ROOT,capture_output=True,text=True);assert result.returncode==0
    try:counts('public',result.stdout+result.stderr)
    except AssertionError:pass
    else:raise AssertionError('accepted actual zero-selected execution')
    validate(directory,False)
    print('PASS PLAN-013 controls: stale/partial/wrong-runner/actual-zero-selection/PCM/native/warm/guard/resource mutations refused')
if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--reports',type=pathlib.Path);ap.add_argument('--validate',type=pathlib.Path);ap.add_argument('--fresh',type=pathlib.Path);ap.add_argument('--controls',action='store_true');a=ap.parse_args();assert not(a.reports and a.validate)
    if a.validate:directory=a.validate.resolve()
    else:
        directory=a.reports.resolve() if a.reports else pathlib.Path(tempfile.mkdtemp(prefix='ui013-reports-')).resolve()
        if a.reports:assert not directory.exists();directory.mkdir(parents=True)
        run(directory,a.fresh.resolve() if a.fresh else None)
    validate(directory)
    if a.controls:controls(directory)
