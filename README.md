# AutoUI

基于 Auto 语言的跨平台 UI 框架

## 项目概述

AutoUI 是一个使用 Auto 语言作为描述层，支持多种 UI 框架后端的跨平台 UI 库。

### 长期目标

支持 6+ 种 UI 框架后端：
- PC 端：gpui, iced
- Web 端：vue.js
- 移动端：Jetpack Compose, 鸿蒙 UI
- 嵌入式：LVGL

### 短期目标（Phase 1-5）

实现 Auto 语言描述层 + iced/gpui 双后端的桌面跨平台 UI 库。

## 项目结构

```
auto-ui/
├── crates/
│   ├── auto-ui/            # 核心框架（底层无关）
│   ├── iced-examples/      # iced 示例（✅ 可用）
│   └── gpui-examples/      # gpui 示例（开发中）
├── scratch/                # Auto 语言原型
├── docs/
│   ├── plans/              # 实施计划
│   └── phase1-summary.md   # Phase 1 总结
├── Cargo.toml              # Workspace 配置
├── CLAUDE.md               # Claude Code 指南
├── README.md               # 本文档
└── README.cn.md            # 中文说明
```

## 快速开始

### 前置要求

- Rust 2021 edition
- Windows / macOS / Linux

### 运行 iced 示例

```bash
# Hello World
cargo run --bin hello

# Counter 示例
cargo run --bin counter

# Button 示例
cargo run --bin button
```

### 编译项目

```bash
# 编译核心框架
cargo build -p auto-ui

# 编译 iced 示例
cargo build -p iced-examples

# 编译可用的包
cargo build -p auto-ui -p iced-examples
```

## 当前状态

### ✅ Phase 1 - 基础设施（已完成）

- [x] Cargo workspace 搭建
- [x] auto-ui 核心 crate
- [x] iced-examples 可运行示例
- [x] 基础文档

详细内容见：[Phase 1 总结](docs/phase1-summary.md)

### 🔄 Phase 2 - 核心抽象层（进行中）

- [ ] 完善 Widget trait
- [ ] 实现布局系统
- [ ] 状态管理
- [ ] iced 后端适配

## 技术栈

### 当前使用

- **Rust** 2021 edition
- **iced** 0.14.0 - UI 框架（主要后端）
- **gpui-component** 0.5.0 - UI 框架（次要后端，开发中）
- **auto-lang** - Auto 语言核心（待集成）

### 设计理念

AutoUI 采用类似 ELM 的消息驱动架构：

```
User Action → Message → Update → State Change → View → UI
```

## 文档

- [项目计划](docs/plans/001-starting-plan.md) - 完整的实施计划
- [Phase 1 总结](docs/phase1-summary.md) - 第一阶段完成情况
- [CLAUDE.md](CLAUDE.md) - Claude Code 开发指南
- [README.cn.md](README.cn.md) - 项目说明（中文）

## Auto 语言示例

在 [scratch/](scratch/) 目录中可以看到 Auto 语言的原型示例：

- [counter.at](scratch/counter.at) - 计数器组件
- [button.at](scratch/button.at) - 按钮组件
- [layouts.at](scratch/layouts.at) - 布局示例
- [login.at](scratch/login.at) - 登录表单

```auto
type Counter is Widget {
    count int

    fn view() View {
        col {
            button("+") { onclick: do_inc }
            text(count)
            button("-") { onclick: do_dec }
        }
    }

    fn do_inc() { count += 1 }
    fn do_dec() { count -= 1 }
}
```

## 贡献

当前项目处于早期开发阶段，欢迎贡献！

## 参考

- [iced](https://github.com/iced-rs/iced) - Rust GUI 库
- [gpui](https://github.com/zed-industries/zed) - Zed 编辑器的渲染引擎
- [gpui-component](https://github.com/longbridgeapp/gpui-component) - GPUI 组件库
- [ELM](https://guide.elm-lang.org/) - 函数式 UI 架构

## 许可证

待定
