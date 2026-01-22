# AutoUI Run 命令 - 一键运行 .at 文件

## 🚀 快速开始

```bash
# 运行 Auto 文件（默认使用 GPUI 后端）
cargo run --package auto-ui-transpiler-cli -- run hello.at

# 指定后端
cargo run --package auto-ui-transpiler-cli -- run hello.at -b gpui
cargo run --package auto-ui-transpiler-cli -- run hello.at -b iced

# 保留生成的临时文件
cargo run --package auto-ui-transpiler-cli -- run hello.at -b gpui --keep-temp
```

## 📋 命令格式

```
auto-ui-transpile run [INPUT] [OPTIONS]
```

### 参数

- `<INPUT>` - 要运行的 .at 文件路径
- `-b, --backend <BACKEND>` - 选择后端（gpui 或 iced，默认：gpui）
- `--keep-temp` - 保留临时生成的文件

## 🔄 工作流程

`run` 命令自动执行以下步骤：

1. **转译** (.at → .rs)
   - 解析 Auto 语言代码
   - 生成 Rust 组件代码

2. **生成示例** (添加 main 和 Default)
   - 自动添加 Default trait 实现
   - 生成完整的可运行示例

3. **编译并运行**
   - 编译生成的示例
   - 启动应用程序

4. **清理** (可选)
   - 默认自动删除临时文件
   - 使用 `--keep-temp` 保留文件用于调试

## 📝 示例

### 示例 1：简单文本组件

文件 `scratch/hello.at`:
```auto
type Hello {
    msg str = "Hello World"

    fn view() {
        text(msg)
    }
}
```

运行：
```bash
cargo run --package auto-ui-transpiler-cli -- run scratch/hello.at -b gpui
```

### 示例 2：带布局的组件

文件 `scratch/my_app.at`:
```auto
type MyApp {
    title str = "My App"
    content str = "Content"

    fn view() {
        col {
            text(title)
            text(content)
        }
    }
}
```

运行：
```bash
cargo run --package auto-ui-transpiler-cli -- run scratch/my_app.at -b gpui
```

## 🔍 调试技巧

### 查看生成的代码

使用 `--keep-temp` 保留生成的文件：

```bash
cargo run --package auto-ui-transpiler-cli -- run scratch/hello.at --keep-temp
```

生成的文件位置：
- GPUI: `crates/auto-ui-gpui/examples/auto_ui_run_hello.rs`
- Iced: `crates/auto-ui-iced/examples/auto_ui_run_hello.rs`

### 查看转译结果

如果只想看转译结果，不运行：

```bash
cargo run --package auto-ui-transpiler-cli -- file scratch/hello.at
```

## ⚙️ 默认值处理

`run` 命令会为组件生成默认的 `Default` 实现：

```rust
impl Default for MyComponent {
    fn default() -> Self {
        Self::new(
            "Hello from Auto!".to_string(),
            // 如果组件有多个字段，可能需要手动调整
        )
    }
}
```

如果你的组件有多个字段或需要特定的默认值：
1. 使用 `--keep-temp` 保留生成的文件
2. 手动编辑 `Default` 实现
3. 直接运行生成的示例

## 📊 当前限制

1. **默认值推断**: 所有字段默认使用字符串 "Hello from Auto!"
   - 需要手动修改 Default 实现以使用正确的默认值

2. **new() 参数**: 假设组件的 `new()` 方法接受字符串参数
   - 如果组件有多个字段或不同类型的参数，需要手动调整

3. **后端支持**:
   - ✅ GPUI: 完全支持
   - 🚧 Iced: 基础支持（需要测试）

## 🎯 下一步

- 尝试运行自己的 .at 文件
- 使用 `--keep-temp` 查看和修改生成的代码
- 查看完整文档：[RUNNING_GENERATED_CODE.md](RUNNING_GENERATED_CODE.md)
- 故障排查：[GPUI_TROUBLESHOOTING.md](GPUI_TROUBLESHOOTING.md)
