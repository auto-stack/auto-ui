# Plan 011: Auto 动态解释器设计

**Status**: 🔨 In Progress
**Created**: 2025-01-24
**Priority**: High
**Complexity**: High
**Estimated Timeline**: 4-6 周
**Last Updated**: 2025-01-27

## Overview

设计并实现一个完整的 Auto 语言**动态解释器**，支持**真正的热重载**（hot-reload）开发体验。与当前的静态转译方案（.at → Rust → 编译 → 运行）不同，动态解释器在运行时直接解释 Auto 代码，修改文件后**下一帧即可看到界面变化**，大幅提升开发效率。

## 进展追踪

### ✅ 已完成 (2025-01-26 更新)

#### 1. 基础设施修复
- ✅ 修复 `node_converter.rs` 中的 API 调用问题
  - 将 `child_node.args.args.first()` 改为 `child_node.main_arg()`
  - 确保与 auto-val 的 API 兼容性
- ✅ 为 auto-ui bin 添加 `required-features = ["cli"]` 配置
  - 确保只在启用 cli feature 时才构建二进制文件
  - 避免依赖项缺失导致的编译错误

#### 2. 解释器核心实现 (Phase 1-2 基础)
- ✅ 创建 `crates/auto-ui/src/interpreter/` 模块
  - `mod.rs` - 模块导出和错误类型定义
  - `bridge.rs` - 连接 auto-lang 和 auto-ui 的桥梁
    - `InterpreterBridge` - 管理 auto-lang::Interpreter 和 Widget 状态
    - `WidgetState` - Widget 运行时状态（字段、缓存视图、脏标记）
    - `DynamicMessage` - 保留类型信息的动态消息
    - 文件加载和代码解释
    - 事件处理接口
    - 热重载支持框架

- ✅ 增强的 Node 转换器 (`node_converter.rs`)
  - 实现 `convert_node_dynamic()` 函数
  - 支持所有基础组件的动态转换：
    - 布局：center, col, row, container, scrollable
    - 元素：text, button, input, checkbox, radio, select
  - 事件处理程序提取（onclick, onchange, ontoggle, onselect）
  - `DynamicMessage` 类型消息生成
  - `SelectCallback` 正确创建

#### 3. 编译状态验证
- ✅ auto-ui 库编译成功（interpreter feature）
  - 0 个错误，只有警告（未使用的导入）
- ✅ auto-ui-gpui 库编译成功（11 个警告）
- ✅ auto-ui-iced 库编译成功（2 个警告）
- ✅ unified-counter 示例编译成功（GPUI 和 Iced 后端）
- ✅ 核心依赖项工作正常

#### 4. 架构验证
- ✅ 验证了基于 auto-lang::Interpreter 的解释流程：
  ```
  .at 文件
     ↓
  auto_lang::Interpreter（解析和求值）
     ↓
  auto_val::Node（AST 结果）
     ↓
  node_converter::convert_node_dynamic（转换为 View）
     ↓
  View<DynamicMessage>（通用 UI 描述）
     ↓
  GPUI/Iced 渲染（待实现）
  ```

#### 5. auto-lang 兼容性修复 (2025-01-27)
- ✅ 修复 auto-lang 编译错误
  - 添加缺失的 TokenKind 类型：`Const`, `Bool`, `Byte`
  - 将 `parse_type_param()` 返回类型从 `TypeParamOld` 更新为 `TypeParam`
  - 解决 GPUI 0.2.2 API 兼容性问题
- ✅ auto-lang 核心库编译通过（仅有警告）

#### 6. GPUI 集成初步实现 (Phase 5 框架) (2025-01-27)
- ✅ 创建 `interpreter-gpui-minimal` 简化演示示例
  - 完整的 GPUI 应用框架（使用 `Application::new()` API）
  - 实现了基础的 UI 布局（标题栏、渲染区、信息面板）
  - 添加了 `Assets` 结构体实现 `AssetSource` trait
  - 修复了 GPUI 0.2.2 API 差异：
    - `ViewContext` → `Context`
    - `px()` 宏用于 `Point`/`Size`
    - 移除了不兼容的方法调用
- ✅ GPUI API 兼容性文档化
  - 记录了 GPUI 0.2.2 的正确使用方式
  - 为后续开发提供了参考示例
- ⚠️ **已知限制**：
  - 由于 GPUI Entity 系统的限制，`DynamicInterpreterComponent` 暂时无法直接嵌入
  - 当前示例使用静态 UI 展示架构，而非实际的动态渲染
  - 需要设计新的架构来绕过 Entity 生命周期限制

### 🔄 进行中

- **Phase 2: Node 转换器增强** - 基础完成，需要添加：
  - [ ] list 和 table 组件的动态转换
  - [ ] 样式元数据提取和类型化消息支持

