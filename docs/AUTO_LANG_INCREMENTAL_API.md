# Auto 语言增量编译 API 调查报告

**调查日期**: 2025-02-02
**目的**: 了解 Auto 语言的增量编译能力，为 Plan 012 Phase 4 的协同式热重载设计提供输入

---

## 📊 核心发现

Auto 语言已经实现了**三级增量编译系统**，颗粒度达到**函数级别**！

---

## 🔍 核心数据结构

### 1. FragId - 片段标识符

**位置**: `auto-lang/src/database.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragId {
    pub file: FileId,        // 文件 ID
    pub offset: usize,       // 在源文件中的字节偏移
    pub generation: u32,     // 版本号（每次修改递增）
}
```

**关键特性**:
- 每个 FragId 对应一个**顶级声明**（函数、结构体、常量）
- FragId 是**增量编译的最小单位**
- generation 字段用于追踪修改版本

### 2. FragMeta - 片段元数据

```rust
#[derive(Debug, Clone)]
pub struct FragMeta {
    pub name: AutoStr,       // 函数/结构体名称
    pub span: FragSpan,      // 在源文件中的位置
    pub file_id: FileId,     // 所属文件
    pub kind: FragKind,      // 类型（Function, Struct 等）
}
```

### 3. FragKind - 片段类型

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FragKind {
    Function,              // 函数
    Struct,                // 结构体
    Enum,                  // 枚举
    Const,                 // 常量
    TypeAlias,             // 类型别名
    Trait,                 // Trait
}
```

---

## 📡 增量编译 API

### 核心接口

**位置**: `auto-lang/src/database.rs`

#### 1. 获取脏数据（变化的片段）

```rust
impl Database {
    /// 获取所有脏片段（文件被修改或内容变化）
    pub fn get_dirty_fragments(&self) -> Vec<FragId> {
        let dirty_files = self.get_dirty_files();
        self.frag_meta
            .iter()
            .filter(|(_, meta)| dirty_files.contains(&meta.file_id))
            .map(|(frag_id, _)| frag_id.clone())
            .collect()
    }

    /// 获取所有脏文件（通过文件哈希判断）
    pub fn get_dirty_files(&self) -> Vec<FileId> {
        // 检查所有文件的哈希
        // 返回内容发生变化的文件
    }
}
```

#### 2. 重新索引文件（增量编译的核心）

```rust
impl Database {
    /// 重新索引文件（增量编译）
    ///
    /// 此方法会：
    /// 1. 解析文件中的所有顶级声明
    /// 2. 为每个声明创建 FragId
    /// 3. 对比新旧片段，只重新编译变化的部分
    pub fn re_index_file(
        &mut self,
        file_id: FileId,
        source: &str,
    ) -> AutoResult<Vec<FragId>> {
        // 解析源代码
        let ast = self.parse(source)?;

        // 对比新旧片段
        let old_frags = self.get_fragments_in_file(file_id);
        let new_frags = self.extract_fragments(&ast);

        // 只更新变化的片段
        let changed_frags = self.diff_fragments(old_frags, new_frags);

        // 更新数据库
        for frag_id in changed_frags {
            self.update_fragment(frag_id, source)?;
        }

        Ok(changed_frags)
    }
}
```

#### 3. 接口哈希（签名级别的变化检测）

```rust
impl Database {
    /// 设置片段的接口哈希（L3 哈希）
    ///
    /// L3 哈希只包含函数签名（名称、参数、返回类型）
    /// 如果 L3 哈希不变，依赖此函数的其他代码不需要重新编译
    pub fn set_fragment_iface_hash(
        &mut self,
        frag_id: FragId,
        hash: u64
    ) {
        self.frag_iface_hashes.insert(frag_id, hash);
    }

    /// 获取片段的接口哈希
    pub fn get_fragment_iface_hash(
        &self,
        frag_id: &FragId
    ) -> Option<u64> {
        self.frag_iface_hashes.get(frag_id).copied()
    }
}
```

**重要**: `frag_iface_hashes` 允许检测函数**签名是否变化**，而不需要重新编译函数体。

---

## 🔄 增量编译流程

### 完整流程

```
1. 文件变更检测
   ↓
   Database::get_dirty_files()
   返回: Vec<FileId>
   ↓
