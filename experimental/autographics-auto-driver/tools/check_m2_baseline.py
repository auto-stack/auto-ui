"""Run five real Host workloads in fresh processes and reject incomplete evidence."""
import argparse
import copy
import ctypes
import json
import math
import os
import pathlib
import platform
import re
import subprocess
import time
from m2_evidence import ROOT, record_runner, verify_runner, sha, CURRENT
from m2_api_credentials import api_bytes

CASES = ['move', 'maze2d', 'corridor3d', 'wide128', 'deep128']


def quantile(values, q):
    assert values and all(isinstance(v, int) and v >= 0 for v in values), 'invalid numeric sample'
    return sorted(values)[math.ceil(q * len(values))-1]


def validate(d):
    assert d['schema'] == 'autographics.m2-baseline/1' and d['completed'] is True, 'partial baseline'
    assert d['case'] in CASES and type(d['observer']) is bool, 'case/mode'
    assert [t['tick'] for t in d['ticks']] == list(range(6, 36)), 'missing/duplicate accepted ticks'
    assert [t['tick'] for t in d['warmup']] == list(range(1, 6)), 'warmup missing'
    quantile([t['elapsed_ns'] for t in d['ticks']], .95)
    for key in ['read_ns','generate_ns','host_init_ns','first_render_ns','process_to_first_render_ns','run_sync_ns','initialization_ns','compile_us','stop_ns']:
        assert isinstance(d[key], int) and d[key] > 0, 'missing elapsed '+key
    assert d['initialization_ns'] >= d['run_sync_ns'], 'nested clock boundary'
    identity=d['identity']
    assert identity['version'] == 'autographics.behavior/'+str(identity['profile']-6), 'version/profile'
    for key in ['source','api','bridge','combined']:
        assert len(identity[key]) == 64 and all(c in '0123456789abcdef' for c in identity[key]), 'invalid identity'
    assert 0 < identity['managed_bytes'] <= 64*1024 and identity['user_bytes']+identity['managed_bytes']==identity['combined_bytes'], 'managed bytes'
    assert set(d['observations']) == {'vm_output_admission_pump','output_admission','renderer_prepare'}, 'segment coverage'
    for name,m in d['observations'].items():
        assert m['enabled'] == d['observer'] and m['dropped'] == 0, 'observer overflow/mode'
        assert len(m['samples_ns']) <= 4096 and bool(m['samples_ns']) == d['observer'], 'missing/unbounded segment'
        if d['observer']:quantile(m['samples_ns'],.95)
    assert d['owners'] == dict(alive_during_run=True,released_after_stop=True), 'VM/Descriptor owner'
    assert d['pixels']['editing']==d['pixels']['stop'], 'Stop content'
    assert all(isinstance(v,list) and len(v)==2 and all(len(h)==64 for h in v) for v in d['pixels'].values()), 'pixel evidence'
    for stage in ['before','init','warm','stop']:
        r=d['resources'][stage]
        for key in ['target_bytes','target_count','mesh_bytes','mesh_count','bytes_rewritten','buffer_rewrites']:
            assert isinstance(r[key],int) and r[key]>=0, 'resource metric'
        assert r['target_bytes']>0 and r['target_count']>0, 'no real renderer'
    frames=[f for t in d['warmup']+d['ticks'] for f in t['frames']]
    assert all(sum(not f['pending_after'] for f in t['frames'])==1 for t in d['warmup']+d['ticks']), 'whole frame publication count'
    for key in ['buffer_rewrites','bytes_rewritten']:
        assert sum(f[key] for f in frames)==d['resources']['warm'][key]-d['resources']['init'][key], 'unattributed uploads'
    # Pending rounds repaint the same already accepted scene; compare their exact
    # cost to an additional actual paused Host frame recorded by the runner.
    assert all({k:f[k] for k in ['buffer_rewrites','bytes_rewritten']}==d['idle_upload'] for f in frames if f['pending_after']), 'pending pump upload drift'
    for key in ['target_bytes','target_count','mesh_uploads']:

        assert d['resources']['init'][key]==d['resources']['warm'][key], 'warm resource changed '+key
    if d['case'].endswith('128'):assert d['configuration']['calls']==897, '128 fixture not actual config'
    return d


def semantic(d):
    # Exclude observation times and OS allocator values, retain every accepted value.
    return dict(identity=d['identity'], adapter=d['adapter'], pixels=d['pixels'], owners=d['owners'],
                documents=[t['document'] for t in d['warmup']+d['ticks']],
                resources={stage:{k:r[k] for k in ['target_bytes','target_count','mesh_bytes','mesh_count','mesh_uploads','scene_pipeline_buffers','depth_texture_bytes','two_buffers']} for stage,r in d['resources'].items()},
                configuration_calls=d['configuration']['calls'],
                accepted_uploads=[[(f['buffer_rewrites'],f['bytes_rewritten']) for f in t['frames'] if not f['pending_after']] for t in d['warmup']+d['ticks']])


