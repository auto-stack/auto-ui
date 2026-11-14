# FeaturePack daemon 宿主托管合同（声明、健康、配置边界与单实例）

> PLAN-083 交付合同（SD-01）。Windows 本地 FeaturePack/共享服务 daemon 的
> 宿主侧 canonical 行为：可选能力声明与健康核验、服务/宿主配置分界与受管
> 凭据解析、跨进程单实例门与共享 provider 生命周期。分类：runtime ｜
> 状态：active ｜ 来源计划：docs/plans/083-featurepack-daemon-runtime-contract.md

## 1. 适用范围与所有权

- 本页约束 **auto-ui 运行时宿主侧**：读 `apps.manifest` 声明、ensure（检活/
  spawn/就绪）、跨进程协调、把宿主配置投递给 App 后端。
- daemon 本体（服务实现、业务数据、policy 语义）归对应产品仓；OS 拥有受管
  policy 签发（`scripts/fp_provision_policy.{sh,ps1}`）与真实双应用联合验收；
  Reader/OS App 拥有自己的启动声明与客户端消费。
- 运行时**只**验证声明并精确传递：不生成凭据、不解释授权书内容、不把
  凭据写进公开 manifest 或日志。
- `autoos` 旧链（os-config `AUTOOS_DAEMON`）逐字保留（§8），不因本页改写。

## 2. manifest 声明（`apps.manifest` 条目 `daemon`）

```json
"daemon": {
  "port": 17819,
  "bin": "../../../auto-notes/target/release/notes-service",
  "env_port": "AUTO_HTTP_PORT",
  "health_interface": "notes.fpack/1",
  "ready_timeout_ms": 20000,
  "env": { "NOTES_DATA_DIR": "%APPDATA%/auto-notes/data" },
  "host": {
    "env": { "NOTES_RETRY": "2" },
    "credential": { "env": "NOTES_FP_CRED", "path": "%APPDATA%/auto-notes/data/fp-policy.json", "principal": "fp-reader", "field": "cred" }
  },
  "instance_key": "notes:default",
  "data_root": "%APPDATA%/auto-notes/data"
}
```

| 字段 | 必填 | 语义 | 非法时 |
|---|---|---|---|
| `port` | 是 | 端点端口（`http://127.0.0.1:<port>`） | 解析失败整表宽容跳过（既有） |
| `bin` | 否 | 仓相对可执行路径（平台中性；Windows 缺 `.exe` 自动补）；缺席 = 只探不孵 | — |
| `env_port` | 否 | spawn 期端口覆盖键；缺席 = `<NAME>_BACK_PORT` | 非 env 键语法 → 具名拒绝 |
| `health_interface` | 否 | `<name>/<n>`：名 `[A-Za-z0-9._-]` 首字符字母/数字、版本十进制、≤128 | 具名拒绝（不退回旧探活） |
| `ready_timeout_ms` | 否 | 就绪预算，200..=60000；缺席 = 调用方 5s | 越界具名拒绝 |
| `env` | 否 | **服务进程**非敏感 literal（spawn 时投递） | 空值/控制符/疑似凭据键/长 hex 值 → 具名拒绝 |
| `host.env` | 否 | **宿主后端**非敏感 literal | 同上 |
| `host.credential` | 否 | 受管凭据引用：`env`(注入键)、`path`(policy 文件，可缺省为基目录 `fp-policy.json`)、`principal`、`field`(缺省 `cred`) | 键语法/主体/字段/路径非法或与 literal 同键 → 具名拒绝 |
| `instance_key` | 否 | 受管实例键，`[A-Za-z0-9._:-]`、≤96；缺席 = 条目 `id` | 非法字符/超长/空 → 具名拒绝 |
| `data_root` | 否 | 规范化数据根（`%VAR%`/`$VAR` 展开）；参与实例身份 | 空串/控制符 → 具名拒绝 |

公开声明纪律：`env`/`host.env` 中出现疑似凭据键（`cred|token|secret|passwd|
password|apikey|api_key`，大小写不敏感）或形似凭据的值（≥32 位偶数长 hex）一律
**具名拒绝**——凭据只能经 `host.credential` 受管引用。

## 3. 健康合同（就绪判定）

- 请求：`GET /api/health`（HTTP/1.x，loopback）。
- 预算：连接/读超时由探测点给出（首检 800ms、轮询 250ms；`ready_timeout_ms`
  约束整体就绪）；响应头 ≤8192B；响应体 ≤8192B（`HealthSpec::DEFAULT_MAX_BODY_BYTES`）。
- 支持集：`Content-Length`、`Transfer-Encoding: chunked`、连接关闭定界；其他
  framing（如 `gzip`）→ 具名拒绝，**不从任意无限 body 假定支持**。