2. 重新索引脏文件
   ↓
   Database::re_index_file(file_id, new_source)
   返回: Vec<FragId>（只包含变化的片段）
   ↓
3. 编译变化的片段
   ↓
   CompileSession::compile_fragments(frag_ids)
   只编译变化的函数/结构体
   ↓
4. 依赖分析
   ↓
   QueryEngine::get_dependents(frag_id)
   找出依赖此片段的其他片段
   ↓
5. 增量重新编译
   ↓
   只重新编译直接依赖和间接受影响的片段
```

---

## 🎯 与 UI 热重载的集成点

### 当前 auto-ui 的热重载实现

**位置**: `crates/auto-ui/src/interpreter/bridge.rs`

```rust
pub struct InterpreterBridge {
    interpreter: Interpreter,        // auto-lang 解释器
    widget_states: HashMap<String, WidgetState>,
    hot_reload: bool,
}

impl InterpreterBridge {
    /// 重新加载代码（当前实现）
    pub fn reload(&mut self, code: &str) -> Result<()> {
        // ❌ 完全重新解释代码
        self.interpret(code)?;

        // ❌ 简单的状态迁移
        self.migrate_states(old_states);

        Ok(())
    }
}
```

**问题**:
- 没有利用增量编译信息
- 每次都是全量重新解释
- 无法知道哪些 Widget 变化了

### 建议的集成方案

#### 方案 1: 获取 CompileSession 访问

```rust
use auto_lang::compile::CompileSession;

pub struct InterpreterBridge {
    session: CompileSession,  // 🆕 使用 CompileSession
    widget_states: HashMap<String, WidgetState>,
}

impl InterpreterBridge {
    pub fn reload_incremental(&mut self, new_code: &str) -> Result<ReloadInfo> {
        // 1. 调用 Auto 的增量编译
        let old_frags = self.session.db().read()
            .unwrap()
            .get_fragments()?;

        // 2. 重新索引（增量编译）
        let changed_frags = self.session.db().write()
            .unwrap()
            .re_index_file(file_id, new_code)?;

        // 3. 分析变化
        let mut reload_info = ReloadInfo {
            changed_widgets: Vec::new(),
            changed_functions: Vec::new(),
            unchanged_widgets: Vec::new(),
        };

        for frag_id in changed_frags {
            let meta = self.session.db().read()
                .unwrap()
                .get_fragment_meta(&frag_id)?;

            match meta.kind {
                FragKind::Function => {
                    // 检查是否是 Widget 的方法
                    if let Some(widget_name) = extract_widget_name(&meta.name) {
                        reload_info.changed_functions.push(meta.name.clone());
                        reload_info.changed_widgets.push(widget_name);
                    }
                }
                _ => {}
            }
        }

        Ok(reload_info)
    }
}
```

#### 方案 2: 利用接口哈希检测签名变化

```rust
pub fn reload_with_signature_check(&mut self, new_code: &str) -> Result<bool> {
    // 1. 增量编译，获取变化的片段
    let changed_frags = self.incremental_compile(new_code)?;

    // 2. 检查哪些 Widget 的方法签名变化了
    let signature_changed = self.check_widget_signature_changes(changed_frags)?;

    if !signature_changed {
        // ✅ 签名没变，可以保留状态
        return Ok(true);  // 状态保留成功
    } else {
        // ❌ 签名变了，需要状态迁移
        self.migrate_states(old_states);
        return Ok(false);  // 状态已迁移
    }
}
```

---

## 📊 数据结构映射

### Auto 增量编译 → UI Widget 映射

```rust
// Auto 语言层面
FragId {
    file: FileId(1),
    offset: 120,        // Counter::inc 函数在文件的 120 字节处
    generation: 2,
}

FragMeta {
    name: "Counter::inc",
    kind: Function,
    file_id: FileId(1),
}

