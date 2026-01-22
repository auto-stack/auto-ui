# AutoUI Run 命令 - 快速使用指南

## 🚀 一键运行 .at 文件

```bash
# 基本用法
cargo run --package auto-ui-transpiler-cli -- run <文件.at>

# 示例
cargo run --package auto-ui-transpiler-cli -- run scratch/text_simple.at
```

## 📋 参数说明

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `<INPUT>` | .at 文件路径 | 必需 |
| `-b, --backend` | 后端选择 (gpui/iced) | gpui |
| `--keep-temp` | 保留生成的临时文件 | false |

## 🔄 工作流程

```
.at 文件
    ↓ [transpile]
Rust 组件代码
    ↓ [add Default + main]
完整可运行示例
    ↓ [cargo run]
GPUI/Iced 窗口 ✨
```

## 📝 完整示例

### 示例 1：简单文本

**scratch/hello.at**:
```auto
type Hello {
    msg str = "Hello"

    fn view() {
        text(msg)
    }
}
```

**运行**:
```bash
cargo run --package auto-ui-transpiler-cli -- run scratch/hello.at -b gpui
```

### 示例 2：带布局

**scratch/myapp.at**:
```auto
type MyApp {
    title str = "My App"

    fn view() {
        col {
            text(title)
            text("Content")
        }
    }
}
```

**运行**:
```bash
cargo run --package auto-ui-transpiler-cli -- run scratch/myapp.at -b gpui
```

## ⚠️ 当前限制

### 1. 默认值

所有组件默认使用：
```rust
Self::new("Hello from Auto!".to_string())
```

如果你的组件有多个字段或不同类型：
- 使用 `--keep-temp` 保留生成的文件
- 手动编辑 `Default` 实现

### 2. 字段类型

当前只支持字符串字段。如果你的组件有其他类型（int, bool 等），需要手动修改 Default 实现。

### 3. Backend 状态

| Backend | 状态 |
|---------|------|
| GPUI | ✅ 完全支持 |
| Iced | 🚧 基础支持（待测试） |

## 🔧 调试技巧

### 查看生成的代码

```bash
# 保留临时文件
cargo run --package auto-ui-transpiler-cli -- run scratch/hello.at --keep-temp

# 查看生成的文件
cat crates/auto-ui-gpui/examples/auto_ui_run_hello.rs
```

### 手动修改并运行

```bash
# 1. 生成代码（不运行）
cargo run --package auto-ui-transpiler-cli -- file scratch/hello.at scratch/hello_gen.rs

# 2. 查看并修改
cat scratch/hello_gen.rs

# 3. 手动运行
# （复制代码到 examples 目录，然后运行）
```

## 💡 提示

- 首次运行会编译所有依赖，可能需要几分钟
- 后续运行会快很多（增量编译）
- 窗口关闭后临时文件会自动清理（除非用 --keep-temp）
- 如需调试，添加 `--keep-temp` 参数

## 🎯 更多信息

- [COMMAND_RUN.md](COMMAND_RUN.md) - 完整文档
- [QUICKSTART_GPUI.md](QUICKSTART_GPUI.md) - 快速开始
- [GPUI_TROUBLESHOOTING.md](GPUI_TROUBLESHOOTING.md) - 问题排查