- **Phase 5: GPUI 集成** - ⚠️ 架构重新设计中
  - ✅ 创建了基础演示框架 (`interpreter-gpui-minimal`)
  - ✅ 验证了 GPUI 0.2.2 API 兼容性
  - 🔄 **技术挑战**：GPUI Entity 系统限制
    - `DynamicInterpreterComponent` 无法在组件创建时初始化（需要 `Context`）
    - Entity 生命周期与解释器需求不匹配
    - 需要设计新的架构模式：
      - 方案 1：使用全局状态管理器绕过 Entity 限制
      - 方案 2：延迟初始化模式（首次 render 时创建解释器）
      - 方案 3：将解释器完全独立于 GPUI Entity 系统
  - [ ] 完善新的架构设计
  - [ ] 实现 View<DynamicMessage> → GPUI 元素的完整映射
  - [ ] 事件处理器连接
  - [ ] 热重载触发和重新渲染

### 📋 待完成

- **Phase 3: 事件路由与消息分发**
  - [ ] 实现 Widget 状态管理
  - [ ] 实现 on() 方法调用
  - [ ] 智能消息路由

- **Phase 4: 热重载集成**
  - [ ] 文件监听器集成
  - [ ] 状态迁移策略
  - [ ] 错误恢复机制

- **Phase 6: 开发者工具**
  - [ ] CLI dev server
  - [ ] 错误覆盖层 UI
  - [ ] 调试工具

## Motivation

### 当前问题：开发周转时间过长

**静态转译流程**：
```
修改 .at 文件
→ 运行转译器生成 Rust 代码
→ cargo compile (30秒-2分钟)
→ 重启应用
→ 查看效果
```

**痛点**：
- ✗ 每次修改都需要完整编译
- ✗ 编译时间长（30秒 - 2分钟）
- ✗ 应用重启丢失状态
- ✗ 开发体验差，反馈循环慢

### 目标：真正的热重载

**动态解释流程**：
```
修改 .at 文件
→ 文件监听器检测变化
→ 解释器重新解析 (50-100ms)
→ 下一帧立即显示变化
```

**优势**：
- ✓ 无需编译，即时反馈
- ✓ 保持组件状态（智能迁移）
- ✓ 开发体验接近 Web 前端
- ✓ 生产期仍可用静态转译保证性能

## 架构设计

### 1. 分层解释器架构

```
┌─────────────────────────────────────────────────────────┐
│                    GPUI Application                     │
└───────────────────────┬───────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────┐
│          DynamicComponent (GPUI Render Wrapper)        │
│  • 实现 Render trait                                   │
│  • 持有 InterpreterRuntime                              │
│  • 将 View<DynamicMessage> 转换为 GPUI 元素            │
└───────────────────────┬───────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────┐
│              InterpreterRuntime                         │
│  ┌─────────────────────────────────────────────────┐  │
│  │  SymbolTable (组件、类型、函数元数据)           │  │
│  │  ComponentInstance (状态、视图缓存、脏标记)      │  │
│  │  EventRouter (消息路由到处理器)                  │  │
│  │  StateManager (状态迁移和持久化)                 │  │
│  └─────────────────────────────────────────────────┘  │
└───────────────────────┬───────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────┐
│              AutoParser (auto-lang 集成)              │
│  • 解析 .at 文件为 ast::Code                          │
│  • 提取 Widget 类型和元数据                           │
│  • 构建 view() 和 on() 方法的 AST                     │
└───────────────────────┬───────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────┐
│              Enhanced NodeConverter                    │
│  • auto_val::Node → View<DynamicMessage>             │
│  • 保留类型元数据用于事件路由                         │
│  • 支持所有 Auto UI 构造                              │
└───────────────────────────────────────────────────────┘
```

### 2. 核心数据结构

```rust
/// 解释器运行时状态
pub struct InterpreterRuntime {
    /// 符号表：所有已加载组件的元数据
    symbol_table: Arc<RwLock<SymbolTable>>,

    /// 活动组件实例 (component_name → instance)
    instances: HashMap<String, Arc<RwLock<ComponentInstance>>>,

    /// 事件路由器：将消息分发到正确的组件
    event_router: EventRouter,

    /// 文件监听器：热重载支持
    file_watcher: Option<FileWatcher>,
}

/// 符号表：存储类型信息
pub struct SymbolTable {
    /// 组件元数据
    components: HashMap<String, WidgetMetadata>,

    /// 类型定义（enum, struct）
    types: HashMap<String, TypeDef>,

    /// 全局函数
    functions: HashMap<String, FunctionSignature>,
}

/// Widget 组件元数据
pub struct WidgetMetadata {
    pub name: String,
    pub fields: Vec<FieldDef>,           // 组件字段（状态）
    pub view_method: Option<ViewMethod>, // view() 方法 AST
    pub on_method: Option<OnMethod>,     // on() 方法 AST
    pub message_type: Option<TypeDef>,   // 消息类型
}

/// 运行时组件实例
pub struct ComponentInstance {
    /// 组件元数据
    metadata: WidgetMetadata,

    /// 字段值（状态）
    state: HashMap<String, Value>,

    /// 缓存的视图树
    cached_view: Option<View<DynamicMessage>>,

    /// 视图脏标记
    view_dirty: bool,
}

/// 增强的动态消息（保留类型信息）
#[derive(Clone, Debug)]
pub enum DynamicMessage {
    /// 字符串事件（向后兼容）
    String(String),

    /// 类型化事件
    Typed {
        component: String,       // 组件名
        event_name: String,      // 事件名（如 "Inc"）
        args: Vec<Value>,        // 事件参数
    },
}
```

