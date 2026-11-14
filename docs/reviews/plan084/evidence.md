# PLAN-084 实施证据汇总（T-06）

日期：2026-11-14。工作树 `D:/autostack/.wt/ui-084/auto-ui`，分支
`codex/ui-084-file-dialog-modes`，基线 `a6f776c2`。原始工件（命令/日志/
截图/report.json）在 `D:/autostack/plan-artifacts/plan-084/`（仓库外
scratch，merge 按政策清理）。本文只留可复核索引。

## 工具与依赖身份

| 项 | 值 |
|---|---|
| 本树 HEAD | 见 §9 work receipt（实施 commit） |
| auto.exe（Windows 门） | `D:/autostack/.wt/ui-084/auto-man/target/debug/auto.exe`——组内 auto-man（detached `54fe407`）构建，相对 path 依赖解析到本树 auto-ui（构成"补丁版"） |
| auto-ui-tool.exe（Windows 门） | 本树 `cargo build -p auto-ui-tools --bin auto-ui-tool` |
| 依赖工作树组 | `.wt/ui-084/{auto-lang e0cfa2d8, auto-web 8b1742a, auto-down 58ea33f, auto-man 54fe407}` 全 detached 只读；本树零依赖仓改动 |
| PLAN-086 pin | master `765251b9`（PaginationControl 包 + canonical spec；本树含同字节 `packages/pagination`） |

## 门结果（§6 命令 → 结果）

| 命令 | 结果 |
|---|---|
| `cargo test -p auto-ui-runtime --test file_dialog_model --test file_dialog` | **16+13 pass**（原 10+8 不退化 + V-01..04 模型反例 6 + 组件级 4：嵌套 modal P-1 / ui.focus P-2 / 目录模式组件回路 / 保存覆盖组件回路 / 模式冻结） |
| `cargo test -p auto-ui-tools --test file_dialog_component` | **3/3 pass**（双生成器 + 新 props defaultFileName/defaultExtension/覆盖层源断言 + 闭包） |
| `python tests/product/verify_file_dialog.py --auto-bin … --ui-tool-bin … --out-dir …/vfd-full-win` | **outcome: pass**（900x700 全门：原 1003 项 11 页流/取消重开 10 次/焦点恢复/系统可达性 + PLAN-084 目录/保存/覆盖全流程、字节基线 unchanged、accepted_directory/accepted_child_directory/accepted_save_new/accepted_overwrite/save_named_errors/accepted_save_completion 全记录） |
| 同上 `--window 420x560 --visual-only`（vfd-narrow-win） | **outcome: pass**（窄窗腿 + V-08 参考区截图：directory-mode[+dark]、save-mode、overwrite-layer[+dark]，footer 可达） |
| `python tests/product/verify_file_dialog_modes_consumer.py …`（consumer-win） | 041 真实消费门（Ctrl+S 打开保存对话框→普通保存写入读回、另存为预填+新路径、覆盖 Back 保字节/Replace 应用写入、取消不建文件保脏、只读目标具名失败保脏），详见 report.json checks |
| Linux native 轨 | runner 在位（WSL Ubuntu-22.04：cargo 1.97.1 + Xvfb + python3；`auto-ui-tool` Linux 版构建成功）。`auto` Linux 构建因 WSL→crates.io 网络间歇失败（curl error 7，IPv4 hosts 固定后仍间歇）多次重试未完成——**不记 pass**；Linux 门状态见 §9/§10（可补跑：网络恢复后在同组构建 auto 并跑三脚本） |
| 统一回归 | 未在本阶段跑（owner=review/merge 按仓档位） |

## 视觉证据与 Q-01（DEBT-006）

- 窄窗四态截图（420x560，真实窗口，sha256 见 report.json）：
  `1-directory-mode.png`（选择一个目录/行选择+chevron 进入/选择当前目录/
  文件行无 checkbox/取消+选择此目录可达）、`1-save-mode.png`（保存位置副
  题、格式组、文件名行、固定 footer）、`1-overwrite-layer.png`（嵌套层
  替换现有文件?/目标名+路径/返回修改+确认覆盖/背景置灰）、各深色对照
  `-dark`。
- 全门截图（900x700）：tail（含 1003 项分页尾页）、alternate-theme、
  focus-restored；`system-tree.json`（Windows UIA 树）。
- **Q-01 复现并归因**：同帧其余文本完整而 ①分页文本 `第 11 / 11 页`
  可见为 `第 11 /`（q01-pagination-crop.png）；②路径值 `\\?\D:\…` 渲染
  丢失 `D:` 后反斜杠（q01-pathinput-crop.png；MCP value 完整）。归因公共
  渲染器文本 shaping（input/text 两原语）——**具名修复依赖
  DEBT-006**（docs/plans/debts.md），受影响视觉 AC 不以本计划关闭。

## Gate 修正记录（work 内迭代，非回归）

1. fd_command 新 op 的 serde tag 为 camelCase（`selectCurrent` 等）——首版
   snake_case 全部反序列化失败，由组件级测试暴露并修正。
2. `SetFileName` 增本地回显（同 `SetPath` 形态），避免输入回写抖动。
3. Gate 脚本：save_name 等待接受按钮块脱离 disabled（防 canAccept 泵前
   竞态）；MCP 只读工具瞬态超时重试；等待预算按窄窗/主题切换放宽到 15s；
   consumer 端 dirt_by_cut 基线先于 press 读取。
