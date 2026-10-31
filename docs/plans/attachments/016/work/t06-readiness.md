# PLAN-016 work 移交 readiness（T-06 完成时点）

date: 2026-10-31 · tree: D:/autostack/.wt/ui-016/auto-ui · branch: codex/plan-016-dev
base_commit: 69690dfb8e0b691a9af8d9e1eacc240511d2e83e（master，含 PLAN-013 交付 8b1e799）
code_commit: 见 §9 最终移交收据（T-06 提交）
dependency_revisions: SDK 7b987ca8（driver .sdk-source，source-lock 校验）；auto-lang 只读引用（生成器树 3fa75dd9、语言仓零写面）；auto-down 只读兄弟树（runtime dev-deps）
合同: attachments/016/work/node2d-contract.md（含 §3a T-04 实测修订）

## 工作侧 AC 结果（全部闭合，证据=当前提交上的命名测试断言）

| AC | 结果 | 证据 |
|---|---|---|
| AC-01（013基线/唯一入口/预算先冻结） | pass | G-01：013 归档 60e6d48/交付 8b1e799 实读；合同 §1/§2 冻结（接法比较 (a) 结构性排除附代码行证据，(b) 原型整链 t02-prototype.log SHA 1e54b98f）；无第二解释器：生成物由 pinned SDK 真编译执行；语言仓写面 0 |
| AC-02（node/view-2d/Life/msg/use 真实生效+诊断） | pass | auto-ui-dialect tests/node2d_pilot 6/6（真实夹具解析、6类定位拒绝、UiDialect opt-out）；auto-ui-compiler tests/node2d_pilot 6/6（模块定位/错导出/缺模块/跨模块 fn 池化）；改 helper 敏感性=verify_plan016 夹具变异控制（world 必变） |
| AC-03（H1/H2 同定义独立状态/receiver/订阅；Start/Tick 调度） | pass | flow_minimal_world_instances_events：双 Hazard 相位/计数独立匹配独立 oracle、根 phase 哨兵不动、starts 各 1、ticks=收集 tick；receiver 隔离另见 flow_supplement |
| AC-04（Player 真实运动→父事件→条件差量真销毁；负控） | pass | 同上：player node x 精确=2+2t/60（真扫掠）；真事件→main_collect→coin slot free（AG 帧层）+ GPU 金类消失；重复收集幂等；"只有 hidden" 负控=slot generation/存活断言（hidden 保持 live）；错 receiver=flow_supplement（b 未被投递） |
| AC-05（Stop/fresh/旧代/owner 终态 20 轮） | pass | flow_host_pause_step_neutral_and_20_rounds（Pause 不推进/Step 恰一中性 tick/20 轮 stop-fresh tick 归零/币复活/终态健康）；flow_cpu_20_round_session_owner_release（20 轮 owner probe：开=持、释放后=无）；旧代=flow_budget（stale handle 故障 33、新根不沾染） |
| AC-06（120tick GPU、预算/守护负控、A→B、旧触面保持） | pass | flow_gpu_project_pixels_and_real_despawn（120 tick 窗口内 GPU 像素三层证据）；预算=满池 33rd 调用故障 10 + verify_plan016；A→B=consumerB（git archive 新根 + auto-lang 3fa75dd9 只读 + 冷 .sdk-source 681 files）再生字节等价（f66b1b92/4bd54fa3）且 driver node2d_flow 在 B 上执行通过；旧触面=driver/生产 crate src 零 diff（本计划 driver 侧仅新增 tests/），legacy profile 字节无触 |

## 开放 in-contract findings / 缺口 / 未决合同决定

- 无工作侧未闭合 finding。合同 §10 D-04（独立终审上下文）归 review 阶段；
  D-01/D-02/D-03 已由 T-01/T-02 闭合（013 已归档；接法 (b) 冻结；refs/msg/
  预算细节已实测冻结入 §3/§3a）。

## 受影响既有用例重放

- driver 生产源零改动 → legacy profile 组合字节/行为无可影响面（复用既有
  全绿证据，未重复全量回归；统一回归门按工作流归 review/merge 所有，保持
  pending）。