## 实现计划

### Phase 1: 符号表与类型提取 (Week 1-2)

**目标**：解析 .at 文件并提取组件元数据，无需完整求值。

#### 1.1 集成 auto-lang Parser

**文件**: `crates/auto-ui/src/interpreter/symbol_table.rs` (新建)

```rust
use auto_lang::Parser;
use auto_lang::ast::{Code, Stmt, TypeDecl, FnDecl};

pub struct SymbolTableBuilder {
    components: HashMap<String, WidgetMetadata>,
}

impl SymbolTableBuilder {
    /// 从 .at 文件构建符号表
    pub fn from_file(path: &Path) -> Result<Self> {
        let code = std::fs::read_to_string(path)?;
        Self::from_code(&code)
    }

    /// 解析 Auto 代码并提取符号
    pub fn from_code(code: &str) -> Result<Self> {
        let scope = Rc::new(RefCell::new(Universe::new()));
        let mut parser = Parser::new(code, scope);
        let ast = parser.parse()?;

        let mut builder = SymbolTableBuilder {
            components: HashMap::new(),
        };

        // 提取所有 Widget 类型
        for stmt in &ast.stmts {
            if let Stmt::Type(TypeDecl { name, kind, .. }) = stmt {
                if kind.is_widget() {
                    builder.extract_widget_metadata(&name, &ast)?;
                }
            }
        }

        Ok(builder)
    }

    /// 提取 Widget 元数据
    fn extract_widget_metadata(&mut self, name: &str, ast: &Code) -> Result<()> {
        // 查找类型声明
        // 提取字段定义
        // 定位 fn view() 方法
        // 定位 fn on(ev Msg) 方法
        // 构建 WidgetMetadata

        Ok(())
    }
}
```

**关键任务**：
- [ ] 创建 `crates/auto-ui/src/interpreter/` 模块
- [ ] 集成 `auto-lang::Parser`
- [ ] 遍历 AST 查找 `type Name is Widget` 声明
- [ ] 提取字段定义（包括类型和默认值）
- [ ] 定位 `fn view()` 和 `fn on()` 方法
- [ ] 存储到线程安全的 `Arc<RwLock<SymbolTable>>`

**验证标准**：
- ✅ 能解析简单的 Counter.at 文件
- ✅ 正确提取组件字段（如 `count int = 0`）
- ✅ 正确识别 view 和 on 方法

#### 1.2 组件实例管理

**文件**: `crates/auto-ui/src/interpreter/component_instance.rs` (新建)

```rust
use auto_val::{Value, Node};
use std::collections::HashMap;

pub struct ComponentInstance {
    metadata: WidgetMetadata,
    state: HashMap<String, Value>,
    cached_view: Option<View<DynamicMessage>>,
    view_dirty: bool,
}

impl ComponentInstance {
    /// 使用元数据创建新实例
    pub fn new(metadata: WidgetMetadata) -> Self {
        let state = metadata.fields.iter()
            .map(|f| (f.name.clone(), f.default_value.clone()))
            .collect();

        Self {
            metadata,
            state,
            cached_view: None,
            view_dirty: true,
        }
    }

    /// 获取字段值
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.state.get(name)
    }

    /// 设置字段值（触发视图重建）
    pub fn set(&mut self, name: &str, value: Value) {
        if self.state.contains_key(name) {
            self.state.insert(name.to_string(), value);
            self.view_dirty = true;
        }
    }

    /// 处理事件消息
    pub fn on_event(&mut self, event: &str) -> Result<()> {
        // 执行组件的 on() 方法
        // 解析事件字符串
        // 匹配 on() 方法中的模式
        // 更新状态

        Ok(())
    }

    /// 构建或获取缓存的视图
    pub fn view(&mut self) -> Result<View<DynamicMessage>> {
        if self.view_dirty || self.cached_view.is_none() {
            self.cached_view = Some(self.build_view()?);
            self.view_dirty = false;
        }

        Ok(self.cached_view.clone().unwrap())
    }

    /// 从元数据构建视图树
    fn build_view(&self) -> Result<View<DynamicMessage>> {
        // 解释 view() 方法
        // 使用 auto-lang 解释器求值为 Node
        // 转换 Node → View<DynamicMessage>

        Ok(View::empty())
    }
}
```

