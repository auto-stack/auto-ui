//! PLAN-083 T-03：daemon 跨进程启动协调（D-04）。
//!
//! 目标（Windows 本地共享服务）：两个独立宿主同时冷启同一声明 daemon 时，
//! **只允许 spawn 一次**；输家有限等待后复用已验证实例；失败/进程退出/
//! 超时释放所有权；不同数据根与相同端点必须具名拒绝；普通端口占用仍拒绝。
//!
//! 机制（不引入新依赖）：
//! - **互斥**：Windows 以 `share_mode(0)` 独占打开锁文件（等价 named mutex 的
//!   单写者语义；进程退出由 OS 关闭句柄自动释放，无陈旧锁）。非 Windows
//!   回退 `create_new` + Drop 删除 + 陈旧 TTL（Windows 是合同平台）。
//! - **身份**：`user + instance_key + port + 规范化 data_root` 的 blake3 短哈希
//!   （文件名）与完整规范化串（标记文件内容）；仅靠路径文本别名或进程内
//!   `DID_SPAWN` 标记不算身份。
//! - **实例标记**：赢家在就绪后写 `{instance_key, data_root, port, pid,
//!   started_at}`；输家复用前核对标记——端点相同但身份不同 → 具名冲突拒绝。
//!   标记缺席（例如开发者手工启动的服务）只允许在健康接口已核验下复用，
//!   并记录 owner 未知。
//!
//! 与 [`super::osconfig_daemon`] 的关系：本模块只做"谁可以 spawn"的门，
//! 健康核验/就绪预算/接口合同全部复用 [`super::osconfig_daemon::ensure_generic_io`]
//! 族语义（同 [`GenericDaemon`] 定义）。

use std::path::{Path, PathBuf};

use super::osconfig_daemon::{
    DaemonIo, DaemonStatus, GenericDaemon, HealthSpec, HealthVerdict,
};

/// 跨进程身份（身份键 + 数据根 + 端点 + 用户）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceIdentity {
    /// 受管实例键（manifest `daemon.instance_key`，缺席 = 条目 id）。
    pub instance_key: String,
    /// 规范化数据根（`%VAR%`/`$VAR` 已展开；`None` = 未声明）。
    pub data_root: Option<PathBuf>,
    /// 端点端口（同端点必须同实例身份）。
    pub port: u16,
    /// 当前用户（不同用户不共享本地实例范围）。
    pub user: String,
}

impl InstanceIdentity {
    /// 从声明组装：`instance_key` 缺省 = daemon 名；数据根按进程环境展开。
    pub fn from_declaration(
        name: &str,
        port: u16,
        instance_key: Option<&str>,
        data_root: Option<&str>,
    ) -> Self {
        Self {
            instance_key: instance_key
                .map(str::trim)
                .filter(|key| !key.is_empty())
                .unwrap_or(name)
                .to_string(),
            data_root: data_root
                .map(|raw| PathBuf::from(super::osconfig_daemon::expand_env_path(raw))),
            port,
            user: current_user(),
        }
    }

    /// 规范化身份串（标记/锁同源；避免大小写与分隔符别名）。
    pub fn normalized(&self) -> String {
        let root = self
            .data_root
            .as_deref()
            .map(normalize_path_text)
            .unwrap_or_else(|| "<none>".to_string());
        format!(
            "user={}|key={}|port={}|root={}",
            self.user, self.instance_key, self.port, root
        )
    }

    /// 身份短哈希（锁/标记文件名；blake3 已在本 crate 依赖内）。
    pub fn identity_hash(&self) -> String {
        blake3::hash(self.normalized().as_bytes()).to_hex()[..16].to_string()
    }

    /// 实例范围目录（`AUTO_UI_DAEMON_SCOPE_DIR` 显式覆盖 >
    /// `%LOCALAPPDATA%/auto-ui/daemon-instances` > 临时目录）。测试/运维可
    /// 用显式覆盖隔离，不污染真实用户范围。
    pub fn scope_dir(&self) -> PathBuf {
        if let Some(explicit) = std::env::var_os("AUTO_UI_DAEMON_SCOPE_DIR") {
            return PathBuf::from(explicit);
        }
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        base.join("auto-ui").join("daemon-instances")
    }