- 主仓 crates/ 触面：auto-ui-dialect（仅新增 node2d 模块与测试）与
  auto-ui-compiler（仅新增测试）——两 crate 全量套件实跑全绿
  （dialect 95+6+2，compiler 1123+9+6+1+3，0 failed）。
- 阴性集：合同 §3 各禁止项均在测试或实测中闭合（float % 运行期拒绝、
  保留名准入拒绝、声明先于使用、非 const 顶层初始化、重复根符号）。

## 本次实跑的测试（当前提交）

1. cargo test -p auto-ui-dialect --test node2d_pilot（6 passed）
2. cargo test -p auto-ui-compiler --test node2d_pilot（6 passed）
3. cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml
   --locked --offline（生成器构建+verify 工具自检）
4. python .../tools/verify_plan016.py --controls（PASS exit0，日志
   plan-artifacts t06-verify.log SHA 139ff823）
5. cargo test --locked --offline --test node2d_flow -- --test-threads=1
   （6 passed 0 failed，driver 树）
6. consumerB：git archive 新根 + auto-lang@3fa75dd9 + prepare_sdk 681 files
   → node2d-lower 再生 sha f66b1b92/4bd54fa3 与 A 逐字节等价 → driver
   node2d_flow 全量 6 passed exit0（B 消费不借生产者构建状态；日志
   plan-artifacts/auto-ui/plan-016/t06-consumerB.log）

## 身份

- generated/world.at 4858B sha f66b1b927e50c52f96242600deaab84e9f10b2759a881c878d5cfc85048c0c41
- generated/scene.agscene 绑定上 sha（triggers2d + byte_length 4858）
- generated-supplement/world.at 3764B sha 4bd54fa364aa8fd136d7a18a820bf2a921e3e6d08fd3648f4460286c9e250b19
- 夹具 14 文件 sha 见 verify_plan016 PASS 行（t06-verify.log）

## R1 修复闭合（2026-10-31，独立终审 R1 needs_fix → 本轮 repair）

| Finding | 关闭方式 | 证据 |
|---|---|---|
| F-01 错导出 | `validate_closure` 错导出定位拒绝 + `wrong_export_rejected` 负测 | pilot gates 9/9 |
| F-02 坏 actor 引用 | `refs.<未知实例>` 定位拒绝 + `unknown_actor_ref_rejected` | 同上 |
| F-03 未知实例属性 | 属性名不在参数表 → 定位拒绝 + `unknown_instance_property_rejected` | 同上 |
| F-04 重复局部绑定 | 参数/model 字段重名定位拒绝 + 两个负测 | 同上 |
| F-05 fmt | `rustfmt --edition 2021` 已应用到 driver 测试与全部触面 | fmt --check exit 0 |
| F-06 warnings | dialect 2、dialect 测试 3、pilot 3、driver 1 全清（触面 0 warning） | 构建输出 |
| F-07 合同命令 | 合同 §4 更正为 pilot `--test node2d_lower_gates` + driver `--test node2d_flow` | 合同 diff |
| F-08 预算算式 | held1+char_move26(24+2)+2+1+1+1=32 修正入合同/SD（实测 32 不变） | 合同/SD diff |
| F-09 Life.Frame | 解析层显式越界拒绝 + `life_frame_out_of_scope_rejected` | 同上 |
| F-10 Start 同名区分 | `life_start_and_business_start_are_distinct`（lifestart vs start 两 fn、调用面分离） | gates 9/9 |
| F-11 化妆项 | schema/6 注释、verify docstring、重复 node 声明带行号（node.span） | diff |

修复后电池：dialect 6/6、compiler 6/6、pilot gates 9/9、driver flow 6/6、
verify --controls PASS exit0（t06-verify-r2.log）；生成物 sha 不变
（f66b1b92/4bd54fa3——拒绝面不触碰已冻结世界）；dialect 全量套件 0 failed。
修复提交：9eb1d0e（见 §9）。

## review/merge-owned 待办（不阻塞移交）

- T-07 同树独立 final review（D-04 上下文按 §6 既有安排）；
- T-08 冻结定向门（本计划命名用例 + 上述负控 + fmt/diff 检查）与合入；
- T-09/T-10 归档与清理（merge 所有）。
