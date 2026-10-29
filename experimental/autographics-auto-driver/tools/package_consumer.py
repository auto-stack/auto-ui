"""Export verified source inputs and pinned SDK, without build/evidence outputs."""
import pathlib,shutil,sys,json,subprocess
root=pathlib.Path(__file__).resolve().parents[1];dest=pathlib.Path(sys.argv[1]).resolve()
assert not dest.exists(),'refusing to overwrite existing package'
assert root not in dest.parents,'package must be outside producer'
subprocess.run([sys.executable,str(root/'tools/verify_source.py')],check=True)
latest=next(root/f'research/plan{n}-identity.json'for n in ['005','004','002','001','879','875','874','861','860','857','856','855','854','853','851','849','848','846','845','840','839','838','837','836','835']if(root/f'research/plan{n}-identity.json').exists())
identity=json.loads(latest.read_text());files=set(identity['files'])|{latest.relative_to(root.parent).as_posix()}
# Only receipt-listed inputs are exported: no target, native captures or producer
# working files become accidental dependencies of the source consumer.
for relative in sorted(files):
 source=root.parent/relative;assert source.is_file()and not source.is_symlink()and not source.is_junction(),relative
 target=dest/'experimental'/relative;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target)
if latest.name=='plan005-identity.json':
 for relative in identity['folders']:
  (dest/'experimental'/relative).mkdir(parents=True,exist_ok=True)
for relative in ([] if '--snapshot-only' in sys.argv[2:] else json.loads((root/'research/source-lock.json').read_text())['files']):
 source=root/'.sdk-source'/relative;assert source.is_file()and not source.is_symlink()and not source.is_junction(),relative
 target=dest/'experimental'/root.name/'.sdk-source'/relative;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target)
print(dest/'experimental'/root.name)