    /// 锁文件路径——**按端口**（端点互斥）：同端点不同数据根/实例键也必须
    /// 先争同一把锁，赢家写标记，输家读标记后具名拒绝（不能各起一份）。
    pub fn lock_path(&self) -> PathBuf {
        self.scope_dir()
            .join(format!("port-{}.lock", self.port))
    }

    /// 实例标记路径——**按端口**（同端点唯一 owner；身份差异由标记内容判定）。
    pub fn marker_path(&self) -> PathBuf {
        self.scope_dir()
            .join(format!("port-{}.instance.json", self.port))
    }
}

/// 当前用户名（身份组成；缺省 `unknown`——同一用户范围内协调）。
fn current_user() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .map(|user| user.trim().to_ascii_lowercase())
        .ok()
        .filter(|user| !user.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn normalize_path_text(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

/// 实例标记（赢家写；输家核对身份）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InstanceMarker {
    pub instance_key: String,
    #[serde(default)]
    pub data_root: Option<String>,
    pub port: u16,
    pub pid: u32,
    pub started_at: u64,
}

impl InstanceMarker {
    /// 以自身进程身份写标记（就绪后调用）。
    pub fn write_for(identity: &InstanceIdentity) -> Result<Self, String> {
        let marker = Self {
            instance_key: identity.instance_key.clone(),
            data_root: identity
                .data_root
                .as_deref()
                .map(|path| normalize_path_text(path)),
            port: identity.port,
            pid: std::process::id(),
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_secs())
                .unwrap_or(0),
        };
        let path = identity.marker_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("实例标记目录不可建（{}）：{err}", parent.display()))?;
        }
        let body = serde_json::to_vec(&marker).map_err(|err| format!("实例标记序列化失败：{err}"))?;
        std::fs::write(&path, body)
            .map_err(|err| format!("实例标记写入失败（{}）：{err}", path.display()))?;
        Ok(marker)
    }

    /// 读取（缺席/损坏 → None；不把坏标记当身份）。
    pub fn read(identity: &InstanceIdentity) -> Option<Self> {
        let raw = std::fs::read(identity.marker_path()).ok()?;
        serde_json::from_slice(&raw).ok()
    }

    /// 标记与期望身份是否一致（数据根按规范化文本比较）。
    pub fn matches(&self, identity: &InstanceIdentity) -> bool {
        let expected_root = identity
            .data_root
            .as_deref()
            .map(normalize_path_text);
        self.instance_key == identity.instance_key
            && self.port == identity.port
            && self.data_root == expected_root
    }

    /// 非敏感描述（日志用）。
    pub fn describe(&self) -> String {
        format!(
            "instance_key={} port={} data_root={} pid={}",
            self.instance_key,
            self.port,
            self.data_root.as_deref().unwrap_or("<none>"),
            self.pid
        )
    }
}

/// 跨进程启动锁（Drop 释放；Windows 由句柄关闭释放）。
#[derive(Debug)]
pub struct InstanceLock {
    path: PathBuf,
    file: std::fs::File,
    remove_on_drop: bool,
}

/// 取锁结果。
#[derive(Debug)]
pub enum LockOutcome {
    Acquired(InstanceLock),
    /// 另一进程持有（输家路径：有限等待 + 复用已验证实例）。
    Busy,
    /// 文件系统错误（具名；调用方 Offline 有因）。
    Io(String),
}

