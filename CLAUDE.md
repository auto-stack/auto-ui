# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

AutoUI 是基于 Auto 语言和 Rust 实现的 UI 框架，底层采用 GPUI 图形库（来自 Zed 编辑器项目），并参考 GPUI-Component 进行组件设计和实现。

**注意**: 当前 master 分支已被清空，代码需要从 git 历史中恢复或重新开发。

## 构建和运行命令

### 构建整个工作空间
```bash
cargo build
```

### 运行示例
```bash
# Auto 语言示例（需要先从 git 历史恢复代码）
cargo run --package auto-ui --example hello

# GPUI 组件示例
cargo run --package gpui-components-examples --example hello
cargo run --package gpui-components-examples --example counter
cargo run --package gpui-components-examples --example login
cargo run --package gpui-components-examples --example table_simple
cargo run --package gpui-components-examples --example table_full
cargo run --package gpui-components-examples --example docks
```

### 代码生成
```bash
# 从 Auto 语言代码生成 Rust 代码
cargo run --package auto-ui --example gen
```

### 测试
```bash
# 运行所有测试
cargo test

# 运行特定包的测试
cargo test --package auto-ui
```

### Lint 检查
项目使用 rust-analyzer 进行检查，VSCode 设置中已启用 `rust-analyzer.checkOnSave`。

## 代码架构

### 工作空间结构
项目采用 Cargo workspace 结构，包含以下 crates：

- **crates/auto-ui**: 核心 UI 库
  - `src/lib.rs`: 库入口，导出主要模块
  - `src/app.rs`: 应用程序结构（目前未使用）
  - `src/layout.rs`: 布局相关
  - `src/bridge.rs`: GPUI-Component 桥接层，重新导出常用组件
  - `src/assets.rs`: 资源管理
  - `src/dyna/`: 动态组件系统
    - `dyna.rs`: 动态组件核心
    - `button.rs`, `label.rs`: 基础动态组件
    - `snip.rs`: 代码片段处理
    - `story.rs`: Story 集成
  - `src/trans/`: Auto 语言到 Rust 的转换层
    - `trans.rs`: 转换器核心
    - `templates.rs`: 代码生成模板
  - `src/widgets/`: 高级组件
    - `table.rs`: 表格组件
    - `story.rs`: Story 包装器
  - `examples/`: Auto 语言示例（.at 文件）和生成的 Rust 代码（.rs 文件）

- **crates/gpui-components-examples**: GPUI-Component 的使用示例
  - `examples/`: 各种 UI 组件示例

### 依赖关系

**核心依赖**:
- `auto-lang`: Auto 语言的核心库（AST、解析器、求值器）
- `auto-val`: Auto 语言的值类型系统
- `auto-gen`: 代码生成器
- `auto-atom`: 原子操作和状态管理

**UI 框架**:
- `gpui`: Zed 编辑器的底层 UI 框架（从官方 git 仓库引用）
- `gpui-component`: GPUI 组件库（tag: v20250611）
- `gpui-story`: Storybook 风格的组件展示工具
- `markdown`: Markdown 渲染支持

**重要**: 项目使用特定版本的 GPUI（tag v20250611），如需更新 GPUI 版本，运行：
```bash
cargo update -p gpui --precise <commit-hash>
```

### 核心架构概念

1. **双层系统**:
   - **Auto 语言层**: 使用 `.at` 文件编写声明式 UI 代码
   - **Rust 层**: 通过转换器将 Auto 代码编译为 Rust 代码

2. **转换流程**:
   ```
   .at 文件 → Parser → AST → Trans → Rust 代码 → GPUI 渲染
   ```

3. **动态组件系统 (dyna)**:
   - 运行时解析和渲染 UI 组件
   - 支持热重载（通过 file watcher）
   - 基于代码片段（Snippet）动态生成组件

4. **组件桥接**:
   - `bridge.rs` 重新导出 GPUI-Component 的常用组件
   - 提供统一的 API 访问 Button, Label, Input 等组件

### Auto 语言语法示例

```auto
type Hello as Widget {
    msg str = "Hello World"
    button_label str = "Click"

    fn view() {
        center {
            col {
                label(self.msg) {}
                button(self.button_label) {
                    onclick: "button-clicked"
                }
            }
        }
    }

    fn on(ev str) {
        msg = "Hello Button clicked"
    }
}

fn main() {
    app("Hello Example") {
        hello() {}
    }
}
```