- 响应体：JSON；允许 ≤2 层字符串信封（当前 Notes `JsonResponse<String>` 双层
  编码）；必须是对象，`interface` 与声明逐字相等，`status` 若存在必须为 `ok`。
- 裁决（具名）：
  | 裁决 | 含义 | 复用/拉起 |
  |---|---|---|
  | `Ready` | 2xx + 合法 body + 接口匹配 | 复用（零打扰） |
  | `NoListener` | 连接拒绝/超预算 | 允许 spawn |
  | `OccupiedElsewhere` | 有监听但不是 HTTP 目标（非 HTTP/无响应） | 拒绝复用、**不 spawn** |
  | `WrongInterface` | 2xx 但接口不符（错版/普通占用者） | 拒绝复用、**不 spawn**、不向其投递配置/凭据 |
  | `NotReady` | 非 2xx | 拒绝复用、不 spawn |
  | `BadBody` | 2xx 但坏 JSON/超预算/不支持 framing | 拒绝复用、不 spawn |
- spawn 后按**同一接口**轮询：只有 `Ready` 结束；`WrongInterface` 立即拒绝；
  其余等到预算截止（2xx 早到不提前 ready）。就绪不推导数据库写成功，业务提交
  仍以服务回执为准。
- **无 `health_interface` 声明**时保持既有语义：**只读状态行**——2xx 即就绪，不读 body、不核 framing/字节预算（与旧 `tcp_ping` 同义；超大/截断/怪 framing 的 body 不影响就绪）。spawn 后轮询同理：非 2xx 继续等，2xx 结束（HTTP 503 初始态不构成终局拒绝）。

## 4. 服务进程配置（spawn env）

- 投递 `daemon.env` 的非敏感 literal + 端口覆盖键（`env_port`，值 = 端点端口）。
- 端口键由声明 `port` 权威，同名 `env` 项被覆盖（不允许声明与端口不一致）。
- detached spawn：`DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`，stdio 全 null，
  句柄即刻释放；桌面退出不杀共享 daemon（生命周期归 OS/服务自身）。

## 5. 宿主后端配置（公开声明与私有凭据的分界）

- 投递面：**独立进程后端**——native exe 应用子进程、Rust API 后端（桌面
  `DesktopBackends` 启动器）子进程。投递方式：子进程 `Command`/`TrackedSpawn`
  的 env，并在 spawn 前剔除继承的同名键（宿主历史值不得泄漏/覆盖）。
- 键集合：`<NAME>_DAEMON=<url>`（`daemon_env_key` 派生：大写蛇形 + `_DAEMON`）、
  `host.env` literal、`host.credential.env` = 受管 policy 解析值。
- 受管凭据解析：读 `path`（或缺省 `<base_dir>/fp-policy.json`），在
  `entries[]` 中按 `principal` 命中取 `field`；值必须非空、无空白/控制符、
  16..=512 字符。错误只回显主体/路径/字段，**不回显值**；Debug 输出打码为
  `<managed-credential>`。
- 明文边界：凭据不出现在公开 manifest、诊断、日志、回执中；只出现在目标
  子进程 env。
- **in-proc 接口缺口（明确记录，不是静默降级）**：VM/解释态 App 的
  `Env.get` 是进程全局（`auto-lang/crates/a2r-std/src/env.rs` = `std::env::var`；
  VM `auto.env.get` 同源），无法给同一进程内多个宿主做凭据隔离。因此：
  - in-proc 只注入非敏感键（地址与 literal），凭据**不写**进程全局；
  - 运行时输出具名缺口诊断（`[plan083] ... 仅独立进程后端持有凭据`）；
  - 「轮流设置同名全局变量」不得作为隔离实现或验收通过方式。
  该缺口需 auto-lang 增加显式上下文（另一仓/另一计划），本页只固定边界。
- 生效时序：宿主配置在后端首次请求前可见——解析/ensure 必须发生在后端
  spawn 之前（桌面启动器在 spawn 前解析共享 provider；在位复用与重开恢复
  不依赖运行期改写全局变量）。

## 6. 实例身份与跨进程单实例门

- 身份 = `user + instance_key + port + 规范化 data_root`（路径统一为小写
  `/` 分隔、去尾斜杠；`<none>` 表示未声明）。仅靠路径文本别名或进程内
  `DID_SPAWN` 标记**不构成**身份。
- 互斥：Windows 以锁文件 `share_mode(0)` 独占打开实现单写者（等价 named
  mutex）；进程异常退出由 OS 关闭句柄释放，无陈旧锁。非 Windows 为
  `create_new` + 陈旧 TTL 回退（Windows 是合同平台）。
- 实例标记：就绪后写 `{instance_key, data_root, port, pid, started_at}`，**按
  端口**定位（同端点唯一 owner）。
