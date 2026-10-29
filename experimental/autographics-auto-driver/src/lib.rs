use auto_lang_core::{
    create_vm_from_source,
    vm::{
        engine::{AutoVM, CpuSliceBudget, ParkedSegment, SegmentOutcome},
        task::AutoTask,
    },
};
use std::time::{Duration, Instant};
#[derive(Clone, Copy, Debug)]
pub enum Scalar {
    Int(i32),
    Float(f32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultKind {
    Int,
    Float,
    Unit,
}
#[derive(Debug)]
pub enum Progress {
    Pending,
    Complete(Option<Scalar>),
}
struct Pending {
    seg: ParkedSegment,
    kind: ResultKind,
    base: usize,
    slices: usize,
    started: Instant,
}
/// Thread-confined VM owner. Only completed callbacks expose a value.
/// Fault/cancel destroys the entire VM/task; consumers retain their last-good output.
pub struct Session {
    signatures: std::collections::BTreeMap<String, (Vec<Option<ResultKind>>, Option<ResultKind>)>,
    vm: Option<AutoVM>,
    task: Option<AutoTask>,
    pending: Option<Pending>,
    pub compile_us: u128,
    pub calls: u64,
    pub yielded: u64,
    /// Bounded scalar observations; retain no VM, task or payload owner.
    pub max_call_steps: u64,
    pub max_call_slices: usize,
}
impl Session {
    pub fn new(source: &str) -> Result<Self, String> {
        Self::new_with_diagnostics(source, |stage, e| format!("{stage}: {e:?}"))
    }
    pub fn new_with_diagnostics(
        source: &str,
        map: impl Fn(&str, &auto_lang_core::error::AutoError) -> String,
    ) -> Result<Self, String> {
        let now = Instant::now();
        use auto_lang_core::{
            ast::{Stmt, Type},
            parser::Parser,
        };
        let ast = Parser::from(source)
            .parse()
            .map_err(|e| map("signature parse", &e))?;
        let scalar = |ty: &Type| match ty {
            Type::Int => Some(ResultKind::Int),
            Type::Float => Some(ResultKind::Float),
            Type::Void | Type::Unknown => Some(ResultKind::Unit),
            _ => None,
        };
        let signatures = ast
            .stmts
            .iter()
            .filter_map(|s| {
                if let Stmt::Fn(f) = s {
                    Some((
                        f.name.to_string(),
                        (
                            f.params.iter().map(|p| scalar(&p.ty)).collect(),
                            scalar(&f.ret),
                        ),
                    ))
                } else {
                    None
                }
            })
            .collect();
        let (vm, _, _, _) = create_vm_from_source(source).map_err(|e| map("compile", &e))?;
        auto_lang_core::vm::native_registry::register_builtin_natives();
        Ok(Self {
            signatures,
            vm: Some(vm),
            task: Some(AutoTask::new(1, 65536, 0)),
            pending: None,
            compile_us: now.elapsed().as_micros(),
            max_call_steps: 0,
            max_call_slices: 0,
            calls: 0,
            yielded: 0,
        })
    }
    fn budget() -> CpuSliceBudget {
        CpuSliceBudget {
            max_steps: 1000,
            max_duration: Duration::from_millis(2),
            clock_check_every: 64,
            cumulative_steps: 100_000,
        }
    }
    pub fn closed(&self) -> bool {
        self.vm.is_none()
    }
    pub fn close(&mut self) {
        self.signatures.clear();
        self.pending = None;
        self.task = None;
        self.vm = None;
    }
    pub fn start(
        &mut self,
        name: &str,
        args: &[Scalar],
        kind: ResultKind,
    ) -> Result<Progress, String> {
        if self.closed() {
            return Err("closed".into());
        }
        if self.pending.is_some() {
            return Err("callback already running".into());
        }
        let validation = (|| {
            let (params, ret) = self
                .signatures
                .get(name)
                .ok_or_else(|| format!("function '{name}' not found"))?;
            if *ret != Some(kind) {
                return Err("declared return type mismatch".to_string());
            }
            if params.len() != args.len()
                || params.iter().zip(args).any(|(ty, v)| {
                    *ty != Some(match v {
                        Scalar::Int(_) => ResultKind::Int,
                        Scalar::Float(_) => ResultKind::Float,
                    })
                })
            {
                return Err("argument type/count mismatch".to_string());
            }
            Ok(())
        })();
        if let Err(e) = validation {
            self.close();
            return Err(e);
        }
        if args.len() > 4 {
            return Err("argument count exceeds profile".into());
        }
        if args
            .iter()
            .any(|x| matches!(x,Scalar::Float(f) if !f.is_finite()))
        {
            return Err("nonfinite argument".into());
        }
        let vm = self.vm.as_ref().ok_or("closed")?;
        let t = self.task.as_mut().ok_or("closed")?;
        let base = t.ram.sp;
        t.cpu_steps_total = 0;
        for a in args {
            match a {
                Scalar::Int(i) => t.ram.push_i32(*i),
                Scalar::Float(f) => t.ram.push_f32(*f),
            }
        }
        let o = vm.call_fn_by_name_cpu_slice(t, name, args.len(), Self::budget());
        self.calls += 1;
        self.finish(o, kind, base, 0, Instant::now())
    }
    fn finish(
        &mut self,
        o: SegmentOutcome,
        kind: ResultKind,
        base: usize,
        slices: usize,
        started: Instant,
    ) -> Result<Progress, String> {
        self.max_call_steps = self
            .max_call_steps
            .max(self.task.as_ref().map_or(0, |t| t.cpu_steps_total as u64));
        self.max_call_slices = self.max_call_slices.max(slices + 1);
        let result = match o {
            SegmentOutcome::Completed(Ok(())) => {
                let t = self.task.as_mut().unwrap();
                if t.ram.sp != base + 1 {
                    Err(format!(
                        "return stack imbalance: {} != {}",
                        t.ram.sp,
                        base + 1
                    ))
                } else {
                    let nv = t.ram.pop_nv();
                    match kind {
                        ResultKind::Unit => Ok(Progress::Complete(None)),
                        ResultKind::Float if auto_val::is_f32(nv) => {
                            let f = auto_val::decode_f32(nv);
                            if f.is_finite() {
                                Ok(Progress::Complete(Some(Scalar::Float(f))))
                            } else {
                                Err("nonfinite return".into())
                            }
                        }
                        ResultKind::Int if auto_val::is_i32(nv) => Ok(Progress::Complete(Some(
                            Scalar::Int(auto_val::decode_i32(nv)),
                        ))),
                        _ => Err(format!("return type mismatch: {nv:016x}")),
                    }
                }
            }
            SegmentOutcome::Completed(Err(e)) => Err(format!("call: {e:?}")),
            SegmentOutcome::Parked { .. } => Err("unsupported-wait".into()),
            SegmentOutcome::Runnable { seg } => {
                self.yielded += 1;
                if slices >= 120 || started.elapsed() > Duration::from_millis(250) {
                    Err("callback budget exhausted".into())
                } else {
                    self.pending = Some(Pending {
                        seg,
                        kind,
                        base,
                        slices: slices + 1,
                        started,
                    });
                    Ok(Progress::Pending)
                }
            }
        };
        if result.is_err() {
            self.close()
        }
        result
    }
    pub fn pump(&mut self) -> Result<Progress, String> {
        let p = self.pending.take().ok_or("no pending callback")?;
        let o = self.vm.as_ref().unwrap().resume_fn_by_name_cpu_slice(
            self.task.as_mut().unwrap(),
            &p.seg,
            Self::budget(),
        );
        self.finish(o, p.kind, p.base, p.slices, p.started)
    }
    pub fn call(
        &mut self,
        name: &str,
        args: &[Scalar],
        kind: ResultKind,
    ) -> Result<Option<Scalar>, String> {
        let mut p = self.start(name, args, kind)?;
        loop {
            match p {
                Progress::Pending => p = self.pump()?,
                Progress::Complete(v) => return Ok(v),
            }
        }
    }
    pub fn int(&mut self, name: &str, args: &[Scalar]) -> Result<i32, String> {
        match self.call(name, args, ResultKind::Int)? {
            Some(Scalar::Int(v)) => Ok(v),
            _ => Err("expected int".into()),
        }
    }
    pub fn float(&mut self, name: &str, args: &[Scalar]) -> Result<f32, String> {
        match self.call(name, args, ResultKind::Float)? {
            Some(Scalar::Float(v)) => Ok(v),
            _ => Err("expected float".into()),
        }
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.close()
    }
}
pub mod output;
impl Session {
    pub fn heap_objects(&self) -> usize {
        self.vm.as_ref().map_or(0, |vm| vm.heap_object_count())
    }
    /// Count of scalar globals whose name starts with the given prefix.
    /// Research-only state-bound accounting (PLAN-860 probe units).
    pub fn globals_with_prefix(&self, prefix: &str) -> usize {
        self.vm.as_ref().map_or(0, |vm| {
            vm.globals
                .iter()
                .filter(|e| e.key().starts_with(prefix))
                .count()
        })
    }
    pub fn owner_probe(&self) -> impl Fn() -> bool {
        let vm = self.vm.as_ref().expect("open session");
        let strings = std::sync::Arc::downgrade(&vm.strings);
        let heaps: Vec<_> = vm
            .heap_objects
            .iter()
            .map(|v| std::sync::Arc::downgrade(v.value()))
            .collect();
        move || strings.upgrade().is_some() || heaps.iter().any(|v| v.upgrade().is_some())
    }
}
pub mod consumer;
pub mod host;

#[cfg(test)]
mod segment_memory_measurement {
    use super::*;
    use crate::authoring::behavior::{Api, SourceSnapshot};
    use std::path::Path;
    #[test]
    fn motion_slot_real_storage_and_owner_boundary() {
        let snap = SourceSnapshot::from_bytes_for(
            Path::new("motion-slot.at"),
            b"fn app_start(){}\nfn app_update(dt float){}\n".to_vec(),
            Api::MotionV5,
        )
        .unwrap();
        let mut s = Session::new(&snap.source).unwrap();
        s.call("main", &[], ResultKind::Unit).unwrap();
        let alive = s.owner_probe();
        let vm = s.vm.as_ref().unwrap();
        let keys: Vec<_> = vm
            .globals
            .iter()
            .filter(|e| {
                e.key().starts_with("_ag_cast_")
                    || e.key().starts_with("_ag_motion_")
                    || e.key().starts_with("_ag_segment_d")
                    || ["_ag_query_kind", "_ag_sweep_overlap", "_ag_spatial_calls"]
                        .contains(&e.key().as_str())
            })
            .collect();
        assert!(keys.len() <= 40);
        eprintln!(
            "MOTION_SLOT {}",
            serde_json::json!({"scalar_count":keys.len(),"logical_bytes":keys.len()*4,"nanovalue_bytes":keys.len()*std::mem::size_of::<auto_val::NanoValue>(),"key_struct_bytes":keys.len()*std::mem::size_of::<String>(),"key_capacity_bytes":keys.iter().map(|e|e.key().capacity()).sum::<usize>(),"shared_map_capacity":vm.globals.capacity(),"shared_map_struct_bytes":std::mem::size_of_val(&vm.globals),"managed_bytes":snap.source.len()-snap.bytes.len(),"boundary":"allocator/program/stack and shared map separate; 160 logical bytes is not total SDK bytes"})
        );
        drop(keys);
        s.close();
        assert!(!alive());
    }
    #[test]
    fn sweep_slot_real_scalar_storage_and_owner_boundary() {
        let snap = SourceSnapshot::from_bytes_for(
            Path::new("sweep-slot.at"),
            b"fn app_start(){}\nfn app_update(dt float){}\n".to_vec(),
            Api::SweepV4,
        )
        .unwrap();
        let mut s = Session::new(&snap.source).unwrap();
        s.call("main", &[], ResultKind::Unit).unwrap();
        let alive = s.owner_probe();
        {
            let vm = s.vm.as_ref().unwrap();
            let keys: Vec<_> = vm
                .globals
                .iter()
                .filter(|e| {
                    e.key().starts_with("_ag_cast_")
                        || e.key().starts_with("_ag_segment_d")
                        || ["_ag_query_kind", "_ag_sweep_overlap", "_ag_spatial_calls"]
                            .contains(&e.key().as_str())
                })
                .collect();
            assert_eq!(keys.len(), 18);
            assert!(keys.len() * 4 <= 128);
            eprintln!(
                "SWEEP_SLOT {}",
                serde_json::json!({"scalar_count":keys.len(),"logical_bytes":keys.len()*4,"nanovalue_bytes":keys.len()*std::mem::size_of::<auto_val::NanoValue>(),"key_struct_bytes":keys.len()*std::mem::size_of::<String>(),"key_capacity_bytes":keys.iter().map(|e|e.key().capacity()).sum::<usize>(),"shared_map_capacity":vm.globals.capacity(),"shared_map_struct_bytes":std::mem::size_of_val(&vm.globals),"boundary":"Map allocator/control, compiled program, stack and container overhead remain separate; logical128B is not total SDK residency."})
            );
        }
        s.close();
        assert!(!alive());
    }

    #[test]
    fn result_slot_reports_real_sdk_value_and_key_storage_separately() {
        for api in [Api::V1, Api::SpatialV2, Api::SegmentV3] {
            let source = SourceSnapshot::from_bytes_for(
                Path::new("slot.at"),
                b"fn app_start(){}\nfn app_update(dt float){}\n".to_vec(),
                api,
            )
            .unwrap();
            let mut s = Session::new(&source.source).unwrap();
            s.call("main", &[], ResultKind::Unit).unwrap();
            let vm = s.vm.as_ref().unwrap();
            let keys: Vec<_> = vm
                .globals
                .iter()
                .filter(|e| e.key().starts_with("_ag_cast_"))
                .collect();
            assert_eq!(keys.len(), if api == Api::SegmentV3 { 11 } else { 0 });
            let key_capacity: usize = keys.iter().map(|e| e.key().capacity()).sum();
            let dirty: Vec<_> = vm
                .globals
                .iter()
                .filter(|e| {
                    [
                        "_ag_segment_dx",
                        "_ag_segment_dy",
                        "_ag_segment_dz",
                        "_ag_segment_dv",
                    ]
                    .contains(&e.key().as_str())
                })
                .collect();
            assert_eq!(dirty.len(), if api == Api::SegmentV3 { 4 } else { 0 });
            eprintln!("DIRTY_STORAGE api={} scalars={} logical={} nanovalue={} key_struct={} key_capacity={}",api.version(),dirty.len(),dirty.len()*4,dirty.len()*std::mem::size_of::<auto_val::NanoValue>(),dirty.len()*std::mem::size_of::<String>(),dirty.iter().map(|e|e.key().capacity()).sum::<usize>());

            eprintln!(
                "{}",
                serde_json::json!({"api":api.version(),"result_scalar_globals":keys.len(),"logical_payload_bytes":keys.len()*4,"sdk_nanovalue_bytes":keys.len()*std::mem::size_of::<auto_val::NanoValue>(),"sdk_key_struct_bytes":keys.len()*std::mem::size_of::<String>(),"sdk_owned_key_capacity_bytes":key_capacity,"whole_shared_global_map_capacity":vm.globals.capacity(),"whole_shared_global_map_struct_bytes":std::mem::size_of_val(&vm.globals),"heap_objects_after_main":vm.heap_object_count(),"boundary":"Key capacities and value/key element sizes are measured separately. Shared map allocation/control bytes, compiled program, task stack and allocator/driver residency are not falsely charged as a portable exact incremental total."})
            );
        }
    }
}
pub mod script;
#[cfg(test)]
mod terminal_policy_tests {
    use super::*;
    #[test]
    fn parked_outcome_closes_task_vm_and_pending_state() {
        use auto_lang_core::vm::engine::ParkedWait;
        let mut s = Session::new("fn main() {}").unwrap();
        let alive = s.owner_probe();
        let outcome = SegmentOutcome::Parked {
            wait: ParkedWait::Future(1),
            seg: ParkedSegment {
                fn_name: "unsupported".into(),
                saved_bp: 0,
                saved_fn_n_args: 0,
            },
        };
        assert!(s
            .finish(outcome, ResultKind::Unit, 0, 0, Instant::now())
            .unwrap_err()
            .contains("unsupported-wait"));
        assert!(s.closed());
        assert!(!alive());
    }
    #[test]
    fn owner_oracle_detects_a_retained_last_owner() {
        let mut s = Session::new("fn main() {}").unwrap();
        let pin = s.vm.as_ref().unwrap().strings.clone();
        let alive = s.owner_probe();
        s.close();
        assert!(alive());
        drop(pin);
        assert!(!alive());
    }
}

pub mod squash;

pub mod authoring;
pub mod research;
pub mod squash_host;