**关键任务**：
- [ ] 实现状态初始化（从字段默认值）
- [ ] 实现 get/set 方法与脏标记
- [ ] 实现事件分发到 on() 方法
- [ ] 实现懒视图构建与缓存

**验证标准**：
- ✅ 能创建 Counter 组件实例
- ✅ 字段读写正确工作
- ✅ 状态变更触发视图重建

### Phase 2: 增强的 Node 转换器 (Week 2-3)

**目标**：将 `auto_val::Node` 转换为 `View<DynamicMessage>`，保留类型元数据。

**文件**: `crates/auto-ui/src/node_converter.rs` (修改)

**当前限制**：
```rust
// 当前：View<String> - 弱类型字符串消息
pub fn convert_node(node: &Node) -> ConversionResult<View<String>> {
    // ...
}
```

**增强方案**：
```rust
// 新增：带类型信息的转换
pub fn convert_node_typed(
    node: &Node,
    component_name: &str,
    symbol_table: &SymbolTable,
) -> ConversionResult<View<DynamicMessage>> {
    let kind = node.name.as_str();

    match kind {
        "button" => {
            let label = extract_main_arg_str(node)?;
            let onclick = extract_prop_str(node, "onclick")?;

            // 从符号表查找组件的消息类型
            let msg_type = symbol_table
                .get_component(component_name)
                .and_then(|c| c.get_message_type(&onclick));

            Ok(View::Button {
                label,
                onclick: DynamicMessage::Typed {
                    component: component_name.to_string(),
                    event_name: onclick,
                    args: Vec::new(),
                },
                style: extract_style(node)?,
            })
        }
        // ... 其他组件
        _ => Err(ConversionError::UnknownKind { kind: kind.to_string() })
    }
}
```

**关键任务**：
- [ ] 添加 `component_name` 参数
- [ ] 查询符号表获取事件类型信息
- [ ] 生成 `DynamicMessage::Typed` 而非纯字符串
- [ ] 保留类型元数据用于路由
- [ ] 向后兼容（仍支持 `View<String>`）

**验证标准**：
- ✅ 转换 Counter.at 的视图树
- ✅ 按钮点击生成类型化消息
- ✅ 保留组件和事件名信息

### Phase 3: 事件路由与消息分发 (Week 3)

**目标**：将 UI 事件路由到正确的组件处理函数。

**文件**: `crates/auto-ui/src/interpreter/event_router.rs` (新建)

```rust
pub struct EventRouter {
    /// 组件实例注册表
    components: HashMap<String, Arc<RwLock<ComponentInstance>>>,
}

impl EventRouter {
    /// 注册组件实例
    pub fn register(&mut self, name: String, instance: Arc<RwLock<ComponentInstance>>) {
        self.components.insert(name, instance);
    }

    /// 分发消息到组件
    pub fn dispatch(&self, msg: DynamicMessage) -> Result<()> {
        match msg {
            DynamicMessage::String(event) => {
                self.dispatch_string(&event)
            }
            DynamicMessage::Typed { component, event_name, args } => {
                self.dispatch_typed(&component, &event_name, &args)
            }
        }
    }

    /// 分发字符串事件
    fn dispatch_string(&self, event: &str) -> Result<()> {
        // 解析 "component.event" 或仅 "event"
        if let Some(dot_pos) = event.find('.') {
            let component = &event[..dot_pos];
            let event_name = &event[dot_pos + 1..];
            self.dispatch_typed(component, event_name, &[])
        } else {
            // 广播到所有组件？或使用默认组件？
            if let Some((name, _)) = self.components.iter().next() {
                self.dispatch_typed(name, event, &[])?;
            }
        }
        Ok(())
    }

    /// 分发类型化事件
    fn dispatch_typed(&self, component: &str, event: &str, args: &[Value]) -> Result<()> {
        let instance = self.components.get(component)
            .ok_or_else(|| Error::ComponentNotFound(component.to_string()))?;

        let mut instance = instance.write()
            .map_err(|e| Error::LockError(e.to_string()))?;

        instance.on_event(event)?;

        Ok(())
    }
}
```

**关键任务**：
- [ ] 实现组件注册表
- [ ] 解析字符串消息格式
- [ ] 路由类型化消息
- [ ] 处理组件生命周期
- [ ] 支持广播消息

**验证标准**：
- ✅ Counter 的 Inc/Dec 事件正确路由
- ✅ 多组件应用正确分发到各组件
- ✅ 错误处理（未找到组件等）

### Phase 4: 热重载集成 (Week 4)

**目标**：增强 `hot_reload.rs`，支持真正的热重载。

**文件**: `crates/auto-ui/src/interpreter/hot_reload.rs` (新建)