impl InstanceLock {
    /// 非阻塞取锁。
    pub fn try_acquire(identity: &InstanceIdentity) -> LockOutcome {
        let path = identity.lock_path();
        if let Some(parent) = path.parent() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                return LockOutcome::Io(format!(
                    "锁目录不可建（{}）：{err}",
                    parent.display()
                ));
            }
        }
        match open_exclusive(&path) {
            Ok((file, created_new)) => {
                // 写入持有人标记（诊断用；不参与身份判定）。
                if let Ok(mut handle) = file.try_clone() {
                    use std::io::Write;
                    let holder = format!(
                        "pid={} identity={}\n",
                        std::process::id(),
                        identity.normalized()
                    );
                    let _ = handle.write_all(holder.as_bytes());
                }
                LockOutcome::Acquired(InstanceLock {
                    path,
                    file,
                    remove_on_drop: created_new,
                })
            }
            Err(OpenError::Busy) => LockOutcome::Busy,
            Err(OpenError::Io(err)) => LockOutcome::Io(format!(
                "锁文件打开失败（{}）：{err}",
                path.display()
            )),
        }
    }

    /// 锁文件路径（诊断）。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 显式释放（Drop 同效；失败路径提前释放以便下一次冷启）。
    pub fn release(self) {
        drop(self);
    }

    /// 句柄（测试/诊断；持有时长即所有权）。
    pub fn holder_pid(&self) -> Option<u32> {
        let raw = std::fs::read_to_string(&self.path).ok()?;
        raw.trim()
            .strip_prefix("pid=")?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        // Windows：句柄关闭即释放（不删文件，避免删除/重建竞态）。
        // 非 Windows 回退：删除自建锁文件（否则下次取锁必 Busy）。
        #[cfg(not(windows))]
        {
            if self.remove_on_drop {
                let _ = std::fs::remove_file(&self.path);
            }
        }
        #[cfg(windows)]
        {
            let _ = &self.remove_on_drop;
        }
    }
}

enum OpenError {
    Busy,
    Io(std::io::Error),
}

/// Windows：`share_mode(0)` 独占打开（第二个打开者必失败）。文件已存在则
/// 复用同一 inode（不做删除/重建竞态）。
#[cfg(windows)]
fn open_exclusive(path: &Path) -> Result<(std::fs::File, bool), OpenError> {
    use std::os::windows::fs::OpenOptionsExt;
    let existed = path.exists();
    match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .share_mode(0)
        .open(path)
    {
        Ok(file) => Ok((file, !existed)),
        Err(err) => {
            // ERROR_SHARING_VIOLATION(32) / ERROR_LOCK_VIOLATION(33)
            if matches!(err.raw_os_error(), Some(32) | Some(33)) {
                Err(OpenError::Busy)
            } else {
                Err(OpenError::Io(err))
            }
        }
    }
}

/// 非 Windows 回退：`create_new` 原子创建 + 陈旧 TTL 回收（Windows 是
/// 合同平台；此臂只保证开发机可测）。
#[cfg(not(windows))]
fn open_exclusive(path: &Path) -> Result<(std::fs::File, bool), OpenError> {
    match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => Ok((file, true)),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            // 陈旧回收：存在超过 120s 视为崩溃遗留（仅在无共享句柄可判定的平台）。
            let stale = std::fs::metadata(path)
                .and_then(|meta| meta.modified())
                .ok()
                .and_then(|modified| modified.elapsed().ok())
                .map(|age| age.as_secs() > 120)
                .unwrap_or(false);
            if stale {
                let _ = std::fs::remove_file(path);
                if let Ok(file) = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(path)
                {
                    return Ok((file, true));
                }
            }
            Err(OpenError::Busy)
        }
        Err(err) => Err(OpenError::Io(err)),
    }
}

/// 共享 daemon 协调参数。
#[derive(Debug, Clone)]
pub struct SharedEnsureOptions {
    /// 就绪预算（含等待赢家 + spawn 后轮询）。
    pub budget: std::time::Duration,
    /// 输家等待赢家的上限（≤ `budget`）。
    pub lock_wait: std::time::Duration,
}

impl Default for SharedEnsureOptions {
    fn default() -> Self {
        Self {
            budget: std::time::Duration::from_secs(5),
            lock_wait: std::time::Duration::from_secs(5),
        }
    }
}

