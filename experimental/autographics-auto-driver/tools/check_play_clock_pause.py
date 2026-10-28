"""Actual OS/Auto/GPU clock-pause controls; production thresholds are unchanged."""
from pathlib import Path
import argparse, hashlib, json, os, re, subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('output',type=Path)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
root=Path(__file__).resolve().parents[1]
entry=root/'src/bin/autographics-authoring.rs';probe=root/'src/bin/authoring_probe/play.rs'
original={f:f.read_bytes()for f in [entry,probe,root/'src/bin/authoring/cli.rs',root/'src/bin/authoring/probes.rs']};results=[]
libraries={name:max((root/'target/debug/deps').glob('lib'+name+'-*.rlib'),key=lambda f:f.stat().st_mtime_ns)for name in ['agp','autographics_auto_driver','winit','iced_wgpu','serde_json','glam']}
lib_hashes={str(f):hashlib.sha256(f.read_bytes()).hexdigest()for f in libraries.values()}
for name,phase,mutant in [('motion',3,False),('held-motion',4,False),('single-v',17,False),('accepted-v',17,False),('restore-v',21,False),('no-recovery',3,True)]:
    d=out/name;d.mkdir();copy=d/'play.rs.txt';text=probe.read_text()
    if mutant:
        assert text.count('if self.live_input_phase() {')==1
        text=text.replace('if self.live_input_phase() {','if false && self.live_input_phase() {')
    copy.write_text(text)
    code=re.sub(r'"((?:authoring_probe|authoring)/[^"\r\n]+)"',lambda m:json.dumps((copy if m[1]=='authoring_probe/play.rs'else root/'src/bin'/m[1]).as_posix()),entry.read_text())
    code=code.replace('    frames: u64,','    frames: u64,\n    clock_probe_injected: bool,')
    anchor='            WindowEvent::RedrawRequested => {';assert code.count(anchor)==1
    accepted=' && h.play.as_ref().is_some_and(|p|p.last.as_ref().is_some_and(|f|!f.document.objects[1].visible))' if name=='accepted-v' else ''
    code=code.replace(anchor,anchor+f'''
                if !self.clock_probe_injected && self.play_probe.as_ref().is_some_and(|p|p.phase == {phase}){accepted} {{
                    self.clock_probe_injected=true;
                    std::thread::sleep(Duration::from_millis(100));
                    eprintln!("CLOCK_PROBE injected100ms phase{phase}");
                }}''')
    source=d/'entry.rs.txt';source.write_text(code);exe=d/'entry.exe'
    cmd=['rustc','--edition=2021','--crate-name','clock_pause_probe','--crate-type','bin',str(source),'-L','dependency='+str(root/'target/debug/deps'),'-C','debuginfo=0','-C','link-arg=/STACK:33554432','-o',str(exe)]
    for libname,lib in libraries.items():
        assert hashlib.sha256(lib.read_bytes()).hexdigest()==lib_hashes[str(lib)], 'Concurrent build invalidated probe inputs'
        cmd+=['--extern',libname+'='+str(lib)]
    for output in (root/'target/debug/build').glob('*/output'):
        for line in output.read_text(errors='replace').splitlines():
            if line.startswith('cargo:rustc-link-search=native='):cmd+=['-L',line.split('rustc-link-search=',1)[1]]
    env=dict(os.environ,CARGO_MANIFEST_DIR=str(root),RUST_MIN_STACK='16777216')
    r=subprocess.run(cmd,cwd=root,env=env,capture_output=True,timeout=120)
    (d/'build.log.txt').write_bytes(r.stdout+r.stderr);assert r.returncode==0,(name,r.stderr[-3000:])
    command=['python',str(root/'tools/check_user_behavior.py'),str(d/'native'),'--cases','patrol','--executable',str(exe)]
    r=subprocess.run(command,cwd=root,env=env,capture_output=True,timeout=120)
    (d/'aggregate.log.txt').write_bytes(r.stdout+r.stderr)
    report=json.loads((d/'native/patrol/report.json').read_text())
    assert 'CLOCK_PROBE injected100ms' in (d/'native/patrol/stderr.txt').read_text()
    if mutant:
        assert r.returncode!=0 and not report['passed'] and 'play timeout phase 4' in report['error'],report['error']
    else:
        assert r.returncode==0 and report['passed'] and report['play_phase']==27,(name,report['error'])
        pauses=[v for v in report['play_records']if v['case']=='clock-pause-recovery']
        assert any(v['reason']=='auto-Pause: tick backlog >4'for v in pauses),name
        reposts=[v for v in report['play_records']if v['case']=='reposted-unaccepted-v'and v['phase']==phase]
        if name in ['single-v','restore-v']:assert any(v['edges']==1 for v in reposts),name
        if name=='accepted-v':assert not reposts,'Accepted V batch must not be applied twice'
    row=dict(case=name,phase=phase,mutant=mutant,command=command,exit=r.returncode,passed=report['passed'],error=report['error'],binary_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),rlibs=lib_hashes)
    results.append(row);print(json.dumps(row),flush=True)
for f,b in original.items():assert f.read_bytes()==b
(out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