```rust
pub struct HotReloadInterpreter {
    /// 监听的文件路径
    path: PathBuf,

    /// 当前运行时状态
    runtime: Arc<RwLock<InterpreterRuntime>>,

    /// 文件监听器
    watcher: Option<RecommendedWatcher>,
}

impl HotReloadInterpreter {
    /// 加载并监听 .at 文件
    pub fn load_and_watch(path: &Path) -> Result<Self> {
        let runtime = Self::initial_load(path)?;
        let mut instance = Self {
            path: path.to_path_buf(),
            runtime: Arc::new(RwLock::new(runtime)),
            watcher: None,
        };

        instance.start_watching()?;
        Ok(instance)
    }

    /// 初始文件加载
    fn initial_load(path: &Path) -> Result<InterpreterRuntime> {
        let code = std::fs::read_to_string(path)?;
        let symbol_table = SymbolTableBuilder::from_code(&code)?;

        // 创建主组件实例
        let main_component = symbol_table.get_main_component()
            .ok_or_else(|| Error::NoMainComponent)?;

        let mut runtime = InterpreterRuntime::new(symbol_table);
        runtime.instantiate_component(&main_component.name)?;

        Ok(runtime)
    }

    /// 启动文件监听
    fn start_watching(&mut self) -> Result<()> {
        let runtime = self.runtime.clone();
        let path = self.path.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                if let EventKind::Modify(_) = event.kind {
                    if event.paths.iter().any(|p| p == &path) {
                        // 触发重载
                        if let Ok(mut rt) = runtime.write() {
                            let _ = rt.reload_component(&path);
                        }
                    }
                }
            }
        })?;

        watcher.watch(&self.path, RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);

        Ok(())
    }

    /// 获取当前视图
    pub fn view(&self) -> Result<View<DynamicMessage>> {
        let runtime = self.runtime.read()
            .map_err(|e| Error::LockError(e.to_string()))?;
        runtime.get_main_view()
    }
}
```

**关键任务**：
- [ ] 集成 `notify` crate 进行文件监听
- [ ] 文件变更时重新解析
- [ ] 更新符号表（尽可能保留）
- [ ] 标记受影响组件为脏
- [ ] **智能状态迁移**（尽可能保持状态）
- [ ] 错误恢复（解析失败时不崩溃）

**状态迁移策略**：
```rust
fn migrate_state(old_instance: &ComponentInstance, new_metadata: &WidgetMetadata) -> HashMap<String, Value> {
    let mut new_state = HashMap::new();

    // 保留同名字段
    for field in &new_metadata.fields {
        if let Some(old_value) = old_instance.get(&field.name) {
            // 类型兼容性检查
            if is_type_compatible(old_value, &field.type_) {
                new_state.insert(field.name.clone(), old_value.clone());
            }
        }
    }

    // 新字段使用默认值
    for field in &new_metadata.fields {
        if !new_state.contains_key(&field.name) {
            new_state.insert(field.name.clone(), field.default_value.clone());
        }
    }

    new_state
}
```

**验证标准**：
- ✅ 修改 .at 文件后 100ms 内看到变化
- ✅ 组件状态尽可能保留
- ✅ 解析错误显示友好的错误信息
- ✅ 不会因为语法错误崩溃应用

### Phase 5: GPUI 集成 (Week 5)

**目标**：创建 GPUI 可渲染的动态组件包装器。

**文件**: `crates/auto-ui-gpui/src/interpreter_component.rs` (新建)

```rust
use auto_ui::interpreter::{InterpreterRuntime, DynamicMessage};
use gpui::*;

pub struct DynamicInterpreterComponent {
    interpreter: Arc<RwLock<InterpreterRuntime>>,
    root_entity: Entity<()>,
}

impl DynamicInterpreterComponent {
    /// 从 .at 文件加载
    pub fn from_file(path: &Path, cx: &mut Context<Self>) -> Self {
        let interpreter = InterpreterRuntime::load_file(path)
            .expect("Failed to load interpreter");

        Self {
            interpreter: Arc::new(RwLock::new(interpreter)),
            root_entity: cx.new(|_| ()),
        }
    }
}

impl Render for DynamicInterpreterComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 从解释器获取当前视图
        let view = self.interpreter.read()
            .and_then(|rt| rt.get_main_view())
            .unwrap_or_else(|_| View::empty());

        // 转换为 GPUI 元素
        self.render_view(view, cx)
    }
}

impl DynamicInterpreterComponent {
    fn render_view(&mut self, view: View<DynamicMessage>, cx: &mut Context<Self>) -> AnyElement {
        match view {
            View::Button { label, onclick, style } => {
                let interpreter = self.interpreter.clone();

                div()
                    .child(label)
                    .on_click(cx.listener(move |_this, _event, _window, cx| {
                        // 分发事件
                        let _ = interpreter.read()
                            .and_then(|rt| rt.dispatch(onclick.clone()));
                        cx.notify();
                    }))
                    .into_any()
            }
            View::Col { spacing, children, .. } => {
                div().flex().flex_col().gap(*spacing)
                    .children(children.iter().map(|child| {
                        self.render_view(child.clone(), cx)
                    }))
                    .into_any()
            }
            // ... 其他变体
            _ => div().into_any()
        }
    }
}
```