def generated(folder):
    records=json.loads((folder/'generated.json').read_text(encoding='utf-8'))
    assert {r['name'] for r in records}=={f'v{v}-{line}' for v in range(1,6) for line in ['lf','crlf']} and len(records)==10, 'generated denominator'
    for r in records:
        s=r['snapshot']; raw=(folder/(r['name']+'.user.at')).read_bytes(); combined=(folder/(r['name']+'.combined.at')).read_bytes()
        import hashlib
        digest=lambda b:hashlib.sha256(b).hexdigest()
        assert bytes(s['bytes'])==raw and digest(raw)==s['user_hash'], 'raw source identity'
        assert digest(combined)==s['combined_hash'] and len(combined)==r['combined_bytes'], 'combined actual bytes'
        version=int(r['name'][1]);assert s['version']==f'autographics.behavior/{version}' and r['profile']==version+6, 'generated version/profile'
        assert len(combined)-len(raw)==r['managed_bytes'], 'managed bytes'
        regions=s['regions'];assert len(regions)==4 and [v['file'] for v in regions]==['<scene-transport>',f'<behavior_{["v1","spatial_v2","segment_v3","sweep_v4","motion_v5"][version-1]}.at>','fixture.at','<behavior-bridge>'], 'region labels'
        end=0;parts=[]
        for region in regions:
            assert region['start']==end and region['end']>=end and combined[region['end']:region['end']+1]==b'\n', 'region boundary'
            parts.append(combined[region['start']:region['end']]);end=region['end']+1
        assert end==len(combined) and parts[2]==raw, 'region coverage/user bytes'
        # Existing bridge credential uses namespaced transport plus pre-namespace
        # entry_text (behavior.rs). Recover only fixed transport identifiers;
        # trusted entry contains no strings/comments. Do not rewrite user bytes.
        original=(ROOT/'samples/authoring_play/main.at').read_text(encoding='utf-8')
        names=set(re.findall(r'^(?:fn|var|let) ([A-Za-z_][A-Za-z_0-9]*)',original,re.M))-{'main'}
        entry=parts[3].decode('utf-8')
        entry=re.sub(r'\b_ag_([A-Za-z_][A-Za-z_0-9]*)\b',lambda m:m[1] if m[1] in names else m[0],entry)
        assert digest(parts[0]+entry.encode('utf-8'))==s['bridge_hash'], 'compatible bridge credential'
        assert digest(api_bytes(ROOT,version))==s['api_hash'], 'original API credential'

        r['emitted_bridge_sha256']=digest(parts[0]+parts[3])
        r['credential_boundary']='api_hash pre-namespace assembly; bridge_hash namespaced transport + pre-namespace entry; emitted hashes bind actual final bytes. v3..v5 api_hash retained as original assembly credential; independent final API-region SHA covers actual output; assembly code remains identical to M1.'
        # api_hash is intentionally the pre-namespace API credential. Actual emitted
        # API bytes have an independent hash; these must never be conflated.
        r['emitted_region_sha256']=[digest(b) for b in parts]
    return records


def generated_controls(folder):
    path=folder/'generated.json';original=path.read_bytes()
    for index in range(len(json.loads(original))):
        for key in ['version','regions','api_hash','bridge_hash','combined_hash']:
            bad=json.loads(original)
            if key=='version':bad[index]['snapshot']['version']='autographics.behavior/99'
            elif key=='regions':bad[index]['snapshot']['regions'][0]['end']+=1
            else:bad[index]['snapshot'][key]='0'*64
            path.write_text(json.dumps(bad))
            try:generated(folder)
            except AssertionError:pass
            else:raise AssertionError('generated false green '+bad[index]['name']+' '+key)
            finally:path.write_bytes(original)
    generated(folder)


def memory_reader(pid):
    assert os.name=='nt', 'this measured profile requires Windows process working set'
    from ctypes import wintypes
    class Counters(ctypes.Structure):
        _fields_=[('cb',wintypes.DWORD),('faults',wintypes.DWORD)]+[(n,ctypes.c_size_t) for n in ['peak','working','pool_peak','pool','nonpool_peak','nonpool','page','peak_page']]
    kernel=ctypes.WinDLL('kernel32',use_last_error=True);psapi=ctypes.WinDLL('psapi',use_last_error=True)
    kernel.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];kernel.OpenProcess.restype=wintypes.HANDLE
    kernel.CloseHandle.argtypes=[wintypes.HANDLE]
    psapi.GetProcessMemoryInfo.argtypes=[wintypes.HANDLE,ctypes.POINTER(Counters),wintypes.DWORD]
    handle=kernel.OpenProcess(0x0400|0x0010,False,pid);assert handle, 'cannot sample process'
    def sample():
        c=Counters();c.cb=ctypes.sizeof(c)
        return [c.working,c.peak] if psapi.GetProcessMemoryInfo(handle,ctypes.byref(c),c.cb) else None
    return sample,lambda:kernel.CloseHandle(handle)


