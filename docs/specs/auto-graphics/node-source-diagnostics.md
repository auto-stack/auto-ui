# AutoGraphics Node 源映射与分层诊断（auto-ui PLAN-044）

Status: delivered by auto-ui PLAN-044 r1（实现提交见计划归档收据；本文件只描述
实际交付并验证的能力。所有定位精度以"可证"为界，不声称 SDK 未提供的能力）。

## 0. 范围

Node 生成链（pilot `autographics-node2d-pilot`，components-2d / queries-2d
发射路径）可选产出源映射旁件 `source-map.json`：把 SDK compile 诊断映射回
原作者模块/Node/事件/helper，精度明示。显式 CLI 工作名 `--diagnostics-map`；
默认 CLI 无旁件，world/API 源字节与旧档行为不变（真实进程控制实测）。

非范围（不冒称支持）：断点/单步/变量检查、运行期 trace/调用栈恢复、
managed 区映射到作者源、跨仓 AST/SDK 改造、旧内容批量升级、M2 新前置。
runtime/API 错误无 span 的事实保持不变：只给 stage/callback 与可证上下文，
明确 "SDK supplied no mapped line"，不编造行。

## 1. 精度模型（五档，明示不猜测）

- `exact`：生成片段与原作者源字节一致，可字节级回算行列。
- `range`：身份与行号可证；列可能因行内改写偏移。当前 pooled helper 段
  恒为 range（发射器把声明与调用的 `{name}(` 改写为 `{module}_{name}(`），
  列经重命名重放差分（只接受 `{module}_` 纯插入）在可证时回算。
- `context`：Node/事件/helper 上下文可证，不给语句级位置；锚到声明行。
- `generated`：合成代码（SoA/调度/未绑定 handler 等），永不映到作者行。
- `unmapped`：映射缺失或被拒后的显式回落（原诊断路径不变）。

首批覆盖：跨模块 pooled helper（含同名 helper 多模块不互串）、Life 处理器
（node/event 身份 + 声明行锚）、Member/recv 处理器（唯一属主文本规则：suffix
匹配全工程唯一 Member 事件属主才归属，多属主不映）。未绑定（无发射代码）的
handler 无可映射物，缺口留 generated。

## 2. 旁件与绑定（node-source-map/v1）

- 段 = 生成用户源半开 byte 区间 + module/node/event/helper + 精度 + 源内
  半开 byte 区间；段有序、不重叠、全覆盖（generated 补齐铺瓦）；UTF-8 字符
  边界强制；列口径 = Unicode scalar（.chars()），不宣称屏幕字素宽。
- 绑定：world_sha256 / user_source_sha256 / user_source_bytes 与实际载入
  字节双向核验；module key 为相对闭包名（拒盘符/分隔符/`..`），各带
  sha256+bytes 与 closure/ 目录逐字节一致。任何不符 → 整图拒用。
- 源内容只来自受检 closure 副本：Run 期间磁盘源变更/删除不影响已接受包；
  旧 Run 不能用新 map（sha 绑定）；SaveAs/搬迁 = 包内相对结构，同字节在
  新目录等价消费（实测）。
- 上界（实测冻结）：旁件 ≤ max(64KiB, 4×world) 且 ≤4MiB，段 ≤16384
  （worst 实测 ratio 3.42 @ 密集 helper 源；超限分类 = 整图拒用 → generated
  fallback，不截断、不部分采用）。metadata 是实际 CPU/包开销。

## 3. 受检消费（双锚，映射只解释已有诊断）

driver `behavior_source_map` 为 pilot 旁件的独立镜像（两 Cargo 图不共享
crate，deny_unknown_fields 拒未知字段）：

1. **span 锚**：SDK span 落入受检 assembly user region → 段查找 → range 段
   重放重命名差分回算；若错误消息携带符号（undefined-symbol 类），要求
   world 该处逐字节等于该符号（防偏移 span 误映）。
2. **符号锚恢复**：span 锚失败且消息含符号时，全部 range/exact 段内按标识
   符边界搜该符号，**唯一出现**才可证映射；多处出现不猜。

坏图/坏闭包/旧 Run/漂移源 → 明确回退原诊断；map 错误绝不覆盖原错误；
正文保持 8KiB 截断（UTF-8 安全）。错误文本内嵌的伪造行号永不作为位置
证据。managed/transport/bridge 区的 span 永不映射（managed 绝不当用户）。

## 4. 分层诊断（保原 stage/code/来源）

- 前端/SDK compile：映射成功 → `{module}.at:{line}:{col} [stage/module;
  node-source-map {precision} helper=…/node=…/event=…]: {text}`；失败 →
  原 `sdk_diagnostic`（region 行列或 no mapped line）。
- runtime/API fault：保持 `[stage/callback; SDK supplied no mapped line]`
  原样（本 SDK 运行错误无结构化 span，是事实不是缺陷）。
- managed 内部错误：保持原 stage 与错误文本。

## 5. SDK681 span 语义（冻结事实，换 SDK 须重测）

undefined-symbol 是唯一易达且带 span 的作者可修 compile 错误类；其 span
为 "call 表达式末尾 + 前置 // 注释行数" 口径（实测：无注释恒 +12，每注释
行 +1，与 token 位置/嵌套无关；±1 级），不可字节回算。type/arity/field
错误在该 SDK 编译期放行。因此可证定位走 §3 双锚，span 仅作候选+校验，
不作唯一依据。

## 6. 预算与等价（实测/结构事实）

debug 有无旁件：world/API 字节等价（真实进程再生控制）；VM 步数/费用/
帧等价 = 结构事实（map 仅在 SDK compile 诊断回调消费，不入 Session 状态、
零运行期钩子、无每 Tick IO）；音画无触面改动。8KiB 正文截断两侧保持。

## 7. 入口

- `python experimental/autographics-node2d-pilot/tools/verify_plan044.py --controls`
  （默认无旁件 + 冻结 sha、旁件绑定、再生一致、检入产品现行、driver gates）。
- pilot：`cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node_source_map_gates`
- driver：`cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test node_source_diagnostics -- --test-threads=1`

## 8. 未来 origin 接入合同（043 及后续发射段）

新发射段接入映射一律走本文件的可证规则，不按 Coin/Player/示例名造表：
①发射侧提供唯一可证锚（唯一锚定注释或全局唯一命名规则），driver 侧镜像
校验；②精度按可证性定档（verbatim→exact、行内改写→range、重写体→context、
合成→generated），不得无证据升档；③无法锚定即不映（缺口留 generated），
建图 fail-closed 而非猜测归属；④schema 变更必须升 MAP_FORMAT 并让两侧
镜像同步（deny_unknown_fields 拒未知字段的旧消费）。