**关键任务**：
- [ ] 实现 `Render` trait
- [ ] 转换 `View<DynamicMessage>` 为 GPUI 元素
- [ ] 连接事件处理器到解释器
- [ ] 处理热重载后的重新渲染
- [ ] 性能优化（避免不必要的重建）

**验证标准**：
- ✅ 能渲染简单的 Counter 组件
- ✅ 按钮点击触发状态更新
- ✅ 热重载后视图自动更新
- ✅ 渲染性能可接受（< 16ms/帧）

### Phase 6: 开发者工具 (Week 6)

**目标**：提供完整的开发体验工具。

#### 6.1 CLI Dev Server

**文件**: `crates/auto-ui/src/bin/auto-ui-dev.rs` (新建)

添加新的 `dev` 子命令：

```bash
auto-ui-dev counter.at
```

**功能**：
- 启动解释器监听模式
- 打开 GPUI 窗口
- 显示编译错误的覆盖层
- 支持键盘快捷键（R 重载, Q 退出）
- 实时日志输出

#### 6.2 错误覆盖层

**文件**: `crates/auto-ui/src/interpreter/error_overlay.rs` (新建)

```rust
pub struct ErrorOverlay {
    errors: Vec<ErrorDisplay>,
}

pub struct ErrorDisplay {
    file: PathBuf,
    line: usize,
    message: String,
    source_excerpt: String,
}

impl ErrorOverlay {
    pub fn render(&self) -> View<DynamicMessage> {
        col()
            .style("fixed top-4 right-4 bg-red-500 text-white p-4 rounded-lg")
            .children(self.errors.iter().map(|e|
                text(&format!("{}:{} - {}",
                    e.file.display(),
                    e.line,
                    e.message
                ))
            ))
            .build()
    }
}
```

**关键任务**：
- [ ] 实现 dev server CLI
- [ ] 创建错误覆盖层 UI
- [ ] 添加键盘快捷键
- [ ] 实现日志输出
- [ ] 添加使用文档

**验证标准**：
- ✅ `auto-ui-dev counter.at` 一键启动
- ✅ 语法错误显示友好的错误信息
- ✅ 开发体验流畅无卡顿

## 文件清单

### 新建文件 (~15 个，~3500 行)

```
crates/auto-ui/src/interpreter/
├── mod.rs                    # 模块导出 (50行)
├── runtime.rs               # InterpreterRuntime (200行)
├── symbol_table.rs          # SymbolTable (300行)
├── component_instance.rs    # ComponentInstance (250行)
├── event_router.rs          # EventRouter (200行)
├── hot_reload.rs            # 热重载集成 (300行)
├── error_overlay.rs         # 错误UI (150行)
└── state_migration.rs       # 状态迁移 (200行)

crates/auto-ui/src/
├── node_converter.rs        # 修改：增强类型支持 (+150行)

crates/auto-ui-gpui/src/
├── interpreter_component.rs # GPUI包装器 (400行)
└── interpreter_view.rs      # View转换 (300行)

crates/auto-ui/src/bin/
├── auto-ui-dev.rs           # Dev server (300行)

docs/plans/
└── 011-auto-interpreter.md   # 本文档
```

**总代码量**: ~3,500 行

## 关键技术点

### 1. 运行时类型信息保留

**挑战**：如何在动态模式下保持类型安全？

**方案**：
```rust
// 解析阶段：提取类型信息
let metadata = SymbolTableBuilder::from_code(code)?;
let msg_type = metadata.get_message_type("Counter"); // Msg enum

// 转换阶段：生成类型化消息
View::Button {
    onclick: DynamicMessage::Typed {
        component: "Counter".to_string(),
        event_name: "Inc".to_string(),
        args: vec![],
    }
}

// 运行时：分发到正确的处理函数
event_router.dispatch(msg)?;
```

### 2. 状态迁移策略

**挑战**：热重载时如何保持组件状态？

**方案**：
```rust
// 智能字段迁移
fn migrate_state(old: &ComponentInstance, new: &WidgetMetadata) -> State {
    let mut new_state = State::new();

    // 1. 保留兼容的字段
    for (name, old_value) in old.state.iter() {
        if let Some(new_field) = new.get_field(name) {
            if is_type_compatible(old_value, new_field.type_) {
                new_state.insert(name, old_value);
            }
        }
    }

    // 2. 新字段使用默认值
    for field in new.fields {
        if !new_state.contains(field.name) {
            new_state.insert(field.name, field.default);
        }
    }

    new_state
}
```

