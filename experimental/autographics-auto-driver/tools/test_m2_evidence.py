"""Finding-specific permanent regressions for F-853-R1-01/02/03."""
import json
import hashlib
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

import check_m2_baseline as baseline
import m2_evidence as evidence

ROOT=pathlib.Path(__file__).resolve().parents[1]


class SelectionCompletionTests(unittest.TestCase):
    good='Running tests/example.rs (target/debug/deps/example.exe)\nrunning 1 test\ntest required ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n'
    expected={'Running tests/example.rs':['required']}

    def check_both(self,raw,accepted):
        # Exercise the real consumer and file/runner hash checks, not a mock
        # parser. Runtime dispatch is outside this evidence-only unit fixture.
        with tempfile.TemporaryDirectory() as temp:
            root=pathlib.Path(temp);payload=b'opaque runner payload for evidence unit'
            digest=hashlib.sha256(payload).hexdigest();exe=root/digest/'runner.bin'
            exe.parent.mkdir();exe.write_bytes(payload)
            runner=dict(schema='autographics.runner/1',path=str(exe),sha256=digest)
            log=root/'raw.log';log.write_text(raw)
            receipt=dict(completed=True,exit=0,expected_exit=0,runner_sha256=digest,
                         raw_log=str(log),raw_sha256=evidence.sha(log),selected=self.expected)
            if accepted:
                blocks=evidence.check_selection(raw,self.expected)
                self.assertTrue(evidence.validate_run(receipt,runner));return blocks
            with self.assertRaises(AssertionError):evidence.check_selection(raw,self.expected)
            with self.assertRaises(AssertionError):evidence.validate_run(receipt,runner)

    def test_case_start_cannot_be_discarded_by_summary_or_next_case(self):
        summary='test result: ok. 1 passed; 0 failed; 0 ignored;'
        bad=[
            self.good.replace(summary,'test interrupted ... \n'+summary),
            self.good.replace('test required ... ok','test interrupted ... \ntest required ... ok'),
            self.good+'Doc-tests example\nrunning 0 tests\ntest interrupted ... \ntest result: ok. 0 passed; 0 failed; 0 ignored;\n',
            # Declared room remains: the next start must still not overwrite
            # the pending case, independently of the count/summary checks.
            self.good.replace('running 1 test','running 2 tests').replace('test required ... ok','test interrupted ... \ntest required ... ok'),
            self.good.replace(summary,'ok\n'+summary),
            self.good.replace('running 1 test\n','running 1 test\nok\n'),
            self.good.replace(summary,'  test interrupted ... \n'+summary),
            self.good+'ok\n',
            self.good+'FAILED\n',
        ]
        for raw in bad:
            with self.subTest(raw=raw):self.check_both(raw,False)
        self.check_both(self.good,True)
        self.check_both('\n'.join('  '+line+'  'for line in self.good.splitlines()),True)

    def test_child_pending_parent_completion_and_case_identity(self):
        child='running 1 test\ntest portable_child ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n'
        nested=self.good.replace('test required ... ok','test required ... \n'+child+'ok')
        corrupt=[
            nested.replace('test portable_child ... ok','test lost ... \ntest portable_child ... ok'),
            nested.replace('test portable_child ... ok','test portable_child ... ok\ntest interrupted ... '),
            nested.replace(child+'ok',child),
            nested.replace(child+'ok',child+'ok\nok'),
            self.good.replace('running 1 test','running 2 tests').replace('test required ... ok','test required ... ok\ntest required ... ok'),
        ]
        for raw in corrupt:
            with self.subTest(raw=raw):self.check_both(raw,False)
        self.assertEqual(self.check_both(nested,True)[0]['ids'],['required'])
        # Repeated and deeper child processes do not consume their parent's
        # case completion or contribute extra top-level selected IDs.
        repeated=self.good.replace('test required ... ok','test required ... \n'+child+child+'ok')
        deeper=nested.replace('test portable_child ... ok','test portable_child ... \n'+child+'ok')
        for raw in [repeated,deeper]:
            self.assertEqual(self.check_both(raw,True)[0]['ids'],['required'])
        multi=self.good.replace('running 1 test','running 2 tests').replace('test required ... ok','test required ... ok\ntest added ... ok').replace('1 passed','2 passed')
        self.assertEqual(self.check_both(multi,True)[0]['ids'],['required','added'])
        doc='Doc-tests example\nrunning 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;\n'
        self.assertEqual([len(b['ids'])for b in self.check_both(self.good+doc,True)],[1,0])

    def test_unfinished_entrypoints_and_unframed_results_reject(self):
        for tail in [
            'Running tests/another.rs (target/debug/deps/another.exe)\n',
            'Running tests/another.rs (target/debug/deps/another.exe)\nrunning 1 test\n',
            'Running tests/another.rs (target/debug/deps/another.exe)\nrunning 1 test\ntest extra ... ok\n',
            'Doc-tests example\n',
            'Running tests/one.rs\nRunning tests/two.rs\n',
            'test result: FAILED. 0 passed; 1 failed; 0 ignored;\n',
            'test result: ok. 0 passed; 0 failed; 0 ignored;\n',
            'test lost ... FAILED\n',
            'running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;\n',
        ]:
            with self.subTest(tail=tail),self.assertRaises(AssertionError):
                evidence.check_selection(self.good+tail,self.expected)
        evidence.check_selection(self.good,self.expected)

    def test_completed_new_binary_empty_doctest_and_nested_child_keep_denominator(self):
        extra='Running tests/another.rs (target/debug/deps/another.exe)\nrunning 1 test\ntest added ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\n'
        doc='Doc-tests example\nrunning 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;\n'
        blocks=evidence.check_selection(self.good+extra+doc,self.expected)
        self.assertEqual([len(b['ids'])for b in blocks],[1,1,0])
        nested=self.good.replace('test required ... ok','test required ... \nrunning 1 test\ntest portable_child ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\nok')
        self.assertEqual(evidence.check_selection(nested,self.expected)[0]['ids'],['required'])
        for raw in [nested.replace('test portable_child ... ok','test portable_child ... ignored'),self.good.replace(' ... ok',' ... ignored')]:
            with self.assertRaises(AssertionError):evidence.check_selection(raw,self.expected)