def sensitivity(valid):
    for mutate in [lambda d:d['ticks'].pop(),lambda d:d.update(completed=False),lambda d:d['owners'].update(released_after_stop=False),
                   lambda d:d['identity'].update(profile=99),lambda d:d['observations']['renderer_prepare'].update(dropped=1),
                   lambda d:d['pixels'].update(stop=['0'*64,'0'*64])]:
        bad=copy.deepcopy(valid);mutate(bad)
        try:validate(bad)
        except AssertionError:continue
        raise AssertionError('baseline false green control')
    validate(valid)


def summarize(case,rows):
    stats={}
    for enabled,values in rows.items():
        assert len(values)==5, 'fresh process denominator'
        ticks=[t['elapsed_ns'] for d in values for t in d['ticks']]
        medians=[quantile([t['elapsed_ns'] for t in d['ticks']],.5) for d in values]
        metrics={}
        for key in ['read_ns','generate_ns','host_init_ns','first_render_ns','process_to_first_render_ns','run_sync_ns','initialization_ns','compile_us','stop_ns']:
            samples=[d[key] for d in values]
            metrics[key]=dict(n=len(samples),p50=quantile(samples,.5),p95=quantile(samples,.95),raw=samples)
        frame_ns=[f['frame_ns'] for d in values for t in d['ticks'] for f in t['frames']]
        metrics['warm_frame_ns']=dict(n=len(frame_ns),p50=quantile(frame_ns,.5),p95=quantile(frame_ns,.95))
        for key in values[0]['observations']:
            samples=[v for d in values for v in d['observations'][key]['samples_ns']]
            metrics[key+'_ns']=dict(n=len(samples),p50=quantile(samples,.5) if samples else None,p95=quantile(samples,.95) if samples else None)
        stats['on'if enabled else'off']=dict(n=len(ticks),processes=len(values),p50_ns=quantile(ticks,.5),p95_ns=quantile(ticks,.95),process_p50_ns=medians,process_p50_range_ns=[min(medians),max(medians)],working_set_sampled_peak_bytes=[d['working_set_sampled_peak_bytes'] for d in values],named_metrics=metrics)
    overhead={k:dict(difference_ns=stats['on'][k]-stats['off'][k],ratio=stats['on'][k]/stats['off'][k]) for k in ['p50_ns','p95_ns']}
    return dict(case=case,stats=stats,observed_overhead=overhead)


def validate_existing(out):
    summary=json.loads((out/'summary.json').read_text(encoding='utf-8'))
    runner=json.loads((out/'runner.json').read_text(encoding='utf-8'));verify_runner(runner)
    assert summary['completed'] is True and summary['runner_sha256']==runner['sha256'], 'summary runner/completion'
    assert len(summary['reports'])==50 and len({v['path'] for v in summary['reports']})==50, 'process denominator'
    all_rows={v['path']:v['sha256'] for v in summary['reports']};actual=[]
    for case in CASES:
        rows={False:[],True:[]}
        for repeat in range(5):
            for enabled in [False,True]:
                name=f'{case}-{repeat}-'+('on'if enabled else'off');p=out/(name+'.json')
                assert all_rows.get(str(p))==sha(p), 'raw process report changed/missing'
                d=validate(json.loads(p.read_text(encoding='utf-8')))
                assert d['case']==case and d['observer']==enabled and d['runner_sha256']==runner['sha256'], 'report identity'
                assert d['invocation']['exit']==0 and d['invocation']['command']==[runner['path'],case,'on'if enabled else'off',str(p)], 'actual invocation'
                assert d['invocation']['raw_log_sha256']==sha(out/(name+'.log.txt')), 'raw process log changed'
                samples=d['memory_samples_ns_working_ospeak'];assert samples and all(a[0]<b[0] for a,b in zip(samples,samples[1:])), 'memory sampling sequence'
                assert max(v[1] for v in samples)==d['working_set_sampled_peak_bytes'] and max(v[2] for v in samples)==d['working_set_os_peak_bytes'], 'working set peak'
                rows[enabled].append(d);assert semantic(d)==semantic(rows[False][0]), 'on/off semantic drift'
        actual.append(summarize(case,rows))
    assert actual==summary['cases'], 'statistics do not match raw samples'
    assert sha(out/'generated-hashes.json')==summary['generated_sha256'], 'generated handoff changed'
    assert generated(out/'generated')==json.loads((out/'generated-hashes.json').read_text(encoding='utf-8')), 'generated actual bytes changed'
    return True