### 3. 性能优化

**挑战**：解释执行性能如何保证？

**方案**：
- **视图缓存**：只在状态变化时重建
- **脏标记**：精确跟踪需要重建的组件
- **增量解析**：只重新解析变更的文件
- **惰性求值**：按需求值视图方法
- **AST 缓存**：缓存解析结果避免重复解析

### 4. 错误处理

**挑战**：语法错误如何不崩溃应用？

**方案**：
```rust
pub fn reload_with_recovery(&mut self, path: &Path) {
    match Self::parse_file(path) {
        Ok(new_runtime) => {
            *self.runtime = new_runtime;
            self.error = None;
        }
        Err(e) => {
            // 保留旧运行时，只更新错误信息
            self.error = Some(e);
            // 在 UI 中显示错误覆盖层
        }
    }
}
```

## 成功标准

### Must Have
- ✅ 修改 .at 文件后 100ms 内看到变化（下一帧）
- ✅ 组件状态在热重载时尽可能保持
- ✅ 支持 Counter、TodoList 等基础组件
- ✅ 错误处理不崩溃应用
- ✅ 单命令启动开发服务器

### Nice to Have
- ✅ 类型化事件路由（80% 消息）
- ✅ 错误覆盖层 UI
- ✅ 性能开销 < 50ms/帧
- ✅ 支持多文件组件（import/use）
- ✅ 调试模式（断点、变量检查）

### Future Work
- 增量编译（缓存 AST）
- 多后端切换（GPUI/Iced 运行时）
- 类型推断增强
- 自动测试生成

## 与现有系统集成

### 复用现有代码

1. **hot_reload.rs** - 文件监听框架（需增强）
2. **node_converter.rs** - Node → View 转换（需增强）
3. **auto-render.rs** - GPUI 渲染模式
4. **auto-lang Parser** - 完整解析器集成

### 新建独立模块

