# examples/ui 主题迁移初步清单

> 2026-10-29，只读静态盘点；本表不是已完成迁移或运行验收报告。关联：[总体约定](theme-conventions-and-pilot.md)、[auto-ui/PLAN-007](../../plans/007-autoui-tree-theme-and-examples.md)。
> 归属更新：本表记录的源为 auto-lang/examples/ui；同计划拟整体归位 auto-ui/examples/ui，随后进行主题改写。当前源码未搬迁，见[源码搬迁约定](examples-ownership-and-relocation.md)。
> 观测 auto-lang HEAD：`556af8a084e10bd8fd5bd9dc96c4428096fc6db3`；实际读取工作目录，不能以 HEAD 冒充含 WIP 源码的指纹。扫描文件集合的 SHA-256（按路径排序，路径与原始字节以 NUL 分隔）：`271cee9b0955ec2f1f400c48a01e1cd209b332ce237030ec3757114525d361c5`。

## 1. 范围与计数限制

本次发现 37 个直接子目录，其中 35 个编号目录；031-image-viewer 与 031-paint 是不同目录，不能按数字去重。stylekit 是共享库，remote-frame-e2e 是验证夹具，032-handler-async-probe 为编号验证示例。正式迁移按目录身份与用途逐项记录，不能拿历史28-app展示数字代替当前清单。

扫描优先 src/front 中的 .at；无该目录时检查本目录 .at，排除 node_modules/target/dist/.git/references/tests。旧语义命中是 bg/text/border/ring 的 background、foreground、card、primary、secondary、muted、accent、destructive、sidebar 等角色类；固定色命中是 hex、具名色阶和 black/white 工具类。去掉以 // 开头的整行，其余是词法启发，不是解析后的生效值审计。

命中数不代表新系统支持/不支持：语义命中不能证明完整换肤；固定色命中可能属于图表、画布或业务内容，不能盲改；零命中不证明没有动态字符串、SVG、旧 model 配色、生成器/第三方控件等固定样式。新主题迁移验收还需逐 app 追踪 import/组件、pac/宿主入口与实际 Vue/VM画面。

## 2. 逐目录初始状态

| 目录 | 用途 | 扫描 .at 数 | 旧语义命中 | 固定色候选命中 | 迁移状态 |
|---|---|---|---|---|---|
| 001-helloworld | 应用示例 | 1 | 1 | 0 | 后继逐项迁移；未迁移 |
| 002-counter | 应用示例 | 1 | 0 | 0 | 首批真实迁移；未迁移 |
| 003-converter | 应用示例 | 1 | 4 | 0 | 后继逐项迁移；未迁移 |
| 004-profile-card | 应用示例 | 1 | 14 | 1 | 后继逐项迁移；未迁移 |
| 005-login | 应用示例 | 1 | 12 | 0 | 首批真实迁移；未迁移 |
| 006-hero-section | 应用示例 | 1 | 6 | 0 | 后继逐项迁移；未迁移 |
| 007-stats-board | 应用示例 | 1 | 16 | 14 | 后继逐项迁移；未迁移 |
| 008-pricing-table | 应用示例 | 1 | 59 | 0 | 后继逐项迁移；未迁移 |
| 009-article-feed | 应用示例 | 1 | 65 | 0 | 后继逐项迁移；未迁移 |
| 010-contact-form | 应用示例 | 1 | 34 | 7 | 后继逐项迁移；未迁移 |
| 011-calculator | 应用示例 | 2 | 146 | 23 | 后继逐项迁移；未迁移 |
| 012-clock | 应用示例 | 1 | 79 | 31 | 后继逐项迁移；未迁移 |
| 013-todo | 应用示例 | 4 | 24 | 0 | 后继逐项迁移；未迁移 |
| 014-weather | 应用示例 | 3 | 76 | 14 | 后继逐项迁移；未迁移 |
| 015-notes | 应用示例 | 5 | 74 | 10 | 后继逐项迁移；未迁移 |
| 016-calendar | 应用示例 | 3 | 39 | 10 | 后继逐项迁移；未迁移 |
| 017-chat | 应用示例 | 6 | 63 | 0 | 后继逐项迁移；未迁移 |
| 018-book-reader | 应用示例 | 12 | 116 | 6 | 后继逐项迁移；未迁移 |
| 019-video-app | 应用示例 | 5 | 151 | 34 | 后继逐项迁移；未迁移 |
| 020-music-player | 应用示例 | 8 | 256 | 22 | 后继逐项迁移；未迁移 |
| 021-blog-viewer | 应用示例 | 5 | 111 | 0 | 后继逐项迁移；未迁移 |
| 022-kanban | 应用示例 | 3 | 34 | 1 | 后继逐项迁移；未迁移 |
| 023-realworld | 应用示例 | 10 | 67 | 5 | 后继逐项迁移；未迁移 |
| 024-charts | 应用示例 | 7 | 26 | 35 | 后继逐项迁移；未迁移 |
| 025-sys-monitor | 应用示例 | 7 | 148 | 45 | 后继逐项迁移；未迁移 |
| 026-database | 应用示例 | 6 | 131 | 3 | 后继逐项迁移；未迁移 |
| 027-file-manager | 应用示例 | 8 | 131 | 0 | 后继逐项迁移；未迁移 |
| 028-launcher | 应用示例 | 1 | 38 | 12 | 后继逐项迁移；未迁移 |
| 029-photo-gallery | 应用示例 | 1 | 74 | 15 | 后继逐项迁移；未迁移 |
| 030-video-player | 应用示例 | 5 | 62 | 3 | 后继逐项迁移；未迁移 |
| 031-image-viewer | 应用示例 | 2 | 17 | 0 | 后继逐项迁移；未迁移 |
| 031-paint | 应用示例 | 1 | 57 | 19 | 后继逐项迁移；未迁移 |
| 032-handler-async-probe | 编号验证示例 | 1 | 0 | 0 | 后继逐项迁移；未迁移 |
| 038-minesweeper | 应用示例 | 2 | 0 | 130 | 后继逐项迁移；未迁移 |
| 041-auto-edit | 应用示例 | 7 | 31 | 3 | 后继逐项迁移；未迁移 |
| remote-frame-e2e | 验证夹具 | 5 | 7 | 0 | 配套核验；未迁移 |
| stylekit | 共享样式库 | 1 | 12 | 0 | 配套核验；未迁移 |

## 3. AutoOS 的承接边界

只读核对 D:/autostack/auto-os：main@4b53ee30498debeae484d51114fc717fc6a4b777，apps 内已有多个独立应用/submodule。旧语义类与 dark_mode/accent、pac theme 配置已在部分应用出现：auto-calc 的 pac 注释声明跟随主题链，auto-weather 的源码/pac 声明旧宿主主题合同，auto-explorer 使用语义 token，auto-calendar/auto-paint/auto-notes 等有显式主题配置。因此不能写成“AutoOS 从未接过任何旧主题机制”。没有据此完成全应用有效值或运行审计。

examples/ui 与 AutoOS 独立 app 不是同一份工作目录；教学 demo 改造不自动传播到 app 子模块。本路线后继必须按现行注册表/.gitmodules 对应到实际来源仓和固定提交，再分别迁移、验收、更新 gitlink。源码可有业务差异，但官方主题包保持同一公共来源，不能每 app 复制一份主题常量。

当前表只冻结本次盘点结果，不宣称未来新增目录自动已纳入。正式批次启动时对新增/移动/退役做显式差异对账；所有未迁移项保留，不借重分类删掉原需求。