class GeneratedCredentialTests(unittest.TestCase):
    def test_all_versions_lf_crlf_actual_credentials_and_restoration(self):
        supplied=os.environ.get('M2_GENERATED_FIXTURE')
        self.assertTrue(supplied,'provide actual --generated output via M2_GENERATED_FIXTURE')
        with tempfile.TemporaryDirectory() as temp:
            folder=pathlib.Path(temp)/'generated';shutil.copytree(supplied,folder)
            path=folder/'generated.json';original=path.read_bytes();records=baseline.generated(folder)
            self.assertEqual(len(records),10)
            # Every API and newline variant, not only the first/v1 record.
            for index,record in enumerate(records):
                bad=json.loads(original);bad[index]['snapshot']['api_hash']='0'*64
                path.write_text(json.dumps(bad))
                try:
                    with self.subTest(record=record['name']),self.assertRaisesRegex(AssertionError,'original API credential'):
                        baseline.generated(folder)
                finally:path.write_bytes(original)
                baseline.generated(folder)
            baseline.generated_controls(folder)


class SourceClosureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        artifacts=os.environ.get('M2_TEST_ARTIFACTS')
        cls.temp=None
        if artifacts:
            pathlib.Path(artifacts).mkdir(parents=True,exist_ok=True)
            cls.folder=pathlib.Path(tempfile.mkdtemp(prefix='source-',dir=artifacts))
        else:
            cls.temp=tempfile.TemporaryDirectory();cls.folder=pathlib.Path(cls.temp.name)
        identity=json.loads((ROOT/f'research/plan{evidence.CURRENT}-identity.json').read_text())
        for rel in [*identity['files'],f'autographics-auto-driver/research/plan{evidence.CURRENT}-identity.json']:
            target=cls.folder/'inputs'/rel;target.parent.mkdir(parents=True,exist_ok=True)
            shutil.copyfile(ROOT.parent/rel,target)
        cls.driver=cls.folder/'inputs'/ROOT.name
        shutil.copytree(ROOT/'.sdk-source',cls.driver/'.sdk-source',ignore=shutil.ignore_patterns('target','__pycache__'))
        cls.records=[]

    @classmethod
    def tearDownClass(cls):
        (cls.folder/'results.json').write_text(json.dumps(cls.records,indent=2))
        if cls.temp:cls.temp.cleanup()

    def invoke(self,label,script,*args):
        command=[sys.executable,str(self.driver/'tools'/script),*map(str,args)]
        result=subprocess.run(command,cwd=self.driver,capture_output=True,text=True,timeout=60)
        self.records.append(dict(case=label,command=command,exit=result.returncode,stdout=result.stdout,stderr=result.stderr))
        return result

    def test_required_current_manifest_and_verifier_cannot_fall_back(self):
        for relative in [f'research/plan{evidence.CURRENT}-identity.json',f'tools/verify_plan{evidence.CURRENT}.py']:
            path=self.driver/relative;original=path.read_bytes()
            dest=self.folder/('missing-'+path.name)
            path.unlink()
            try:
                for script,args in [('verify_source.py',()),('package_consumer.py',(dest,))]:
                    result=self.invoke('missing-'+relative+'-'+script,script,*args)
                    self.assertNotEqual(result.returncode,0)
                    self.assertIn(path.name,result.stderr)
                self.assertFalse(dest.exists())
            finally:path.write_bytes(original)
        restored=self.invoke('restored-current-entry','verify_source.py')
        self.assertEqual(restored.returncode,0,restored.stderr)

    def test_root_build_config_and_auto_targets_reject_before_export_then_restore(self):
        for script in [f'verify_plan{evidence.CURRENT}.py','verify_source.py']:
            accepted=self.invoke('accepted-'+script,script);self.assertEqual(accepted.returncode,0,accepted.stderr)
        cases=[
            ('driver-build','autographics-auto-driver/build.rs','fn main(){panic!("undeclared build input")}'),
            ('driver-config','autographics-auto-driver/.cargo/config.toml','invalid Cargo TOML !'),
            ('agp-config','autographics-prototype/.cargo/config','invalid Cargo TOML !'),
            ('shared-config','.cargo/config.toml','invalid Cargo TOML !'),
            ('driver-example','autographics-auto-driver/examples/extra.rs','fn main(){}'),
            ('agp-bench','autographics-prototype/benches/extra.rs','fn main(){}'),
            ('extra-src','autographics-auto-driver/src/unlisted.rs','// unlisted source'),
        ]
        for label,relative,content in cases:
            with self.subTest(case=label):
                path=self.folder/'inputs'/relative;self.assertFalse(path.exists())
                created_parent=not path.parent.exists();path.parent.mkdir(parents=True,exist_ok=True);path.write_text(content)
                dest=self.folder/('rejected-export-'+label)
                try:
                    for script,args in [(f'verify_plan{evidence.CURRENT}.py',()),('verify_source.py',()),('package_consumer.py',(dest,))]:
                        result=self.invoke(label+'-'+script,script,*args)
                        self.assertNotEqual(result.returncode,0)
                        self.assertIn('source set drift',result.stderr)
                    self.assertFalse(dest.exists(),'rejected input must not create an export')
                finally:
                    path.unlink()
                    if created_parent:path.parent.rmdir()
        dest=self.folder/'accepted-export'
        restored=self.invoke('restored-export','package_consumer.py',dest)
        self.assertEqual(restored.returncode,0,restored.stderr)
        consumer=dest/'experimental'/ROOT.name
        result=subprocess.run([sys.executable,str(consumer/'tools/verify_source.py')],cwd=consumer,capture_output=True,text=True,timeout=60)
        self.records.append(dict(case='actual-restored-consumer-source',exit=result.returncode,stdout=result.stdout,stderr=result.stderr))
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertTrue((dest/'experimental/autographics-prototype/build.rs').is_file(),'legitimate AGP build input must export')
        self.assertFalse((consumer/'target').exists(),'source export cannot depend on producer target')


if __name__=='__main__':unittest.main()