1. **interpreter/** - 动态解释器核心
2. **auto-ui-dev** - 开发服务器 CLI

### 向后兼容

- 静态转译模式继续工作
- 独立示例不受影响
- 可选启用解释器模式

## Timeline

- **Phase 1** (符号表): Week 1-2
- **Phase 2** (Node 转换): Week 2-3
- **Phase 3** (事件路由): Week 3
- **Phase 4** (热重载): Week 4
- **Phase 5** (GPUI 集成): Week 5
- **Phase 6** (开发工具): Week 6

**总计**: 4-6 周

## 风险与缓解

### 风险 1：性能问题

**风险**：解释执行可能太慢

**缓解**：
- 视图缓存和脏标记
- 性能基准测试
- 必要时 JIT 编译热点路径

### 风险 2：类型安全丢失

**风险**：动态模式可能过于弱类型

**缓解**：
- 保留类型元数据
- 类型化消息路由
- 可选的静态类型检查

### 风险 3：状态迁移复杂

**风险**：热重载时状态难以保持

**缓解**：
- 简单的按名字段迁移
- 类型兼容性检查
- 提供重置选项

## 使用示例

### 开发期使用解释器

```bash
# 启动开发服务器
auto-ui-dev examples/counter.at

# 修改 counter.at 文件
# 编辑 count 字段的默认值
# 保存文件

# 下一帧立即看到变化 ✨
```

### 生产期使用静态转译

```bash
# 生成 Rust 代码
auto-ui-transpile examples/counter.at

# 编译运行
cargo run --release
```

## 技术挑战与解决方案

### GPUI Entity 系统限制 (2025-01-27)

**问题描述**：

在尝试将 `DynamicInterpreterComponent` 集成到 GPUI 应用时遇到了 Entity 生命周期限制：

1. **初始化时机问题**：
   ```rust
   // ❌ 不工作：DynamicInterpreterComponent::from_file 需要 Context
   struct SimpleDemoApp {
       interpreter: DynamicInterpreterComponent,  // 需要在 new() 中创建
   }

   impl SimpleDemoApp {
       fn new(cx: &mut Context<Self>) -> Self {
           // 问题：from_file 需要 &mut Window 和 &mut Context<Self>
           // 但 DynamicInterpreterComponent 又需要在自己创建时传递 Context
           let interpreter = DynamicInterpreterComponent::from_file(path, window, cx);
           // 类型不匹配：期望 &mut Context<DynamicInterpreterComponent>
           // 实际得到：&mut Context<SimpleDemoApp>
       }
   }
   ```

2. **Context 类型不匹配**：
   - GPUI 的 Entity 系统要求每个组件有唯一的 `Context<T>`
   - `DynamicInterpreterComponent::from_file` 需要 `&mut Context<DynamicInterpreterComponent>`
   - 但在 `SimpleDemoApp::new()` 中只能访问 `&mut Context<SimpleDemoApp>`

3. **嵌套 Entity 问题**：
   - GPUI 不支持在一个 Entity 的创建过程中创建另一个 Entity
   - `cx.new()` 只能在顶级调用，不能嵌套

**尝试的解决方案**：

1. **❌ Option 包装**：
   ```rust
   struct SimpleDemoApp {
       interpreter: Option<DynamicInterpreterComponent>,
   }
   ```
   - 问题：仍然需要在某个地方创建组件，同样遇到 Context 类型不匹配

2. **❌ 延迟初始化**：
   ```rust
   fn render(&mut self, cx: &mut Context<Self>) {
       if self.interpreter.is_none() {
           self.interpreter = Some(DynamicInterpreterComponent::from_file(...));
       }
   }
   ```
   - 问题：render 中同样无法访问 `&mut Window` 和正确的 Context 类型

**当前解决方案**：

创建了 **简化演示版本** (`interpreter-gpui-minimal`)：
- 使用静态 UI 展示架构和设计意图
- 暂时不嵌入实际的 `DynamicInterpreterComponent`
- 清晰标注"演示模式"和已知限制

**未来解决方案方向**：

1. **方案 1：全局状态管理器**
   ```rust
   // 使用全局 Arc<RwLock<>> 绕过 Entity 限制
   static INTERPRETER_STATE: Lazy<Arc<RwLock<InterpreterState>>> = ...;

   struct SimpleDemoApp {
       interpreter_id: Uuid,  // 仅存储 ID
   }
   ```

2. **方案 2：延迟初始化 + 消息传递**
   ```rust
   enum AppMessage {
       InitializeInterpreter(PathBuf),
   }

   fn on(&mut self, msg: AppMessage, cx: &mut Context<Self>) {
       match msg {
           AppMessage::InitializeInterpreter(path) => {
               // 通过特殊通道初始化
           }
       }
   }
   ```

3. **方案 3：独立解释器进程**
   - 将解释器运行在独立线程/进程
   - 通过消息传递与 GPUI 通信
   - 完全解耦 Entity 生命周期

**经验教训**：

- ✅ GPUI 的 Entity 系统与传统组件模型有显著差异
- ✅ 在设计新架构时需要充分考虑 GPUI 的生命周期限制
- ✅ 创建简化原型有助于快速发现架构问题
- ✅ 文档化 API 兼容性问题对后续开发至关重要

### GPUI 0.2.2 API 变更记录 (2025-01-27)

**应用程序启动**：

```rust
// ❌ 旧方式（不工作）
App::new().run(|cx: &mut AppContext| {
    cx.open_window(..., |cx| {
        cx.new_view(|cx| App::new(cx))
    })
})

// ✅ 正确方式
Application::new().run(|cx: &mut App| {
    cx.open_window(options, |_window, cx| {
        cx.new(|_| App::new_empty())  // 使用 new() 而非 new_view()
    })
})
```

**Context 类型**：

```rust
// ❌ ViewContext 不存在
fn render(&mut self, _window: &mut Window, cx: &mut ViewContext<Self>)

// ✅ 使用 Context
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>)
```

**AssetSource trait**：

```rust
// ❌ 错误的错误类型
fn load(&self, path: &str) -> Result<..., Box<dyn std::error::Error>>

// ✅ 正确：使用 anyhow::Error
fn load(&self, path: &str) -> anyhow::Result<...>
```

## Notes

- 这是一个**增量式**实现计划，可以逐步添加功能
- 优先实现**核心功能**，渐进增强
- 保持**向后兼容**，不影响现有代码
- 注重**开发者体验**，这是主要价值

---

**Document Status**: 🔄 In Progress - Phase 5 架构重新设计
**Last Updated**: 2025-01-27
**Author**: Claude Sonnet 4.5
**Review Status**: Pending

---

## 📝 今日总结 (2025-01-27)

### 完成工作
- ✅ 创建 `interpreter-gpui-minimal` 简化演示示例
- ✅ 修复 GPUI 0.2.2 API 兼容性问题
- ✅ 添加 auto-lang 缺失的 TokenKind 类型
- ✅ 更新 Plan 011 文档，记录技术挑战

### 发现的关键问题
- ⚠️ GPUI Entity 系统与动态解释器架构存在生命周期冲突
- ⚠️ 需要重新设计 Phase 5 的集成方案

### 下一步行动
1. 评估三种可能的解决方案（全局状态、消息传递、独立进程）
2. 选择最优方案并实现原型
3. 完成实际的动态渲染集成

### 文件清单
- [examples/interpreter-gpui-minimal/src/main.rs](examples/interpreter-gpui-minimal/src/main.rs) - 简化演示（编译通过）
- [examples/interpreter-gpui-minimal/simple.at](examples/interpreter-gpui-minimal/simple.at) - 测试用 Auto 代码
- [docs/plans/011-auto-interpreter.md](docs/plans/011-auto-interpreter.md) - 更新的计划文档