/// 健康探测（与 ensure 同一合同；只探不孵）。经 [`DaemonIo::health`] 走
/// 注入的执行环境——真实路径即 [`super::osconfig_daemon::probe_health`]。
fn probe(io: &mut dyn DaemonIo, generic: &GenericDaemon, timeout: std::time::Duration) -> HealthVerdict {
    let spec = HealthSpec::declaring(&generic.url, generic.health_interface.as_deref(), timeout);
    io.health(&spec)
}

/// 复用前的身份核对：标记在场必须一致；缺席仅允许在健康已核验下复用。
fn verify_owner(identity: &InstanceIdentity) -> Result<Option<InstanceMarker>, String> {
    match InstanceMarker::read(identity) {
        Some(marker) if !marker.matches(identity) => Err(format!(
            "端口 {} 已被不同实例占用（本机期望 {}；在位 {}）——拒绝共享，需由 profile/实例键隔离",
            identity.port,
            identity.normalized(),
            marker.describe()
        )),
        other => Ok(other),
    }
}

/// 共享 daemon 的跨进程 ensure（spawn 至多一次）。
///
/// 冻结语义：
/// 1. 先探测：`Ready` 且标记一致（或缺席）→ 复用（不 spawn、不取锁）。
/// 2. 取锁：成功 → 锁内二次探测（他人可能刚就绪）→ 仍无监听才 spawn →
///    轮询同一接口 → 就绪后写实例标记 → 释放锁。
/// 3. 取锁失败（输家）→ 在 `lock_wait` 内有界轮询健康：`Ready` + 身份一致 →
///    复用；锁释放 → 重试取锁；预算耗尽 → 具名 Offline（最后裁决）。
/// 4. 失败/超时：回收本次写入的标记、释放锁（进程退出亦由 OS 释放）。
pub fn ensure_shared_io(
    generic: &GenericDaemon,
    identity: &InstanceIdentity,
    io: &mut dyn DaemonIo,
    options: &SharedEnsureOptions,
) -> DaemonStatus {
    let started = std::time::Instant::now();
    let deadline = started + options.budget;
    let lock_deadline = started + options.lock_wait;

    // 1. 快路径：已就绪（含手工/他人启动）→ 身份核对 → 复用。
    let first = probe(io, generic, std::time::Duration::from_millis(250));
    if matches!(first, HealthVerdict::Ready) {
        return match verify_owner(identity) {
            Ok(_) => DaemonStatus::Running(generic.url.clone()),
            Err(reason) => DaemonStatus::Offline(reason),
        };
    }

    // 2. 取锁 → 锁内二次检查 → 仅无监听才 spawn。
    let mut last = first;
    loop {
        match InstanceLock::try_acquire(identity) {
            LockOutcome::Acquired(lock) => {
                let outcome = ensure_under_lock(
                    generic,
                    identity,
                    io,
                    &mut last,
                    deadline,
                    options.budget,
                );
                // 锁在成功/失败后都释放：失败可让下一次冷启重试。
                drop(lock);
                return outcome;
            }
            LockOutcome::Io(reason) => return DaemonStatus::Offline(reason),
            LockOutcome::Busy => {
                // 3. 输家：有界等待 + 复用已验证实例。
                if std::time::Instant::now() >= lock_deadline {
                    return DaemonStatus::Offline(format!(
                        "daemon `{}` 跨进程启动等待超时（{}ms；最后裁决：{}）",
                        generic.name,
                        options.lock_wait.as_millis(),
                        last.reason()
                    ));
                }
                let verdict = probe(io, generic, std::time::Duration::from_millis(250));
                if matches!(verdict, HealthVerdict::Ready) {
                    return match verify_owner(identity) {
                        Ok(_) => DaemonStatus::Running(generic.url.clone()),
                        Err(reason) => DaemonStatus::Offline(reason),
                    };
                }
                last = verdict;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

/// 锁内路径：二次探测 → spawn → 同接口轮询 → 写标记。
fn ensure_under_lock(
    generic: &GenericDaemon,
    identity: &InstanceIdentity,
    io: &mut dyn DaemonIo,
    last: &mut HealthVerdict,
    deadline: std::time::Instant,
    budget: std::time::Duration,
) -> DaemonStatus {
    let verdict = probe(io, generic, std::time::Duration::from_millis(800));
    match verdict {
        HealthVerdict::Ready => {
            return match verify_owner(identity) {
                Ok(_) => DaemonStatus::Running(generic.url.clone()),
                Err(reason) => DaemonStatus::Offline(reason),
            }
        }
        HealthVerdict::NoListener => {}
        other => {
            // 锁内已确认占用者不是目标服务：不 spawn、不向其投递配置。
            *last = other.clone();
            return DaemonStatus::Offline(format!(
                "daemon `{}` 端口 {} 不可复用：{}（占用者存活，未 spawn）",
                generic.name,
                identity.port,
                other.reason()
            ));
        }
    }

    let Some(path) = &generic.bin else {
        return DaemonStatus::Offline(format!(
            "daemon `{}` 未配置 bin（apps.manifest daemon.bin 缺席，只探不孵；health {} 不通）",
            generic.name, generic.url
        ));
    };
    if let Err(err) = io.spawn(path, &generic.spawn_env()) {
        return DaemonStatus::Offline(format!(
            "daemon `{}` spawn {} 失败: {err}",
            generic.name,
            path.display()
        ));
    }

    // spawn 后按同一接口轮询（2xx 早到不算 ready；错接口立即拒绝）。
    let mut marker_written = false;
    let mut result: Option<String> = None;
    loop {
        let poll_verdict = probe(io, generic, std::time::Duration::from_millis(250));
        match poll_verdict {
            HealthVerdict::Ready => {
                // 就绪即写身份标记（失败不推翻就绪，但记录）。
                if let Err(err) = InstanceMarker::write_for(identity) {
                    eprintln!("[plan083] 实例标记写入失败（daemon `{}`）：{err}", generic.name);
                } else {
                    marker_written = true;
                }
                break;
            }
            HealthVerdict::WrongInterface {
                status,
                expected,
                actual,
            } => {
                result = Some(format!(
                    "daemon `{}` 就绪期接口不匹配（HTTP {status}，期望 {expected}，实际 {}；拒绝复用 {}）",
                    generic.name,
                    actual.as_deref().unwrap_or("<缺席>"),
                    path.display()
                ));
                break;
            }
            other => *last = other,
        }
        if std::time::Instant::now() >= deadline {
            result = Some(format!(
                "daemon `{}` 就绪超时（{}ms；最后裁决：{}；{}）",
                generic.name,
                budget.as_millis(),
                last.reason(),
                path.display()
            ));
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    match result {
        Some(reason) => {
            if marker_written {
                let _ = std::fs::remove_file(identity.marker_path());
            }
            DaemonStatus::Offline(reason)
        }
        None => DaemonStatus::Running(generic.url.clone()),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// 测试：锁互斥/身份/标记（进程内）+ 跨进程 U-03 见
// `crates/auto-ui-runtime/tests/featurepack_daemon_hosting.rs`。
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 实例范围目录是进程级环境变量——本模块测试串行化，避免并行互相覆盖。
    static SCOPE_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct ScopeGuard {
        root: PathBuf,
        _guard: std::sync::MutexGuard<'static, ()>,
    }

    impl Drop for ScopeGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
            std::env::remove_var("AUTO_UI_DAEMON_SCOPE_DIR");
        }
    }

    fn identity_fixture(tag: &str) -> (InstanceIdentity, ScopeGuard) {
        let guard = SCOPE_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = std::env::temp_dir().join(format!(
            "autoui-083-inst-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::env::set_var("AUTO_UI_DAEMON_SCOPE_DIR", &root);
        let identity = InstanceIdentity::from_declaration(
            "015-notes",
            17819,
            Some("notes:it"),
            Some(root.join("data").to_string_lossy().as_ref()),
        );
        (
            identity,
            ScopeGuard {
                root,
                _guard: guard,
            },
        )
    }

    #[test]
    fn identity_normalization_is_stable_and_distinct() {
        let root = std::env::temp_dir().join("autoui-083-identity");
        let a = InstanceIdentity::from_declaration(
            "015-notes",
            17819,
            Some("notes"),
            Some(root.join("Data").to_string_lossy().as_ref()),
        );
        let b = InstanceIdentity::from_declaration(
            "015-notes",
            17819,
            Some("notes"),
            Some(root.join("data/").to_string_lossy().as_ref()),
        );
        assert_eq!(a.normalized(), b.normalized(), "大小写/分隔符/尾斜杠不产生别名");
        let c = InstanceIdentity::from_declaration("015-notes", 17819, Some("notes"), None);
        assert_ne!(a.normalized(), c.normalized(), "数据根参与身份");
        let d = InstanceIdentity::from_declaration("015-notes", 17820, Some("notes"), None);
        assert_ne!(c.normalized(), d.normalized(), "端点参与身份");
        assert_eq!(a.identity_hash().len(), 16);
    }

    #[test]
    fn lock_is_exclusive_and_released_on_drop() {
        let (identity, _scope) = identity_fixture("lock");
        let first = match InstanceLock::try_acquire(&identity) {
            LockOutcome::Acquired(lock) => lock,
            other => panic!("首取应成功，实际 {other:?}"),
        };
        match InstanceLock::try_acquire(&identity) {
            LockOutcome::Busy => {}
            other => panic!("占用期第二次取锁必须 Busy，实际 {other:?}"),
        }
        first.release();
        match InstanceLock::try_acquire(&identity) {
            LockOutcome::Acquired(lock) => lock.release(),
            other => panic!("释放后可再取，实际 {other:?}"),
        }
    }

    #[test]
    fn marker_roundtrip_and_identity_match() {
        let (identity, _scope) = identity_fixture("marker");
        let marker = InstanceMarker::write_for(&identity).expect("写标记");
        assert_eq!(marker.pid, std::process::id());
        let read = InstanceMarker::read(&identity).expect("读标记");
        assert_eq!(read, marker);
        assert!(read.matches(&identity));
        // 同端点不同数据根：身份不匹配（拒绝共享的依据）。
        let other = InstanceIdentity::from_declaration(
            "015-notes",
            17819,
            Some("notes:it"),
            Some("D:/other-root"),
        );
        assert!(!marker.matches(&other));
        // 实例键不同也不匹配。
        let other_key = InstanceIdentity::from_declaration(
            "015-notes",
            17819,
            Some("notes:other"),
            identity.data_root.as_deref().map(|p| p.to_string_lossy().to_string()).as_deref(),
        );
        assert!(!marker.matches(&other_key));
    }

    /// 假 IO：健康脚本 + spawn 计数（验证只 spawn 一次/不 spawn）。
    struct ScriptedIo {
        script: Vec<HealthVerdict>,
        calls: usize,
        spawns: usize,
    }

    impl ScriptedIo {
        fn new(script: Vec<HealthVerdict>) -> Self {
            Self {
                script,
                calls: 0,
                spawns: 0,
            }
        }
    }

    impl DaemonIo for ScriptedIo {
        fn ping(&mut self, _url: &str) -> bool {
            false
        }
        fn spawn(&mut self, _path: &Path, _env: &[(String, String)]) -> Result<(), String> {
            self.spawns += 1;
            Ok(())
        }
        fn health(&mut self, _spec: &HealthSpec) -> HealthVerdict {
            let index = self.calls.min(self.script.len().saturating_sub(1));
            self.calls += 1;
            self.script[index].clone()
        }
    }

    fn generic_for(identity: &InstanceIdentity, bin: Option<PathBuf>) -> GenericDaemon {
        GenericDaemon {
            name: "015-notes".to_string(),
            url: format!("http://127.0.0.1:{}", identity.port),
            bin,
            env_key: "NOTES_DAEMON".to_string(),
            env_port_key: "FP_FIXTURE_PORT".to_string(),
            health_interface: Some("notes.fpack/1".to_string()),
            ready_timeout: None,
            extra_env: Vec::new(),
        }
    }

    #[test]
    fn shared_ensure_reuses_ready_instance_without_spawn() {
        let (identity, _scope) = identity_fixture("reuse");
        let _ = InstanceMarker::write_for(&identity);
        let generic = generic_for(&identity, Some(PathBuf::from("D:/fixture.exe")));
        let mut io = ScriptedIo::new(vec![HealthVerdict::Ready]);
        let status = ensure_shared_io(
            &generic,
            &identity,
            &mut io,
            &SharedEnsureOptions {
                budget: std::time::Duration::from_secs(1),
                lock_wait: std::time::Duration::from_secs(1),
            },
        );
        assert_eq!(status, DaemonStatus::Running(generic.url.clone()));
        assert_eq!(io.spawns, 0, "已就绪实例零打扰复用");
    }

    #[test]
    fn shared_ensure_rejects_conflicting_owner() {
        let (identity, _scope) = identity_fixture("conflict");
        // 标记属于不同数据根（同端点），健康 Ready 也不得复用。
        let other = InstanceIdentity::from_declaration(
            "015-notes",
            17819,
            Some("notes:it"),
            Some("D:/other-root"),
        );
        let marker = InstanceMarker::write_for(&other).expect("写他人标记");
        // 写在同一 identity 的标记路径下（模拟端点冲突的真实布局）。
        let mut marker = marker;
        marker.data_root = Some("d:/other-root".to_string());
        std::fs::write(
            identity.marker_path(),
            serde_json::to_vec(&marker).unwrap(),
        )
        .unwrap();
        let generic = generic_for(&identity, Some(PathBuf::from("D:/fixture.exe")));
        let mut io = ScriptedIo::new(vec![HealthVerdict::Ready]);
        let status = ensure_shared_io(
            &generic,
            &identity,
            &mut io,
            &SharedEnsureOptions::default(),
        );
        match status {
            DaemonStatus::Offline(reason) => {
                assert!(reason.contains("不同实例占用"), "{reason}");
                assert!(reason.contains("拒绝共享"), "{reason}");
            }
            other => panic!("身份冲突应 Offline，实际 {other:?}"),
        }
        assert_eq!(io.spawns, 0, "身份冲突不得 spawn");
    }

    #[test]
    fn shared_ensure_spawns_once_then_writes_marker() {
        let (identity, _scope) = identity_fixture("spawn");
        let generic = generic_for(&identity, Some(PathBuf::from("D:/fixture.exe")));
        let mut io = ScriptedIo::new(vec![
            HealthVerdict::NoListener, // 快路径
            HealthVerdict::NoListener, // 锁内二次探测
            HealthVerdict::NoListener, // spawn 后首轮
            HealthVerdict::Ready,      // 就绪
        ]);
        let status = ensure_shared_io(
            &generic,
            &identity,
            &mut io,
            &SharedEnsureOptions::default(),
        );
        assert_eq!(status, DaemonStatus::Running(generic.url.clone()));
        assert_eq!(io.spawns, 1, "冷启只 spawn 一次");
        let marker = InstanceMarker::read(&identity).expect("就绪后写标记");
        assert!(marker.matches(&identity));
    }

    #[test]
    fn shared_ensure_does_not_spawn_on_foreign_occupant() {
        let (identity, _scope) = identity_fixture("foreign");
        let generic = generic_for(&identity, Some(PathBuf::from("D:/fixture.exe")));
        let mut io = ScriptedIo::new(vec![
            HealthVerdict::WrongInterface {
                status: 200,
                expected: "notes.fpack/1".to_string(),
                actual: Some("other.fpack/1".to_string()),
            },
            HealthVerdict::WrongInterface {
                status: 200,
                expected: "notes.fpack/1".to_string(),
                actual: Some("other.fpack/1".to_string()),
            },
        ]);
        let status = ensure_shared_io(
            &generic,
            &identity,
            &mut io,
            &SharedEnsureOptions::default(),
        );
        match status {
            DaemonStatus::Offline(reason) => assert!(reason.contains("不可复用"), "{reason}"),
            other => panic!("错版占用应 Offline，实际 {other:?}"),
        }
        assert_eq!(io.spawns, 0, "占用者存活——锁内不 spawn");
    }
}
