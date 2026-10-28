"""F-837-R1-01 actual-GPU sensitivity controls, adapted from the durable R1 replay.
Build the native entry first. No producer/SDK/AGP source is modified: test-only
entry copies use the actual probe modules and locked compiled libraries.
Usage: python tools/check_hierarchy_pixel_faults.py [new output directory]
"""
import hashlib,json,os,pathlib,re,subprocess,sys
root=pathlib.Path(__file__).resolve().parents[1]
out=pathlib.Path(sys.argv[1]).resolve() if len(sys.argv)>1 else root/'research/hierarchy-pixel-faults'
out.mkdir(parents=True,exist_ok=False)
entry=root/'src/bin/autographics-authoring.rs'
probe=root/'src/bin/authoring_probe/hierarchy.rs'
cli=root/'src/bin/authoring/cli.rs'
original={p:p.read_bytes() for p in [entry,probe,cli,root/'src/bin/authoring/probes.rs']}
gate=root/'tools/check_hierarchy.py'
base=entry.read_text()
def probe_path(match):
 path=(root/'src/bin'/match[1]).resolve()
 assert (root/'src/bin').resolve() in path.parents and path.is_file(),path
 return json.dumps(path.as_posix())
base,count=re.subn(r'"((?:authoring_probe|authoring)/[^"\r\n]+)"',probe_path,base)
assert count>0 and '"authoring_probe/' not in base and '"authoring/' not in base,'unresolved native probe module'
helper=r'''
fn pixel_render_fault(h: &mut Host) -> Result<(), String> {
    let Some(o) = h.editor.preview.objects.iter().find(|o|o.id==1) else {return Ok(())};
    if !o.visible || o.parent.is_none() || o.translation!=[-25.0,20.0,0.0] {return Ok(())}
    let before=h.editor.document.clone();let history=h.editor.history();let selection=h.editor.selection;
    use agp::host::surface_runtime::SurfaceContent;
    let original=if let Some(SurfaceContent::Game2d(c))=h.rt.content_mut(h.two) {
        let original=c.commands.clone();let mut count=0;
        for cmd in &mut c.commands {
            if cmd.center==[375.0,170.0] && cmd.size==[50.0,50.0] {
                __MUTATION__
                count+=1;
            }
        }
        assert_eq!(count,1,"actual leaf command selected before GPU fault");
        __EXTRA_COMMAND__
        original
    } else {return Err("missing 2D consumer".into())};
    h.rt.graph.mark_content_dirty(h.two);
    h.rt.frame(None,&Default::default()).map_err(|e|format!("{e:?}"))?;
    let background=h.rt.probe_pixel(h.top,[20.0,20.0]).expect("actual background readback");
    __GPU_WITNESS__
    if let Some(SurfaceContent::Game2d(c))=h.rt.content_mut(h.two) {c.commands=original;}
    assert_eq!(h.editor.document,before);assert_eq!(h.editor.history(),history);assert_eq!(h.editor.selection,selection);
    eprintln!("F-837-R1-01 __CASE__: actual GPU fault observed; command inspection restored");
    Ok(())
}
'''
results=[]
for case in ['transparent','ghost']:
 directory=out/case;directory.mkdir()
 if case=='transparent':
  mutation='cmd.color=agp::core::scene::Color::rgba(0.0,0.0,0.0,0.0);'
  extra=''
  witness='assert_eq!(h.rt.probe_pixel(h.top,[375.0,170.0]),Some(background),"real moved leaf is transparent");'
 else:
  mutation='// Retain correct endpoint; add a wrong ghost at the vacated origin.'
  extra='let mut ghost=original.iter().find(|c|c.center==[375.0,170.0]&&c.size==[50.0,50.0]).unwrap().clone();ghost.center=[345.0,190.0];c.commands.push(ghost);'
  witness='let endpoint=h.rt.probe_pixel(h.top,[375.0,170.0]).expect("endpoint");let origin=h.rt.probe_pixel(h.top,[345.0,190.0]).expect("origin");assert_ne!(endpoint,background);assert_eq!(origin,endpoint,"real old-position ghost retains leaf color");'
 code=base;needle='                self.frames += 1;';assert code.count(needle)==1
 code=code.replace(needle,'                pixel_render_fault(h).unwrap();\n'+needle)
 cli_code=cli.read_text();needle='            h.frame(None)?;';assert cli_code.count(needle)==1
 cli_copy=directory/'cli.rs.txt'
 cli_copy.write_text(cli_code.replace(needle,needle+'\n            pixel_render_fault(&mut h)?;'))
 cli_path=json.dumps(cli.resolve().as_posix());assert code.count(cli_path)==1
 code=code.replace(cli_path,json.dumps(cli_copy.as_posix()))
 code+=helper.replace('__MUTATION__',mutation).replace('__EXTRA_COMMAND__',extra).replace('__GPU_WITNESS__',witness).replace('__CASE__',case)
 source=directory/'fault-entry.rs.txt';source.write_text(code);exe=directory/'fault-entry.exe'
 command=['rustc','--edition=2021','--crate-name','hierarchy_pixel_fault','--crate-type','bin',str(source),'-L','dependency='+str(root/'target/debug/deps'),'-C','debuginfo=0','-C','link-arg=/STACK:33554432','-o',str(exe)]
 inputs={}
 for name in ['agp','autographics_auto_driver','winit','iced_wgpu','serde_json','glam']:
  lib=max((root/'target/debug/deps').glob(f'lib{name}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
  command+=['--extern',name+'='+str(lib)];inputs[str(lib)]=hashlib.sha256(lib.read_bytes()).hexdigest()
 for output in (root/'target/debug/build').glob('*/output'):
  for line in output.read_text(errors='replace').splitlines():
   if line.startswith('cargo:rustc-link-search=native='):command+=['-L',line.split('rustc-link-search=',1)[1]]
 build=subprocess.run(command,cwd=root,env=dict(os.environ,CARGO_MANIFEST_DIR=str(root),RUST_MIN_STACK='16777216'),capture_output=True,timeout=120)
 (directory/'build.log.txt').write_bytes(build.stdout+build.stderr);assert build.returncode==0,(case,build.stderr[-4000:])
 run=subprocess.run([sys.executable,str(gate),str(directory/'native'),'--executable',str(exe)],cwd=root,capture_output=True,timeout=90)
 (directory/'aggregate.log.txt').write_bytes(run.stdout+run.stderr)
 report=json.loads((directory/'native/positive.json').read_text())
 stderr=(directory/'native/positive.stderr.txt').read_text()
 assert run.returncode!=0 and report['passed'] is False and report['hierarchy_phase']==9,(case,report)
 assert 'move: composed endpoint/origin pixels mismatch' in report['error'],report
 assert f'F-837-R1-01 {case}: actual GPU fault observed' in stderr,stderr
 pixels=[r for r in report['hierarchy_records'] if r['case']=='composed-pixels' and r['stage']=='move'];assert len(pixels)==1
 pixel=pixels[0];assert pixel['passed'] is False
 if case=='transparent':assert pixel['endpoint']==pixel['background']!=pixel['color']
 else:assert pixel['endpoint']==pixel['color'] and pixel['origin']==pixel['color']!=pixel['background']
 results.append(dict(case=case,aggregate_exit=run.returncode,native_error=report['error'],phase=report['hierarchy_phase'],pixel=pixel))
 (directory/'provenance.json').write_text(json.dumps(dict(command=command,rlibs_sha256=inputs,copy_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),producer_source_modified=False),indent=2)+'\n')
for p,b in original.items():assert p.read_bytes()==b
(out/'results.json').write_text(json.dumps(dict(results=results,source_hashes={str(p.relative_to(root)):hashlib.sha256(b).hexdigest() for p,b in original.items()}),indent=2)+'\n')
print(json.dumps(results,indent=2))
