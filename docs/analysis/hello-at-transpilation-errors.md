# hello.at 转译错误分析

## 源文件 (hello.at)

```auto
use auto.ui: View, widget, app, center

widget Hello {
    // data fields
    msg str

    // view
    fn view() View {
        text(msg)
    }
}

// Create an App
app CounterExample {
    center {
        Hello("Hello, World!")
    }
}
```

## 预期的 Rust 输出

```rust
use auto_ui::{Component, View};

#[derive(Debug)]
pub struct Hello {
    pub msg: String,
}

impl Component for Hello {
    type Msg = ();

    fn on(&mut self, _msg: Self::Msg) {}

    fn view(&self) -> View<()> {
        View::text(&self.msg)
    }
}

fn main() -> iced::Result {
    iced::run(
        |ui| ui.view_iced(),
        AutoUI::new(Hello {
            msg: "Hello, World!".to_string()
        })
    )
}
```

## 转译错误清单

### E1. Use 语句未处理
**位置**: Line 1
```auto
use auto.ui: View, widget, app, center
```
**问题**:
- 转译器不识别 `use` 关键字
- 不知道如何处理路径 `auto.ui`
- 不支持导入列表

**解决方案**:
- 解析 use 语句
- 将路径转换为 Rust 模块路径
- 生成 `use auto_ui::{View, Component};`

### E2. Widget 关键字不匹配
**位置**: Line 3
```auto
widget Hello { ... }
```
**问题**:
- auto-lang parser 只识别 `type` 关键字，不识别 `widget`
- 需要预处理或扩展 parser

**解决方案**:
- 方案A: 预处理：将 `widget` 替换为 `type` 后再解析
- 方案B: 扩展 parser：添加 `widget` 作为 `type` 的别名
- 方案C: 自定义解析：手动解析 widget 定义

### E3. 字段默认值缺失
**位置**: Line 5
```auto
msg str
```
**问题**:
- 没有 Rust 字段的默认值
- `String` 类型不支持默认值

**解决方案**:
- 为 String 字段添加 `Default` trait 实现
- 使用 `Option<String>` 或在构造函数中初始化

### E4. View 返回类型不匹配
**位置**: Line 8
```auto
fn view() View { ... }
```
**问题**:
- 返回 `View` 而不是 `View<Msg>`
- 需要泛型参数

**解决方案**:
- 解析时推断 widget 是否有消息
- 如果无消息，生成 `View<()>`
- 如果有消息，需要先解析 Message 定义

### E5. 表达式解析：函数调用
**位置**: Line 10
```auto
text(msg)
```
**问题**:
- `text(msg)` 是一个函数调用表达式
- 不是 AST 的内置表达式类型
- 需要识别这是 View::text() 的调用

**解决方案**:
- 建立表达式类型映射表
- `text(expr)` → `View::text(expr)`
- `button(label, msg)` → `View::button(label, msg)`
- 需要处理参数引用（`msg` vs `&self.msg`）

### E6. App 关键字未处理
**位置**: Line 14
```auto
app CounterExample { ... }
```
**问题**:
- `app` 不是 Auto 语言的关键字
- 这是 auto-ui 特定的应用定义语法

**解决方案**:
- 识别 `app` 语句
- 生成 main() 函数
- 将 app 内容包装为初始组件

### E7. Widget 实例化语法
**位置**: Line 16
```auto
Hello("Hello, World!")
```
**问题**:
- 这是 Widget 构造语法
- 类似于函数调用但用于创建实例
- 需要解析参数并映射到字段

**解决方案**:
- 识别 Widget 名称
- 解析参数列表
- 生成 Rust 结构体实例化代码
- 处理字段初始化顺序

### E8. 嵌套布局解析
**位置**: Lines 15-17
```auto
center {
    Hello("Hello, World!")
}
```
**问题**:
- `center` 是布局容器
- 需要递归解析子节点
- 需要生成嵌套的 View 树