- ensure 序：快路径探测 → `Ready` 且身份一致（或缺席）→ 复用；否则取锁 →
  **锁内二次探测** → 仅 `NoListener` 才 spawn → 同接口轮询 → 就绪后写标记 →
  释放锁。取锁失败 = 输家：在 `lock_wait` 内有界轮询，见到 `Ready` 即按身份
  复用；锁释放则重试；超时具名 Offline（携带最后裁决）。失败/超时回收本次
  标记并释放锁。
- 身份冲突：同端点、标记身份不一致（不同数据根/实例键）→ **具名拒绝**，不得
  悄悄采用另一数据根的库。标记缺席（手工启动的服务）仅在健康接口已核验时
  允许复用，并记录 owner 未知。

## 7. 共享 provider 生命周期与后端去重

- 声明 daemon 即应用后端（pac `back_port` == `daemon.port`，如 Notes 17819）：
  桌面启动器**不**启动第二份自有后端、不把该端口当"已被占用"拒绝，路由直接
  指向已核验身份的 daemon（`strip: true`）。
- 普通端口占用仍按既有规则具名拒绝；共享复用只发生在声明与身份校验之后。
- 共享 daemon 独立于任一主窗口/宿主进程寿命（detached + 标记归属）；自有
  后端仍由 owned-process 设施按既有规则清理。

## 8. 兼容矩阵

| 场景 | 行为 |
|---|---|
| 无任何新字段（autoos/musk/jade-garden 等） | 旧 2xx 探活、路径式 `bin`、`env_port` 派生、`<NAME>_DAEMON` 注入——**逐字不变** |
| `autoos` 旧链 | `ensure_ready`/`AUTOOS_DAEMON` 原样，不走 manifest 新字段 |
| 仅 `health_interface` | 严格核验；错版/坏 body 拒绝复用且不 spawn |
| `health_interface` + `ready_timeout_ms` | 预算覆盖调用方缺省 |
| 仅 `env` | 服务进程多投递 literal；端口键仍权威 |
| `host.*` | 仅独立进程后端投递；in-proc 见 §5 缺口 |
| `instance_key`/`data_root` | 跨进程门 + 身份核对；缺失 = 条目 id + 无数据根身份 |
| 未知字段 | 按 manifest 既有宽容（忽略），不阻断启动 |

## 9. 债务关闭映射

| 债 | 原缺口 | 本合同的收口 |
|---|---|---|
| DEBT-003 | 2xx 前缀即 ready，不核 body/interface | §3 健康合同（`health_interface` + 具名裁决 + 预算/framing） |
| DEBT-004 | spawn env 仅端口键；宿主拿不到凭据 | §4/§5 服务/宿主配置分界 + 受管凭据解析 + 子进程精确投递（in-proc 缺口具名） |
| DEBT-005 | ping→spawn 无跨进程互斥，可双 spawn | §6 身份 + 独占锁 + 锁内二次探测 + 输家复用 + 标记核对（spawn 事件 = 1） |

债务在实现交付后关闭（本页随合并落主干）；OS 侧真实双应用验收仍归 OS-067 T-04。

## 10. 消费接口（OS / Reader）

- OS：声明 `health_interface`/`instance_key`/`data_root`/`env`/`host.*` 并提供
  受管 policy；`bin` 用路径式发布件（不带命令行）。真实双宿主编排由 OS 拥有。
- Reader：宿主后端 env 键固定 `<NAME>_DAEMON`（`015_NOTES_DAEMON`）+ 自有凭据
  键（`NOTES_FP_CRED` 由 `host.credential.env` 声明）；`NOTES_DAEMON` 保持
  独立/测试覆盖语义。
- 诊断只暴露：实例身份（`instance_key/port/data_root`）、健康裁决、policy
  主体/路径/字段，不含任何凭据值。

## 11. 当前缺口与不可达（如实记录）

- in-proc 宿主凭据隔离（§5）需 auto-lang 显式上下文；本版只保证不写全局。
- 真实 Vue `DesktopBackends` 的 Rust 后端编译/启动档未在 PLAN-083 执行
  （`PLAN053_PROBE_DIR` 门）；共享判定与端口豁免有单测，子进程 env 投递有
  接线与用例，但"真后端首请求看到配置"的端到端证据归 OS 联合验收。
- Notes 真实桌面双窗口（主窗口 + Vue 启动器）编排未在本计划跑；去重规则以
  `back_port == daemon.port` 判定并用单测钉死。

## 12. 证据

PLAN-083 §9 的 T-01～T-04 回执：`osconfig_daemon` 39、`daemon_instance` 7、
`featurepack_daemon_hosting` 4（真多进程 spawn=1）、`featurepack_daemon_launch`
1（真实 launch 注入/复用）、compiler `app_registry` 34、tools
`desktop_backends` 3。夹具 `fp_daemon_fixture` 为测试专用 bin（无
`FP_FIXTURE_PORT` 即 exit 2）。