def main():
    p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);p.add_argument('--self-check',action='store_true');p.add_argument('--validate-only',action='store_true');p.add_argument('--executable',type=pathlib.Path,default=ROOT/'target/debug/autographics-m2-baseline.exe');a=p.parse_args()
    out=a.output.resolve()
    if a.validate_only:
        validate_existing(out)
        if a.self_check:generated_controls(out/'generated')
        print('PASS existing50 actual process reports/1500 accepted ticks, statistics, immutable runner and generated bytes',flush=True)
        return
    out.mkdir(parents=True,exist_ok=False)
    runner=record_runner(a.executable,out/'runner.json');exe=verify_runner(runner)
    env=os.environ.copy();env['RUSTC_WRAPPER']='';env['RUST_MIN_STACK']='16777216';env['RUSTFLAGS']='-C link-arg=/STACK:33554432'
    identity=ROOT/f'research/plan{CURRENT}-identity.json'
    environment=dict(platform=platform.platform(),machine=platform.machine(),processor=platform.processor(),computer=os.environ.get('COMPUTERNAME'),rustc=subprocess.check_output(['rustc','-Vv'],text=True),cargo=subprocess.check_output(['cargo','-V'],text=True),python=platform.python_version(),source_identity_sha256=sha(identity),sdk_commit=json.loads((ROOT/'research/baseline.json.txt').read_text())['sdk']['repository_commit'],runner=runner,configuration={k:env[k] for k in ['RUSTFLAGS','RUST_MIN_STACK','RUSTC_WRAPPER']},clock='Rust Instant monotonic ns; external perf_counter_ns polling',cold='new process, OS file/GPU caches not flushed',working_set_interval_ms=10,working_set_limit='sampled maximum can miss peaks shorter than interval; report actual timestamps separately from OS peak')
    (out/'environment.json').write_text(json.dumps(environment,indent=2))
    all_rows=[];summaries=[]
    for case in CASES:
        rows={False:[],True:[]}
        # Alternate modes per pair to make machine drift visible.
        for repeat in range(5):
            for enabled in [False,True]:
                name=f'{case}-{repeat}-'+('on' if enabled else 'off');report=out/(name+'.json');log=out/(name+'.log.txt')
                command=[str(exe),case,'on'if enabled else'off',str(report)]
                verify_runner(runner);started=time.perf_counter_ns();samples=[]
                with log.open('wb') as f:
                    proc=subprocess.Popen(command,cwd=out,env=env,stdout=f,stderr=subprocess.STDOUT)
                    sample,close=memory_reader(proc.pid)
                    try:
                        while proc.poll() is None:
                            memory=sample()
                            if memory:samples.append([time.perf_counter_ns()-started,*memory])
                            if time.perf_counter_ns()-started>180_000_000_000:
                                proc.kill();proc.wait();raise AssertionError('baseline process timeout')
                            time.sleep(.01)
                    finally:close()
                assert proc.returncode==0, f'{name} failed: {log.read_text(errors="replace")[-3000:]}'
                d=validate(json.loads(report.read_text(encoding='utf-8')))
                assert d['case']==case and d['observer']==enabled and samples, 'wrong report/process samples'
                verify_runner(runner);d['runner_sha256']=runner['sha256'];d['memory_samples_ns_working_ospeak']=samples
                d['working_set_sampled_peak_bytes']=max(v[1] for v in samples);d['working_set_os_peak_bytes']=max(v[2] for v in samples)
                d['invocation']=dict(command=command,exit=proc.returncode,process_elapsed_ns=time.perf_counter_ns()-started,raw_log_sha256=sha(log))
                report.write_text(json.dumps(d,indent=2));all_rows.append(dict(path=str(report),sha256=sha(report)));rows[enabled].append(d)
                assert semantic(d)==semantic(rows[False][0]), 'observer/process semantic/source/resource/content drift'
                if a.self_check:sensitivity(d)
                print(name+' PASS',flush=True)
        summaries.append(summarize(case,rows))
    command=[str(exe),'--generated',str(out/'generated')];subprocess.run(command,cwd=out,env=env,check=True,timeout=60)
    records=generated(out/'generated')
    if a.self_check:generated_controls(out/'generated')
    (out/'generated-hashes.json').write_text(json.dumps(records,indent=2))
    (out/'summary.json').write_text(json.dumps(dict(completed=True,runner_sha256=runner['sha256'],quantile='nearest rank ceil(q*N)-1',cases=summaries,reports=all_rows,generated_sha256=sha(out/'generated-hashes.json')),indent=2))
    print('PASS five workloads, 50 fresh processes, 1500 warm accepted ticks, actual generated bytes/regions and controls',flush=True)


if __name__=='__main__':main()