关键特性:
- 类型声明：`type Name as Widget { ... }`
- 字段定义：`field_name Type = default_value`
- 视图函数：`fn view() { ... }`
- 事件处理：`fn on(ev str) { ... }`
- 嵌套布局：`center`, `col` 等布局组件

### 常用模式

1. **创建新组件**:
   - 在 `crates/auto-ui/src/dyna/` 或 `src/widgets/` 中添加组件
   - 在 `dyna.rs` 的 `DynaWidget::parse()` 中注册
   - 在 `bridge.rs` 中重新导出（如果是静态组件）

2. **添加新示例**:
   - 在 `crates/auto-ui/examples/` 创建 `.at` 文件
   - 运行 `cargo run --package auto-ui --example gen` 生成 `.rs` 文件
   - 添加到 `examples/gen.rs` 的 examples 列表中

3. **使用 GPUI-Component**:
   ```rust
   use gpui_component::{Button, Label, h_flex, v_flex};

   fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
       h_flex().gap_4().child(Label::new("Hello"))
   }
   ```

## 开发工作流

1. **修改 Auto 语言核心**: 需要同步更新 `auto-lang` 依赖（本地路径: `../auto-lang/crates/auto-lang`）

2. **更新 GPUI 版本**:
   - 检查 GPUI-Component 兼容性
   - 更新 `Cargo.toml` 中的 gpui 引用
   - 运行 `cargo update -p gpui`

3. **添加资源文件**:
   - SVG 图标放在 `assets/icons/`
   - 使用 `rust-embed` 在代码中嵌入资源

4. **调试**:
   - 使用 VSCode 的 rust-analyzer 进行代码检查
   - 添加 `dbg!()` 或 `println!()` 进行调试
   - 运行示例程序查看 UI 效果

## 相关项目

- [GPUI](https://gpui.rs) - 底层图形库
- [Zed Editor](https://zed.dev/) - 使用 GPUI 的代码编辑器
- [GPUI-Component](https://github.com/longbridgeapp/gpui-component) - GPUI 组件库
- auto-lang (本地路径: `../auto-lang`) - Auto 语言实现

## 注意事项

- 项目使用 Rust 2021 edition
- 当前 GPUI-Component 版本：v20250611
- Auto 语法仍在演进中， breaking changes 可能发生
- 文件监听功能使用 `notify` 和 `notify-debouncer-mini`
- Git 历史包含完整的代码实现，可通过 `git show <commit>:<file>` 查看历史代码

## 文档管理规范

### Phase 总结文档规则

**重要**: 完成 Plan 文档中某个 Phase 的实现后，**禁止**创建独立的总结文档（如 `phase1-summary.md`, `phase1-abstraction-implementation.md`）。

**正确做法**：
1. 直接在对应的 Plan 文件中更新 Phase 状态
2. 将实现总结、技术亮点、经验教训等内容合并到 Plan 文件本身的对应 Phase 章节
3. 更新里程碑进度表
4. 更新"下一步行动"章节

**理由**：
- **单一信息源**: Plan 文件应该是项目状态的唯一真实来源
- **易于维护**: 避免信息分散在多个文件中，减少同步维护成本
- **更好的可读性**: 在一个文件中可以看到完整的计划、进度和总结
- **减少冗余**: 避免重复信息，保持文档简洁

**示例**：
```markdown
### Phase 1: 基础设施 ✅ **已完成**（2025-01-19）

#### 1.1 项目结构搭建 ✅
- [x] 创建 Cargo workspace
- [x] 设置基础目录结构

#### 1.2 核心实现 ✅
- [x] 实现 Component trait
- [x] 实现 View enum

## Phase 1 完成总结 ✅

### 完成日期
2025-01-19

### 主要成果
1. 项目基础设施搭建完成
2. 核心抽象层实现
...

### 技术亮点
- 类型安全的消息传递
- 零成本抽象设计
...
```

**文档删除**：
- 如果存在独立的 phase 总结文档，应在内容合并到 plan 文件后立即删除
- 只保留 Plan 文件作为该 Phase 的唯一文档