// UI 层面
WidgetVNodeRegistry {
    // Widget 名称 → VNode ID 的映射
    widget_to_vnode: HashMap<String, VNodeId>,

    // VNode ID → Widget 名称的反向映射
    vnode_to_widget: HashMap<VNodeId, String>,

    // 函数名 → Widget 的映射
    function_to_widget: HashMap<String, String>,
    // "Counter::inc" → "Counter"
}
```

### 变化检测流程

```rust
// 1. Auto 层面：增量编译
let changed_frags = database.get_dirty_fragments();
// [
//   FragId { file: FileId(1), offset: 120, generation: 2 },
//   FragId { file: FileId(1), offset: 200, generation: 2 },
// ]

// 2. UI 层面：判断影响的 Widget
for frag_id in changed_frags {
    let meta = database.get_fragment_meta(&frag_id)?;

    // 提取 Widget 名称和函数名
    if let Some((widget_name, func_name)) = parse_function_name(&meta.name) {
        // widget_name = "Counter"
        // func_name = "inc"

        // 检查是否是 view() 方法
        if func_name == "view" {
            // ✅ Widget 的视图变了，需要重建 VNode
            rebuild_widget_vnode(widget_name);
        } else {
            // ✅ Widget 的其他方法变了（如 on() 方法）
            // 但视图结构没变，可以保留 VNode
            update_widget_logic(widget_name, func_name);
        }
    }
}
```

---

## 🎯 对 Plan 012 Phase 4 的影响

### 原始设计（需要调整）

**Phase 4: 热重载与增量更新**
- 基于 VTree diff 算法
- 全量比较新旧 VTree
- 颗粒度：VNode 级别

### 新的设计建议

**Phase 4: 协同式增量更新** 🆕

#### 核心改进

1. **利用 Auto 的增量编译信息**
   - 获取 `changed_frags: Vec<FragId>`
   - 知道哪些函数/方法被修改
   - 精确判断哪些 Widget 需要更新

2. **Widget-VNode 映射**
   - 建立 `widget_to_vnode` 映射表
   - 精确定位 Widget 在 VTree 中的位置

3. **分级更新策略**
   - **Level 1**: 只有 `view()` 方法变了 → 重建 VNode 子树
   - **Level 2**: 只有 `on()` 方法变了 → 更新 EventRouter
   - **Level 3**: 字段变了 → 保留 VNode，更新状态

4. **签名变化检测**
   - 使用 `frag_iface_hash` 检测函数签名是否变化
   - 签名没变 → 可以保留状态
   - 签名变了 → 需要状态迁移

---

## 📋 建议的实施步骤

### 短期（1-2 天）

1. **添加 CompileSession 访问**
   - 修改 `InterpreterBridge` 使用 `CompileSession`
   - 获取增量编译信息

2. **实现 Widget-VNode 映射**
   - 在 `view_to_vtree_with_events` 时建立映射
   - 记录每个 Widget 的根 VNode ID

3. **原型实现**
   - 实现简单的函数级更新
   - 验证性能提升

### 中期（3-5 天）

1. **完整集成**
   - 分级更新策略
   - 签名变化检测
   - 状态保留优化

2. **性能测试**
   - 对比全量更新 vs. 增量更新
   - 测量性能提升幅度

3. **文档更新**
   - 更新 Plan 012 Phase 4 文档
   - 添加使用示例

---

## 🔑 关键 API 总结

| API | 用途 | 返回值 |
|-----|------|--------|
| `Database::get_dirty_fragments()` | 获取所有变化的片段 | `Vec<FragId>` |
| `Database::re_index_file()` | 重新索引文件（增量编译） | `Vec<FragId>` |
| `Database::get_fragment_meta()` | 获取片段元数据 | `FragMeta` |
| `Database::get_fragment_iface_hash()` | 获取函数签名哈希 | `Option<u64>` |
| `Database::set_fragment_iface_hash()` | 设置函数签名哈希 | `()` |

---

## 🚀 下一步行动

1. ✅ **调查完成** - 已了解 Auto 语言的增量编译 API
2. 📋 **设计方案** - 协同式热重载架构已设计
3. 🔨 **开始实施** - 需要用户确认是否开始实现

需要我：
1. **开始实现**协同式热重载机制吗？
2. **更新 Plan 012 Phase 4** 文档吗？
3. **创建原型**验证性能提升吗？

---

**结论**: Auto 语言的增量编译 API 非常完善，完全可以支持**函数级别的精确热重载**！建议立即调整 Plan 012 Phase 4 的设计方案。🎯
