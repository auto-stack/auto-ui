# PLAN-087 交付证据（版本绑定）

日期：2026-11-16。工作树 `D:/autostack/.wt/ui-087/auto-ui`，分支
`codex/ui-087-numeric-fields`，基线 master `34bb4326`。
交付提交：`3adfa914`（T-01/T-02 包+引擎+编译器适配）、`01172305`（T-05 双轨门）。
依赖组：`.wt/ui-087/{auto-lang@107b23d0, auto-web@0adebc0, auto-down@5af11a3, auto-man@54fe407}`
全为 detached master、零源码改动（组内同级布局，相对 path 依赖原样解析）。

## 1. 版本指纹（SHA256）

| 产物 | SHA256 |
|---|---|
| auto.exe（.wt/ui-087/auto-man debug 构建，含本组 auto-ui 编译器） | `1EBC6CB0BFFD7D96F555F39E2BA98D3217F7D94C44A51067CF13B0601027FA83` |
| auto-ui-tool.exe（.wt/ui-087/auto-ui debug 构建） | `5E970EC79104494501C98E1D40C829A02D7F03CA539A69CCC07154BBC3B8FD56` |
| packages/numeric-fields/numeric_decimal.at | `AC6BCBD7E4BACD8C2952FEB67369CD3D370753B1838655A1A5165D566E07435A` |
| packages/numeric-fields/numeric_field.at | `60A5C8FACD78AC5C6FFF9B0271EDAE8F2D400ABD2BB667CFB26DA7E2D67B0CA3` |
| packages/numeric-fields/unit_field.at | `E631F22755223394EC71928C7BE20BBFC0A4B0AF63AEF057389F9105E8A57DB9` |
| examples/numeric-fields/src/front/pages/demo.at | `9BCAB6B3A0B8558C7ADB775701C41EF9AAEA117F59C95C40259461D2A6656DC5` |
| crates/auto-ui-compiler/src/ui_gen/ts_adapter.rs | `93F276867C08BAE59B5BD6B2A1CC29810D5BD90D8563410E4906B9986E2DAE44` |
| crates/auto-ui-compiler/src/ui_gen/vue/shadcn_attrs.rs | `C04A1D20F0278DE94E48DDB2E843D7DDFCD9BC15E317B1E0C59060AC01A63796` |

## 2. 产品门结果（逐例计时入 report JSON）

命令（本组 debug 二进制）：

```
python tests/product/verify_numeric_fields.py \
  --auto-bin D:/autostack/.wt/ui-087/auto-man/target/debug/auto.exe \
  --ui-tool-bin D:/autostack/.wt/ui-087/auto-ui/target/debug/auto-ui-tool.exe \
  --out-dir D:/autostack/plan-artifacts/plan-087/gate/native --backend native

# vue 轨同参数 --out-dir …/gate/vue --backend vue
```

| 轨 | 结果 | 用例 | 最长用例 | not_run |
|---|---|---|---|---|
| native（Windows 11 VM 窗口 + MCP） | **pass** | 21 行（20 pass + 1 具名 not_run） | 4.78s | V-03 Escape（native 多字段表单按 decisions §4 平台边界不驱动；单实例语义由 t03 探针实证） |
| vue（vite + playwright chromium） | **pass** | 21 行（21 pass，含 vue Escape 逐元素用例） | 9.86s | 无 |

- report：`D:/autostack/plan-artifacts/plan-087/gate/{native,vue}/report-*.json`（含每例
  entry/expected/observed/seconds、source/tools SHA、截图 sha256）。
- 视觉实拍：同目录 `v05-*.png`（默认/暗色）、`v06-1251ms-after-step.png`；
  native 截图经 `autoui_screenshot` 落盘后校验非空。
- V-01…V-06 关键断言：1.20+0.10⇒1.30 精确；abc/10位整数/7位小数具名拒绝且
  基准不变；1.004(max=1) 先验拒绝（不先舍入）；±1.235⇒±1.24 半幅远离零；
  999999999.999999@p6 可提交、@p2 具名 overflow；210.00mm→cm 零 value 提交、
  21.5cm⇒215.00mm；1250.00ms 切 s 后 1ms 步进显示 1.251；同值提交不重发；
  外部置值撤销旧草稿（77.77 零提交、输入框显 2.00）；双实例独立提交；
  业务拒绝（受理暂停）保留正式值 24.00 且错误可见；退出重开二次渲染成功。
- 退出/回收：native 会话 terminate 后 poll 确认退出码；二次会话重开渲染
  通过（report `cleanup`/`reopen` 字段）。

## 3. tools/compiler 作用域测试

| 目标 | 结果 | 用时 |
|---|---|---|
| `cargo test -p auto-ui-tools --test numeric_fields_component` | ok 2/2（双生成器准入 + vendor 闭包字节等价 + 缺依赖具名失败 + 引擎整型形态/载荷断言） | 2.40s |
| `cargo test -p auto-ui-compiler --lib ts_adapter` | ok 30/30 | 0.03s |
| `cargo test -p auto-ui-compiler --lib shadcn_attrs` | ok 20/20 | 0.17s |

## 4. 编译器最小通用适配（本 Plan 落地，影响面已对齐）

1. `ts_adapter.rs`：a2ts Range 臂（`range(start,end)`）在 SFC 内联发射缺
   runtime import——`AuraTsContext.used_range` 聚合 needs_range 信号，
   `finalize_with_range` 在使用函数体顶部内联一份 `range` 助手。
   实证：修复前 vue 轨子件 Init 抛 ReferenceError（按钮全禁、单位表空）。
2. `shadcn_attrs.rs`：`base_event_to_dom` 原先对小写副本剥 `on` 前缀，
   camelCase 事件名（`onunitChanged`）被降为全小写并与子件
   `emit('unitChanged')` 失配——改为从原始 base 剥离。
   实证：修复前 App.vue 生成 `@unitchanged`（监听器永不触发）。
回归：ts_adapter 30/30、shadcn_attrs 20/20、本 Plan 双轨产品门全绿；
native 探针重跑一致。统一回归（受影响面冻结 union）按流程归 review/merge。

## 5. Q-01 关联（DEBT-006 文字 shaping）

字段 label/值文本在 native 截图中可见可读（V-05 实拍），本 Plan 不做逐字形
workaround，不以截图宣称 DEBT-006 已修复——受影响面（input 内文本 shaping）
在真实键入路径由产品门文本断言覆盖；DEBT-006 仍为仓库级未决债务。

## 6. 探针工件（仓库外 scratch）

`D:/autostack/plan-artifacts/plan-087/`：`probes/`（t01/t02/t03 探针应用、
probe_run/v2/v3/v5 驱动、会话日志）、`gate/{native,vue}/`（report+截图）。
合并闭包时按证据政策折叠入本文件并删除。
