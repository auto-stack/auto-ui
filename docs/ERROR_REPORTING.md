# 错误报告改进文档

## 概述

本文档描述了为 AutoUI Transpiler 添加的 miette 错误报告功能，以及如何完成剩余的修复步骤。

## 已完成的工作

### 1. 添加 miette 依赖

在 `crates/auto-ui-transpiler-cli/Cargo.toml` 中添加了 miette 库：

```toml
miette = { version = "7.0", features = ["fancy"] }
```

### 2. 创建错误处理模块

创建了 `crates/auto-ui-transpiler-cli/src/error.rs`，定义了 `TranspileError` 枚举，支持：
- ParseError - 带源代码位置的解析错误
- IoError - 文件 I/O 错误
- CodeGenError - 代码生成错误

### 3. 改进 run_file 函数

在 `main.rs` 中增强了错误报告功能：

```rust
/// Extract byte position from error message
fn extract_error_position(error_msg: &str) -> Option<usize> {
    // Try to extract offset from error messages like "offset: SourceOffset(377)"
    if let Some(start) = error_msg.find("SourceOffset(") {
        let end = error_msg[start..].find(')')?;
        let num_str = &error_msg[start + 13..start + end];
        num_str.parse().ok()
    } else {
        None
    }
}

/// Show source code context around error position
fn show_error_context(source: &str, pos: usize) {
    let lines: Vec<&str> = source.lines().collect();

    // Find the line containing the error position
    let mut current_pos = 0;
    let mut error_line = 0;
    let mut error_col = 0;

    for (i, line) in lines.iter().enumerate() {
        if current_pos + line.len() >= pos {
            error_line = i;
            error_col = pos - current_pos;
            break;
        }
        current_pos += line.len() + 1; // +1 for newline
    }

    // Show context: 2 lines before and after
    let start_line = error_line.saturating_sub(2);
    let end_line = (error_line + 3).min(lines.len());

    eprintln!("{}", style("Source code context:").cyan().bold());
    eprintln!();

    for i in start_line..end_line {
        let line_num = i + 1;
        let prefix = if i == error_line {
            format!("{} > ", style(line_num).red().bold())
        } else {
            format!("{} | ", style(line_num).dim())
        };

        eprintln!("{}{}", prefix, lines[i]);

        // Show error indicator
        if i == error_line && error_col > 0 {
            let indent = "    ".len() + line_num.to_string().len() + error_col;
            eprintln!("{}{} {}", " ".repeat(indent), style("^").red(), style("error here").red());
        }
    }
    eprintln!();
}
```

## 剩余工作

### 修复 auto-lang 编译错误

需要修复以下文件中的 `match` 语句，添加对 `StoreKind::Mut` 的处理：

1. **`d:\autostack\auto-lang\crates\auto-lang\src\ast\store.rs`**

需要在 4 个 `match` 语句中添加 `StoreKind::Mut` 分支：

```rust
// 在 fmt::Display for Store 中 (第 29 行)
match self.kind {
    StoreKind::Let => write!(f, "(let (name {}){}{})", self.name, ty_str, self.expr),
    StoreKind::Mut => write!(f, "(let mut (name {}){}{})", self.name, ty_str, self.expr),  // 添加这行
    StoreKind::Var => write!(f, "(var (name {}) {})", self.name, self.expr),
    StoreKind::Field => write!(f, "(field (name {}) {})", self.name, self.expr),
    StoreKind::CVar => write!(f, "(cvar (name {}))", self.name),
}

// 在 fmt::Display for StoreKind 中 (第 40 行)
match self {
    StoreKind::Let => write!(f, "let"),
    StoreKind::Mut => write!(f, "let mut"),  // 添加这行
    StoreKind::Var => write!(f, "var"),
    StoreKind::Field => write!(f, "field"),
    StoreKind::CVar => write!(f, "cvar"),
}

// 在 AtomWriter for Store 中 (第 57 行)
let kind_name = match self.kind {
    StoreKind::Let => "let",
    StoreKind::Mut => "let_mut",  // 添加这行
    StoreKind::Var => "var",
    StoreKind::Field => "field",
    StoreKind::CVar => "cvar",
};

// 在 ToNode for Store 中 (第 77 行)
let node_name = match &self.kind {
    StoreKind::Let => "let",
    StoreKind::Mut => "let_mut",  // 添加这行
    StoreKind::Var => "var",
    StoreKind::CVar => "cvar",
    StoreKind::Field => "field",
};
```

### 测试改进的错误显示

修复编译错误后，可以测试新的错误报告功能：

```bash
# 创建一个包含错误的文件
cat > scratch/error_test.at << 'EOF'
type Test {
    count int = 0

    fn view() {
        col {
            text("Hello")
            text("World".  // 缺少闭合括号
        }
    }
}
EOF

# 运行 transpiler 查看改进的错误信息
cargo run --package auto-ui-transpiler-cli -- file scratch/error_test.at scratch/error_test.rs
```

期望输出：
```
✖ Transpilation failed

Failed to parse scratch/error_test.at: MultipleErrors { count: 2, plural: "s", ... }

For better error reporting, try:
  1. Check the syntax around the reported location
  2. Make sure all braces are properly closed
  3. Verify keyword spelling (type, fn, is, etc.)

Source code context:

9 |         col {
10 |             text("Hello")
11 >             text("World".
                ^ error here

```

## 完整的错误显示特性

改进后的错误报告将包含：

1. **清晰的错误标题** - 使用 ✖ 符号和红色高亮
2. **详细错误信息** - 保留完整的错误消息
3. **源代码上下文** - 显示错误位置前后 2 行代码
4. **错误指示器** - 用 `^` 标记具体错误位置
5. **修复建议** - 提供常见的语法错误修复提示

## 与 miette 集成的未来改进

一旦 auto-lang 编译错误修复，可以进一步集成 miette 的完整功能：

1. **使用 miette::Diagnostic** - 为 TranspileError 实现完整的诊断 trait
2. **支持多个相关错误** - 使用 `#[related]` 属性显示多个错误
3. **彩色输出** - 使用 miette 的 fancy 报告处理器
4. **错误代码** - 使用 `#[diagnostic(code(auto_ui::error_code))]` 添加错误代码
5. **帮助提示** - 使用 `#[help("...")]` 添加修复建议

## 相关文件

- `crates/auto-ui-transpiler-cli/Cargo.toml` - 添加了 miette 依赖
- `crates/auto-ui-transpiler-cli/src/error.rs` - 错误类型定义
- `crates/auto-ui-transpiler-cli/src/main.rs` - 改进的错误处理逻辑
- `d:\autostack\auto-lang\crates\auto-lang\src\ast/store.rs` - 需要添加 StoreKind::Mut 处理
