# 原子来源Bundle Reload（atomic source bundle reload）

> PLAN-056 交付合同（SD-01）。多文件候选 bundle 的**内容级**失效检测、
> 独立候选构建/验证、typed schema 迁移与一次原子提交；失败具名拒绝保旧，
> 合法下一编辑成功。分类：feature ｜ 状态：active ｜ 来源计划：
> docs/plans/056-atomic-multifile-reload-and-schema-lifecycle.md

## 1. 来源快照与失效检测（A-01）

- 快照 = root + transitive `use` 模块文件（同一模块解析规则：模块目录 →
  `src/` 父级 → 语言仓根回退；c/rust/py/auto.*/.web/use.py 不参与文件
  快照），逐文件 blake3 内容 hash + mtime。
- `check_reload`：每拍全量内容核对（同 mtime 改内容也能检出；mtime 仅存
  于条目作诊断）。旧快照任一文件已消失 = 具名 Err（缺源保旧，保持旧快照，
  恢复后下一次合法编辑成功）。内容/图变化 → 快照基线前移 + 置待提交标记。
- mtime 快路径不构成身份证明（`reload_bundle::mtimes_unchanged` 保留为
  诊断/未来快路径；权威判定是内容 hash）。

## 2. 候选构建与原子义务（A-02）

- 候选 = 独立全量 `DynamicComponent`（root + imports + store-as-child +
  handlers + schema + effects 单 VM 合成），由真实 build 管线以
  `build_reload_candidate` 构建（`candidate` 模式）：
  - **不** fire Init（准备期零外部 effect：Init/Tick/SSE/IO 均不启动）；
  - **不** 安装 action_config DSL（其独立 mtime 泵不受扰，AC-05）；
  - **不** 注册全局主题树/全局 design/i18n/外接组件名单（提交时由现役链
    接管；per-component 主题 scope 照常建立）。
- 候选构建期间来源再变（快照失配）→ 具名拒绝保旧，修复后自动再次尝试。
- 多文件 guard 已退役：`check_file_changed` 的 import/registry 跳过语义只
  保留在 legacy 单文件 API；泵与永久测试走 `try_reload` 单一入口。

## 3. typed schema 迁移（A-03）

- 组件保留 `last_schema`（root + children + stores 的 AuraWidget 状态定义
  表，构造时捕获、提交时更新）。
- 根态迁移（store 字段平铺单 VM 统一根态，随根态覆盖）：
  - 同名同类型 → 保值；新字段 → 新桥已播种默认；删除 → 具名报告；
  - **同名不同类型（含未知/不透明）→ 整候选拒绝保旧**（不乐观写入）；
  - 宿主槽（`__*`）不迁移、不出现在 dropped_names。
- 跨桥值重物化（`rematerialize_from_old`）：容器/堆引用深拷进新 VM，避免
  悬垂引用；不透明堆对象具名拒绝（fail closed）。
- keyed 实例迁移：旧存活身份按名+类型迁移（model 字段 typed 检查，prop/
  seed 键由父通道每帧重播按名保值）；新 model 字段按新定义默认补位；
  提交后同身份实例集合不变；已删除 widget 类型的身份 tombstone（迟到事件
  丢弃不串新实例）。未 keyed 按身份串携带（旧位置语义，不承诺任意移动）。

## 4. 提交与代次生命周期（A-04）

- 提交前旧桥 `has_parked_tasks`/`cpu_continuation_count > 0` → Busy 保旧，
  下拍重试（待提交标记防检测被吞）。
- 提交 = 新代次模块资源建立一次（`__module_init`；**不**重放 Init handler
  ——保值语义，AC-02 不覆盖迁移值）+ 全量 runtime 面原子切换 + epoch 保守
  失效 + dirty。
- 订阅面经 renderer 每拍由 timesources 重建自然收敛（整体替换旧表，不逐
  项增删）；旧桥随 Drop 释放其 VM 堆。
- Init 挂载语义保持：每身份每代际一次（迁移实例预置 Done 防重放；卸载重挂
  仍重 Init，os-config 身份语义不变）。

## 5. 支持边界（明确未交付）

- Vue/Vite/Node HMR、公开 MCP 组件 reload API、动态路由/外部 Python/web
  模块热替换不在本包（具名重启轨）；旧 `hot_reload.rs` 存根不包装成实现。
- `reload()` legacy 单文件 API 语义原样保留（含其多文件形态灾害边界——
  强制 reload 探针作为"不得回退到部分替换"的回归钉保持绿）。
- 旧单文件路径（`check_file_changed` + `reload`）无回归。

## 6. 验收锚点

- 永久 fixture/测试：`crates/auto-ui-runtime/src/ui_tests/
  plan056_atomic_reload_tests.rs`（V-01..V-05，15 例，2026-11-05 全绿）；
  042 十二例更新后的 13 例全绿（t03 改写为新行为断言，t04 灾害边界保持绿）。
- 入口：`cargo test -p auto-ui-runtime --lib plan056_atomic_reload_tests`、
  `cargo test -p auto-ui-runtime --lib plan042_reload_readiness`、
  `cargo test -p auto-ui-runtime --test plan041_instance_ownership`。
- 变更检测语义（原 reload-readiness-and-lifecycle §2 对账）：mtime 误报/
  漏检两探针（t02_same_mtime_content_change_missed 负控）已被内容级检测
  取代——同 mtime 改内容**检出**（V-01 正控）。