**解决方案**:
- 识别布局组件（center, col, row）
- 递归解析子节点
- 生成 View 树结构：
  ```rust
  View::container(
      View::empty()  // Hello 组件会在这里
  ).center().build()
  ```

### E9. 字符串字面量
**位置**: Line 16
```auto
"Hello, World!"
```
**问题**:
- 需要正确处理字符串字面量
- 转义字符处理

**解决方案**:
- 使用 Auto 语言的字符串解析
- 生成 Rust 字符串字面量

### E10. 类型推断和泛型
**位置**: 整个文件
**问题**:
- 需要推断 `view()` 返回类型是 `View<()>`
- 需要推断字段类型（`str` → `String`）

**解决方案**:
- 建立类型映射表
- 在 transpile 时添加类型注解
- 处理类型转换

## 优先级排序

### P0 (必须实现)
1. **E2: Widget 关键字** - 核心结构
2. **E3: 字段类型映射** - 数据结构基础
3. **E5: 表达式解析** - View 树核心

### P1 (重要)
4. **E7: Widget 实例化** - 组件使用
5. **E8: 嵌套布局** - 复杂 UI 支持
6. **E6: App 定义** - 应用入口

### P2 (改进)
7. **E1: Use 语句** - 模块组织
8. **E10: 类型推断** - 更好的用户体验

## 实现建议

### 阶段 1: 手动解析器（短期）
不使用 auto-lang parser，手动解析 .at 文件：

```rust
fn parse_at_file(code: &str) -> Result<ParsedWidget, String> {
    // 手动实现简单的 parser
    // 识别 widget, type, fn, 等关键字
    // 提取字段、方法、表达式
}
```

**优点**:
- 完全控制
- 不依赖 auto-lang 编译问题
- 可以专门针对 auto-ui 语法优化

**缺点**:
- 需要重新实现词法和语法分析
- 维护成本高

### 阶段 2: 预处理 + auto-lang parser（中期）
添加预处理层：

```bash
.at file → [预处理: widget → type] → .auto file → [auto-lang parser] → AST
```

**优点**:
- 利用现有的 auto-lang 基础设施
- 保持与 Auto 语言兼容

**缺点**:
- 需要修复 auto-lang 编译问题
- 可能与其他 Auto 代码冲突

### 阶段 3: 扩展 auto-lang（长期）
向 auto-lang 贡献 `widget` 支持

**优点**:
- 从根本上解决问题
- 对整个 Auto 生态系统有益

**缺点**:
- 需要社区协作
- 时间周期长

## 立即可行的方案

**推荐：阶段 1 手动解析器**

实现一个简单的 parser，专门处理 .at 文件中的 auto-ui 特定语法：

1. 词法分析：识别关键字（widget, fn, type, str, int, 等）
2. 语法分析：
   - 解析 widget 定义
   - 解析字段列表
   - 解析方法（view, on）
   - 解析表达式
   - 解析 app 定义
3. 代码生成：
   - 生成 struct 定义
   - 生成 Component impl
   - 生成 main() 函数

**预计工作量**: 2-3 天
**复杂度**: 中等
**风险**: 低

## 测试策略

### T1. 最小示例
```auto
widget Test {
    x int
    fn view() View {
        text("test")
    }
}
```

### T2. 带字段引用
```auto
widget Test {
    msg str
    fn view() View {
        text(msg)
    }
}
```

### T3. 简单嵌套
```auto
app Main {
    Test()
}
```

### T4. 完整示例
hello.at 本身

## 下一步行动

1. **实现手动 parser** (推荐)
   - 创建 `src/parser.rs`
   - 实现简单的词法和语法分析
   - 支持 widget、字段、方法、表达式

2. **生成完整代码**
   - 完善 `AutoUITrans::transpile_file()`
   - 集成 parser
   - 测试 hello.at 转译

3. **验证和优化**
   - 测试转译结果可编译
   - 运行生成的 Rust 代码
   - 添加错误提示
