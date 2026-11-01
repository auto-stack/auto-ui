//! PLAN-014 T-04：表达式责任（条件转换、expr_to_* 转译、TS 类型/转义）
//! （自 ui_gen/vue.rs impl 块原样收编；单 state、行为零改）。
use super::*;

impl VueGenerator {
    /// Convert AuraExpr to Auto source code string
    pub(super) fn expr_to_auto_string(&self, expr: &auto_lang_core::ast::Expr) -> String {
        use auto_lang_core::ast::Expr;
        use auto_val::Op;
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::I64(n) => n.to_string(),
            Expr::Float(n, _) | Expr::Double(n, _) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Str(s) | Expr::CStr(s) => format!("\"{}\"", s),
            Expr::Ident(name) => format!(".{}", name),
            Expr::Bina(left, op, right) => {
                let op_str = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Mod => "%",
                    Op::Eq => "==",
                    Op::Neq => "!=",
                    Op::Lt => "<",
                    Op::Le => "<=",
                    Op::Gt => ">",
                    Op::Ge => ">=",
                    Op::And => "&&",
                    Op::Or => "||",
                    _ => "+",
                };
                format!(
                    "{} {} {}",
                    self.expr_to_auto_string(left),
                    op_str,
                    self.expr_to_auto_string(right)
                )
            }
            Expr::Unary(op, operand) => {
                let op_str = match op {
                    Op::Sub => "-",
                    _ => "!",
                };
                format!("{}{}", op_str, self.expr_to_auto_string(operand))
            }
            Expr::Call(call) => {
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    let args_str: Vec<String> = call
                        .args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e)
                            | auto_lang_core::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_auto_string(&a))
                        .collect();
                    // plan446 批二 A1 配套:限定 store 调用 `Store.Method()` 发
                    // facade `store.Method()`(v1 单 store per file;多 store
                    // 命名 facade 留上游项)。不加此臂则 TS2304(裸 store 名)。
                    let receiver = self.expr_to_auto_string(object);
                    let receiver = match object.as_ref() {
                        Expr::Ident(root)
                            if self.store_deps.len() == 1 && &self.store_deps[0] == root =>
                        {
                            "store".to_string()
                        }
                        _ => receiver,
                    };
                    format!("{}.{}({})", receiver, method, args_str.join(", "))
                } else {
                    let args_str: Vec<String> = call
                        .args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e)
                            | auto_lang_core::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_auto_string(&a))
                        .collect();
                    format!(
                        "{}({})",
                        self.expr_to_auto_string(&call.name),
                        args_str.join(", ")
                    )
                }
            }
            Expr::Array(elements) => {
                let elements_str: Vec<String> = elements
                    .iter()
                    .map(|e| self.expr_to_auto_string(e))
                    .collect();
                format!("[{}]", elements_str.join(", "))
            }
            Expr::Object(pairs) => {
                let pairs: Vec<String> = pairs
                    .iter()
                    .map(|p| {
                        format!(
                            "{}: {}",
                            p.key.to_astr(),
                            self.expr_to_auto_string(&p.value)
                        )
                    })
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Expr::Closure(closure) => {
                let params_str: Vec<String> =
                    closure.params.iter().map(|p| p.name.to_string()).collect();
                format!(
                    "|{}| {}",
                    params_str.join(", "),
                    self.expr_to_auto_string(&closure.body)
                )
            }
            Expr::Dot(object, field) => {
                format!("{}.{}", self.expr_to_auto_string(object), field)
            }
            Expr::NavCall { path, params } => {
                let params_str: Vec<String> = params
                    .iter()
                    .map(|p| {
                        format!(
                            "{}: {}",
                            p.key.to_astr(),
                            self.expr_to_auto_string(&p.value)
                        )
                    })
                    .collect();
                format!(
                    "Nav.to(\"{}\", {{ {} }})",
                    self.expr_to_auto_string(path),
                    params_str.join(", ")
                )
            }
            Expr::Index(target, index) => {
                format!(
                    "{}[{}]",
                    self.expr_to_auto_string(target),
                    self.expr_to_auto_string(index)
                )
            }
            _ => "/* unsupported expr */".to_string(),
        }
    }

    /// convert_condition 的 find 守卫：逐处扫描 `.find (` / `.find(`，
    /// 参数段（到下一个 `)` 为止）含 `=>` 时是数组 find 谓词语义，
    /// 保持原样；否则映射为字符串语义的 `.indexOf(`。
    pub(super) fn map_find_in_condition(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        loop {
            let spaced = rest.find(".find (");
            let tight = rest.find(".find(");
            // `(` 的字节位置：".find (" 形态在 pos+6，".find(" 形态在 pos+5。
            let (pos, open) = match (spaced, tight) {
                (Some(a), Some(b)) if b < a => (b, b + 5),
                (Some(a), _) => (a, a + 6),
                (None, Some(b)) => (b, b + 5),
                (None, None) => break,
            };
            let args_end = rest[open..].find(')').map(|i| open + i);
            let is_lambda = args_end.map_or(false, |e| rest[open..e].contains("=>"));
            out.push_str(&rest[..pos]);
            if is_lambda {
                // 数组 find 语义：保持原样（含原空格形态）
                out.push_str(&rest[pos..=open]);
            } else {
                out.push_str(".indexOf(");
            }
            rest = &rest[open + 1..];
        }
        out.push_str(rest);
        out
    }

    /// Convert AURA condition to Vue expression
    /// Plan 051 C7: timer `when` 条件 → script 语境 JS 表达式。script 里
    /// 状态是 ref（需 `.value`）：`.field` → `field.value`；`.a.b` →
    /// `a.value.b`；非点前缀原样返回（调用侧约定简单布尔式）。
    pub(super) fn timer_condition_to_js(condition: &str) -> String {
        let c = condition.trim();
        if !c.starts_with('.') {
            return c.to_string();
        }
        let segs: Vec<&str> = c
            .trim_start_matches('.')
            .split('.')
            .filter(|s| !s.is_empty())
            .collect();
        match segs.split_first() {
            Some((first, rest)) if !rest.is_empty() => {
                format!("{}.value.{}", first, rest.join("."))
            }
            Some((first, _)) => format!("{}.value", first),
            None => "false".to_string(),
        }
    }

    pub(super) fn convert_condition(&mut self, condition: &str) -> String {
        // Convert .var to var, .len to .length, etc.
        let mut result = condition.trim().to_string();

        // Replace .len() with .length (JavaScript property, not method)
        // Plan 028 T9 fix: placeholder .length first so the bare `.len`
        // substring replace cannot re-hit an existing `.length`
        // (`.length` became `.lengthgth`, WorkspaceSelector pre-existing bug).
        result = result.replace(".length", "\u{0}");
        result = result.replace(".len()", "\u{0}");
        result = result.replace(".len", "\u{0}");
        result = result.replace("\u{0}", ".length");
        // Handle spaced-out .len ( ) from parse_condition_expr
        result = result.replace(" .len ( )", ".length");
        result = result.replace(" len ( )", ".length");
        // Plan 345 (gap N1): .contains → .includes (JS uses .includes, not .contains)
        result = result.replace(".contains(", ".includes(");
        // Plan 053 P5-3: view if 条件里的字符串方法同样走 method map
        // （与 bound/text 表达式路径共享同一张表，libs/string.rs 语义）。
        // parse_condition_expr 输出空格分隔 token（`s.starts_with ( '7' )`），
        // 所以 spaced 与紧凑两种形式都替换。无参方法先替换为 JS 名 + 空参。
        for (auto_name, js_name) in [
            ("starts_with", "startsWith"),
            ("ends_with", "endsWith"),
            ("char_at", "charAt"),
            ("substr", "substring"),
            ("sub", "substring"),
            ("slice", "substring"),
            ("replace", "replaceAll"),
            ("repeat", "repeat"),
        ] {
            result = result.replace(&format!(".{auto_name} ("), &format!(".{js_name}("));
            result = result.replace(&format!(".{auto_name}("), &format!(".{js_name}("));
        }
        // find 双语义守卫：`.find(λ)` / `.find ( λ )`（参数段含 `=>`）是
        // 数组 find 谓词语义（JS 同名），保持原样；仅字符串语义才映射
        // str::find → indexOf。逐处判断，不能一把 str::replace。
        result = Self::map_find_in_condition(&result);
        for (auto_name, js_expr) in [
            ("to_lower", "toLowerCase"),
            ("to_upper", "toUpperCase"),
            ("trim_left", "trimStart"),
            ("trim_right", "trimEnd"),
        ] {
            result = result.replace(&format!(".{auto_name} ( )"), &format!(".{js_expr}()"));
            result = result.replace(&format!(".{auto_name}()"), &format!(".{js_expr}()"));
        }
        result = result.replace(".is_empty ( )", ".length === 0");
        result = result.replace(".is_empty()", ".length === 0");
        // Plan 043 M5: Auto's None/nil literal → JS null in view conditions.
        // parse_condition_expr emits tokens space-separated, so `None`/`nil`
        // appear as standalone words. Replace them with `null`.
        let mut out = String::with_capacity(result.len());
        for tok in result.split(' ') {
            if !out.is_empty() {
                out.push(' ');
            }
            match tok {
                "None" | "nil" => out.push_str("null"),
                _ => out.push_str(tok),
            }
        }
        result = out;

        // Remove leading dot from state references (.count -> count)
        // Pattern: .identifier (at word boundary)
        let mut converted = String::new();
        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '.' && (i == 0 || !chars[i - 1].is_alphanumeric()) {
                // Check if this is a number (like 0.5)
                if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                    converted.push('.');
                    i += 1;
                    continue;
                }
                // Skip the dot (remove state prefix)
                i += 1;
                continue;
            }
            converted.push(chars[i]);
            i += 1;
        }

        // Replace double quotes with single quotes for Vue template compatibility
        // (v-if="currentPage == 'button'" not v-if="currentPage == "button"")
        let mut final_result = String::new();
        let mut in_string = false;
        for c in converted.chars() {
            if c == '"' {
                if in_string {
                    final_result.push('\''); // End of string, use single quote
                    in_string = false;
                } else {
                    final_result.push('\''); // Start of string, use single quote
                    in_string = true;
                }
            } else {
                final_result.push(c);
            }
        }

        // Handle leftover "length ( )" from parse_condition_expr + dot removal
        final_result = final_result.replace("length ( )", "length");
        final_result = final_result.replace("length ()", "length");

        // Plan 043: numeric tuple index in conditions, e.g. `field.1.Text` →
        // `field[1].Text`. parse_condition_expr renders `field.1.Text` as a
        // space-tokenized string; convert `.digit` (preceded by an identifier
        // char, not a space) to `[digit]` for valid TypeScript tuple access.
        // (Plain `.5` number literals are preserved — they follow a digit, not ident.)
        if let Ok(re) = regex::Regex::new(r"([A-Za-z_$\]\)])\.(\d+)") {
            final_result = re.replace_all(&final_result, "$1[$2]").to_string();
        }

        final_result
    }

}

impl VueGenerator {
    /// Plan 100: Infer TypeScript type from AuraExpr
    pub(super) fn expr_to_ts_type(&self, expr: &auto_lang_core::ast::Expr) -> String {
        use auto_lang_core::ast::Expr;
        match expr {
            Expr::Int(_)
            | Expr::I64(_)
            | Expr::Uint(_)
            | Expr::U64(_)
            | Expr::I8(_)
            | Expr::U8(_)
            | Expr::Byte(_)
            | Expr::Float(_, _)
            | Expr::Double(_, _) => "number".to_string(),
            Expr::Bool(_) => "boolean".to_string(),
            Expr::Str(_) | Expr::CStr(_) => "string".to_string(),
            // Plan 448 H1: a block-bodied computed's type is its trailing
            // expression's type (the value `transpile_body_as_return`
            // returns) — previously every block computed fell to `<any>`.
            Expr::Block(body) => match body.stmts.last() {
                Some(auto_lang_core::ast::Stmt::Expr(tail)) => self.expr_to_ts_type(tail),
                _ => "any".to_string(),
            },
            Expr::Ident(name) => {
                let resolved = if name.starts_with('.') {
                    &name[1..]
                } else {
                    name.as_str()
                };
                // Prefer declared prop/state types over name heuristics.
                if let Some(ty) = self
                    .prop_types
                    .get(resolved)
                    .or_else(|| self.state_types.get(resolved))
                {
                    return ty.clone();
                }
                // Try to infer type from state variable name
                if name.starts_with("is_") || name.starts_with("has_") {
                    "boolean".to_string()
                } else {
                    "number".to_string() // Default to number for state refs
                }
            }
            // `.field` / `self.field` — resolve like the bare field name.
            Expr::Dot(object, field) => {
                if matches!(object.as_ref(), Expr::Ident(n) if n.as_str() == "self" || n.as_str() == ".")
                {
                    return self.expr_to_ts_type(&Expr::Ident(field.clone()));
                }
                "any".to_string()
            }
            Expr::Bina(left, op, right) => {
                use auto_val::Op;
                match op {
                    // Comparisons always yield booleans.
                    Op::Eq | Op::Neq | Op::Lt | Op::Le | Op::Gt | Op::Ge => "boolean".to_string(),
                    // JS `||`/`&&` yield one of the OPERANDS, not a boolean
                    // (probe 14: `.a || "b"` was mis-typed `computed<boolean>`,
                    // forcing the strOr/orNull extension helpers). Same-type
                    // operands infer that type; mixed operands stay `any`.
                    Op::And | Op::Or => {
                        let lt = self.expr_to_ts_type(left);
                        let rt = self.expr_to_ts_type(right);
                        if lt == rt {
                            lt
                        } else {
                            "any".to_string()
                        }
                    }
                    // `+` is string concatenation when either side is a string
                    // (JS semantics) — don't blindly infer number.
                    Op::Add => {
                        let lt = self.expr_to_ts_type(left);
                        let rt = self.expr_to_ts_type(right);
                        if lt == "string" || rt == "string" {
                            "string".to_string()
                        } else if lt == "number" && rt == "number" {
                            "number".to_string()
                        } else {
                            "any".to_string()
                        }
                    }
                    Op::Sub | Op::Mul | Op::Div | Op::Mod => "number".to_string(),
                    _ => "any".to_string(),
                }
            }
            Expr::Unary(op, _) => {
                if matches!(op, auto_val::Op::Not) {
                    "boolean".to_string()
                } else {
                    "number".to_string()
                }
            }
            Expr::Array(_) => "any[]".to_string(),
            // Plan 028 F1: uniform dict literals infer `Record<K, V>` —
            // mixed-value dicts stay `any` (no union inference yet).
            Expr::Object(pairs) => {
                let all_str_keys = pairs
                    .iter()
                    .all(|p| matches!(p.key, auto_lang_core::ast::Key::StrKey(_)));
                if all_str_keys && !pairs.is_empty() {
                    let value_types: Vec<String> = pairs
                        .iter()
                        .map(|p| self.expr_to_ts_type(&p.value))
                        .collect();
                    let first = &value_types[0];
                    if value_types.iter().all(|t| t == first) && first != "any" {
                        return format!("Record<string, {}>", first);
                    }
                }
                "any".to_string()
            }
            _ => "any".to_string(), // Default fallback
        }
    }

    /// Check if LogicPayload contains NavCall (Plan 105)
    pub(super) fn payload_has_nav_call(payload: &LogicPayload) -> bool {
        match payload {
            LogicPayload::AstStmts(_) => false, // NavCall handled at view tree level
            LogicPayload::Bytecode(_) => false, // Can't analyze bytecode
        }
    }

    /// Check if widget uses router features (Plan 105)
    pub(super) fn widget_needs_router(&self, widget: &AuraWidget) -> bool {
        // Check handlers for NavCall
        for payload in widget.handlers.values() {
            if Self::payload_has_nav_call(payload) {
                return true;
            }
        }
        false
    }

    /// Check if LogicPayload contains route access (Plan 235)
    pub(super) fn payload_has_route_access(payload: &LogicPayload) -> bool {
        match payload {
            LogicPayload::AstStmts(stmts) => {
                crate::ui_gen::ts_adapter::stmts_have_route_access(stmts)
            }
            LogicPayload::Bytecode(_) => false,
        }
    }

    /// Check if widget uses route features (Plan 235)
    pub(super) fn widget_needs_route(widget: &AuraWidget) -> bool {
        for payload in widget.handlers.values() {
            if Self::payload_has_route_access(payload) {
                return true;
            }
        }
        false
    }

    /// Escape a string for use in JavaScript single-quoted string literals.
    pub(super) fn escape_js_string(s: &str) -> String {
        s.replace("\\", "\\\\")
            .replace("'", "\\'")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
    }

    /// Plan 451 P2: escape for a double-quoted HTML attribute value.
    pub(super) fn escape_html_attr(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// Plan 451 P2: escape for HTML text content.
    pub(super) fn escape_html_text(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// Plan 408 P9 / §7.2 缺陷 3: if a statement list is a single expression
    /// statement (`Expr(e)`), return its JS form so callers can use it as a
    /// ternary branch value. Returns None for multi-statement bodies (which
    /// need the IIFE block+return form) or when the expression fails to
    /// transpile (falls back to IIFE).
    pub(super) fn single_body_expr_js(&self, stmts: &[auto_lang_core::ast::Stmt]) -> Option<String> {
        if stmts.len() == 1 {
            if let auto_lang_core::ast::Stmt::Expr(e) = &stmts[0] {
                return self.expr_to_js(e).ok();
            }
        }
        None
    }

    /// Map an Auto `.method()` call on a string/array receiver to its JS
    /// equivalent (Plan 053 M1 table; semantics verified against
    /// `libs/string.rs`). Returns `None` for unmapped methods so callers fall
    /// through to pass-through output.
    ///
    /// This is the shared single source for the **third** codegen path (view
    /// expressions: view fn bodies / template attribute values & text
    /// interpolation) — `expr_to_js` (computed/template) and
    /// `ts_adapter::transpile_expr` (handler body) carry the same table
    /// (P5-3: the view paths used to skip it, emitting invalid JS like
    /// `.to_float()`).
    pub(super) fn map_method_to_js(
        &self,
        method: &str,
        object_js: &str,
        args_js: &[String],
    ) -> Option<String> {
        let a = args_js.join(", ");
        // Plan 028 M1: a CLOSURE argument means receiver semantics the string
        // table can't know — `.find(e => …)` is Array.find, never indexOf.
        if args_js.iter().any(|x| x.contains("=>"))
            && matches!(
                method,
                "find" | "replace" | "contains" | "remove" | "slice" | "substr" | "sub"
            )
        {
            return None;
        }
        Some(match method {
            "len" => format!("{}.length", object_js),
            // Plan 345 (gap N1): Auto `.contains` maps to JS `.includes`
            "contains" => format!("{}.includes({})", object_js, a),
            "to_string" | "str" => format!("{}.toString()", object_js),
            "to_int" | "parse_int" => {
                if args_js.is_empty() {
                    format!("parseInt({})", object_js)
                } else {
                    format!("parseInt({}, {})", object_js, a)
                }
            }
            "to_float" | "to_double" | "parse_float" => format!("parseFloat({})", object_js),
            // 无参 — 大小写 + trim 方向：
            "to_lower" | "lower" => format!("{}.toLowerCase()", object_js),
            "to_upper" | "upper" => format!("{}.toUpperCase()", object_js),
            "trim_left" => format!("{}.trimStart()", object_js),
            "trim_right" => format!("{}.trimEnd()", object_js),
            // 有参 — 前后缀：
            "starts_with" => format!("{}.startsWith({})", object_js, a),
            "ends_with" => format!("{}.endsWith({})", object_js, a),
            // char_at 返回 1 字符 string（.at 按 Unicode char 索引；
            // JS charAt 按 UTF-16 code unit，BMP 字符一致）：
            "char_at" => format!("{}.charAt({})", object_js, a),
            // Plan 028 F3: char_code_at → charCodeAt（token 估算/avatar hash）。
            "char_code_at" => format!("{}.charCodeAt({})", object_js, a),
            // find 双语义守卫：实参为 lambda（transpile 后含 `=>`）时是
            // 数组 find 谓词语义（JS 同名），保持原样输出；否则是字符串
            // str::find 语义，返回 index 或 -1 → indexOf：
            "find" if args_js.first().map_or(false, |a| a.contains("=>")) => {
                format!("{}.find({})", object_js, a)
            }
            // find 返回 index 或 -1：
            "find" => format!("{}.indexOf({})", object_js, a),
            // substr 是 (start, LEN) 语义、slice/sub 是 (start, END) 语义
            // (VM 双 shim + vm_slice_sub_end_substr_len 测试钉死)。JS substring
            // 是 END 语义 → substr 双参必须换算 substring(start, start+len)
            // (实参括号包裹防表达式优先级),单参与 slice/sub 直接透传。
            // Plan 065:此前三名统一直译,substr(i,1) 双参恒取空串 —— vue 轨
            // chips(ai_next 引号扫描)/OnEnter 括号配平等解析全灭的根因。
            "substr" => {
                if args_js.len() == 2 {
                    format!(
                        "{}.substring({}, ({}) + ({}))",
                        object_js, args_js[0], args_js[0], args_js[1]
                    )
                } else {
                    format!("{}.substring({})", object_js, a)
                }
            }
            "sub" | "slice" => format!("{}.substring({})", object_js, a),
            // Rust str::replace 替换所有 → replaceAll：
            "replace" => format!("{}.replaceAll({})", object_js, a),
            "repeat" => format!("{}.repeat({})", object_js, a),
            // 布尔（无参）：
            "is_empty" => format!("({}.length === 0)", object_js),
            // JS 无原生字符串 reverse：
            "reverse" => format!("([...{}].reverse().join(''))", object_js),
            _ => return None,
        })
    }

    /// Convert AuraExpr to JS value string
    pub(super) fn expr_to_js(&self, expr: &auto_lang_core::ast::Expr) -> GenResult<String> {
        use auto_lang_core::ast::Expr;
        use auto_val::Op;
        match expr {
            Expr::Str(s) | Expr::CStr(s) => Ok(format!("'{}'", Self::escape_js_string(s.as_str()))),
            Expr::Int(n) => Ok(n.to_string()),
            Expr::I8(n) => Ok((*n as i32).to_string()),
            Expr::U8(n) => Ok((*n as i32).to_string()),
            Expr::Byte(n) => Ok((*n as i32).to_string()),
            Expr::Uint(n) => Ok((*n as i32).to_string()),
            Expr::Char(c) => Ok((*c as i32).to_string()),
            Expr::I64(n) => Ok(n.to_string()),
            Expr::U64(n) => Ok((*n as i64).to_string()),
            Expr::Float(n, _) | Expr::Double(n, _) => Ok(n.to_string()),
            Expr::Bool(b) => Ok(b.to_string()),
            Expr::Ident(name) => {
                let resolved = if name.starts_with('.') {
                    &name[1..]
                } else {
                    name.as_str()
                };
                if self.prop_names.contains(&resolved.to_string()) {
                    // Props: access via props.xxx (no .value, but need props. prefix in script)
                    Ok(format!("props.{}", resolved))
                } else if self.state_names.contains(&resolved.to_string()) {
                    Ok(format!("{}.value", resolved))
                } else if self.computed_names.contains(resolved) {
                    // Plan 012 Batch A (gap 44): computed refs need .value in
                    // script expressions (template side auto-unwraps already).
                    Ok(format!("{}.value", resolved))
                } else {
                    Ok(resolved.to_string())
                }
            }
            // Plan 012 Batch A (gap 47): Auto `null`/`nil` in script expressions
            // must emit JS `null`, not fall through to the catch-all's
            // `undefined` (which also broke `!= null` comparisons).
            Expr::Null | Expr::Nil | Expr::None => Ok("null".to_string()),
            Expr::Bina(left, op, right) => {
                // Plan 012 Batch A (gap 47): `x != null` / `x == null` in the DSL
                // means "has a value" / "is absent" — JS loose null check
                // (`!= null` covers both null and undefined). Strict `!== null`
                // would let `undefined` through; `!== undefined` (the old output)
                // flipped semantics when a parent explicitly passes null.
                let nullish = |e: &auto_lang_core::ast::Expr| {
                    matches!(e, Expr::Null | Expr::Nil | Expr::None)
                };
                if matches!(op, Op::Eq | Op::Neq) && (nullish(left) || nullish(right)) {
                    let other = if nullish(left) { right } else { left };
                    let other_js = self.expr_to_js(other)?;
                    let op_js = if matches!(op, Op::Eq) { "==" } else { "!=" };
                    return Ok(format!("{} {} null", other_js, op_js));
                }
                // PLAN-026 缺陷③: `x != ""` / `x == ""` 必须按 truthy/falsy 处理，
                // 不能翻译成严格 `!== ''` —— 后者对 undefined 字段（属性缺失）
                // 误判为 true（`undefined !== ''`），导致 hasError/hasThinking 等
                // 守卫永远成立（渲染空的 error/thinking 占位）。镜像 `!!x` 语义。
                let empty_str =
                    |e: &auto_lang_core::ast::Expr| matches!(e, Expr::Str(s) if s.is_empty());
                if matches!(op, Op::Eq | Op::Neq) && (empty_str(left) || empty_str(right)) {
                    let other = if empty_str(left) { right } else { left };
                    let other_js = self.expr_to_js(other)?;
                    // `x != ""` → !!x（非空·非 null·非 undefined）；`x == ""` → !x
                    return Ok(if matches!(op, Op::Neq) {
                        format!("!!({})", other_js)
                    } else {
                        format!("!({})", other_js)
                    });
                }
                let mut left_js = self.expr_to_js(left)?;
                let mut right_js = self.expr_to_js(right)?;
                // Re-insert the parens the parser dropped (probe 09):
                // `(a+b)*c` must not come out as `a + b * c`.
                if crate::ui_gen::ts_adapter::bina_child_needs_parens(op, left, false) {
                    left_js = format!("({})", left_js);
                }
                if crate::ui_gen::ts_adapter::bina_child_needs_parens(op, right, true) {
                    right_js = format!("({})", right_js);
                }
                let op_js = Self::op_to_js(op);
                Ok(format!("{} {} {}", left_js, op_js, right_js))
            }
            Expr::Unary(op, operand) => {
                let mut operand_js = self.expr_to_js(operand)?;
                // `!` / unary `-` bind tighter than any binop — keep the
                // operand's grouping (`!(a && b)`, `-(a + b)`).
                if matches!(operand.as_ref(), Expr::Bina(..)) {
                    operand_js = format!("({})", operand_js);
                }
                let op_js = match op {
                    Op::Not => "!",
                    Op::Sub => "-",
                    _ => "!",
                };
                Ok(format!("{}{}", op_js, operand_js))
            }
            Expr::Dot(object, field) => {
                // Method call pattern: object.method(args) is parsed as
                // Call(Dot(object, method), args).
                // This branch handles plain field access.
                // `.field` is parsed as Dot(Ident("self"), field) — a widget
                // member reference. Resolve it against props/state instead of
                // emitting a nonexistent `self` object.
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "self" || name.as_str() == "." {
                        let field_name = field.as_str();
                        if self.prop_names.contains(&field_name.to_string()) {
                            return Ok(format!("props.{}", field_name));
                        } else if self.state_names.contains(&field_name.to_string()) {
                            return Ok(format!("{}.value", field_name));
                        } else if self.computed_names.contains(field_name) {
                            // Plan 012 Batch A (gap 44): a computed referenced
                            // from another computed (or any script expression)
                            // must be unwrapped — a bare ref is always truthy,
                            // silently killing boolean conditions.
                            return Ok(format!("{}.value", field_name));
                        } else {
                            // Locals are plain consts in <script setup>; the
                            // bare name is the least-wrong fallback
                            // (matches ts_adapter).
                            return Ok(field_name.to_string());
                        }
                    }
                }
                // Plan 012 Batch A (gap 44): idempotence guard for the legacy
                // workaround — users who wrote `.c.value` explicitly must not
                // get `c.value.value` now that `.c` auto-unwraps.
                if field.as_str() == "value" {
                    if let Expr::Dot(inner_obj, inner_field) = object.as_ref() {
                        if matches!(inner_obj.as_ref(), Expr::Ident(n) if n.as_str() == "self" || n.as_str() == ".")
                            && self.computed_names.contains(inner_field.as_str())
                        {
                            return Ok(format!("{}.value", inner_field));
                        }
                    }
                }
                // Plan 408 P12 §10.4: composable facade ref 字段——当转译后的
                // object 是 facade local 且 field 在 ref_fields 标注里时，注入
                // `.value`。检查在 object_js 层（而非 AST 层），因为 `.counter`
                // 的 AST 是 Dot(Ident("."), "counter")，转译后才是裸 "counter"。
                let object_js = self.expr_to_js(object)?;
                if self.is_ext_composable_local(&object_js)
                    && self
                        .facade_ref_fields
                        .get(&object_js)
                        .map(|fields| fields.contains(field.as_str()))
                        .unwrap_or(false)
                {
                    return Ok(format!("{}.{}.value", object_js, field));
                }
                // Plan 043: numeric field (tuple/element index, e.g. `field.0`)
                // must render as `field[0]` — `field.0` is valid JS but invalid
                // TypeScript in Vue templates (TS treats `.0` as a property, not
                // a tuple index).
                // plan 013 follow-up: a binop/unary RECEIVER keeps its parens
                // — `(a + b).x`, not `a + b.x` (the AST drops explicit parens).
                let object_js = if matches!(object.as_ref(), Expr::Bina(..) | Expr::Unary(..)) {
                    format!("({})", object_js)
                } else {
                    object_js
                };
                if field.as_str().chars().all(|c| c.is_ascii_digit()) && !field.as_str().is_empty()
                {
                    Ok(format!("{}[{}]", object_js, field))
                } else {
                    // Plan 408 P11 / §7.7: when the object is a function call
                    // (whose return type TS treats as un-narrowable across
                    // separate invocations), use optional chaining so `fn().field`
                    // compiles even when `fn() != None` was just checked in a
                    // sibling expression. `?.` is a no-op for non-null values.
                    let op = if matches!(object.as_ref(), Expr::Call(_)) {
                        "?."
                    } else {
                        "."
                    };
                    Ok(format!("{}{}{}", object_js, op, field))
                }
            }
            Expr::Call(call) => {
                // Plan 412 续(toast 参数化):toast()/toast.success() 等调用的
                // named 参数转 vue-sonner options 对象 ——
                // toast('msg', position: 'top-left', duration: 2000) →
                // toast('msg', { position: 'top-left', duration: 2000 })。
                // 通用 Call 路径会丢掉 Pair 的 key,故在进入前特判。
                let is_toast_call = match call.name.as_ref() {
                    Expr::Ident(n) => n.as_str() == "toast",
                    Expr::Dot(obj, _) => {
                        matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "toast")
                    }
                    _ => false,
                };
                if is_toast_call {
                    let name_js = self.expr_to_js(&call.name)?;
                    let mut call_args: Vec<String> = Vec::new();
                    let mut obj_fields: Vec<String> = Vec::new();
                    for a in &call.args.args {
                        match a {
                            auto_lang_core::ast::Arg::Pos(e) => {
                                call_args.push(self.expr_to_js(e)?);
                            }
                            auto_lang_core::ast::Arg::Pair(k, v) => {
                                obj_fields.push(format!("{}: {}", k, self.expr_to_js(v)?));
                            }
                            _ => {}
                        }
                    }
                    if !obj_fields.is_empty() {
                        call_args.push(format!("{{ {} }}", obj_fields.join(", ")));
                    }
                    return Ok(format!("{}({})", name_js, call_args.join(", ")));
                }
                // The call's name may be a Dot(object, method) — a method call.
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    let method = method.clone();
                    let args: Vec<auto_lang_core::ast::Expr> = call
                        .args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e)
                            | auto_lang_core::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .collect();

                    // Plan 132: Check if this is an API function call
                    // Case 1: Direct API call like listusers() - object is API name
                    if let Expr::Ident(name) = object.as_ref() {
                        let resolved = if name.starts_with('.') {
                            &name[1..]
                        } else {
                            name.as_str()
                        };
                        if self.is_api_function(resolved) {
                            let args_js: Vec<String> = args
                                .iter()
                                .map(|a| self.expr_to_js(a))
                                .collect::<Result<Vec<_>, _>>()?;
                            return Ok(format!("await {}({})", resolved, args_js.join(", ")));
                        }
                    }
                    // Case 2: self.<api_function>() - treat as direct API call
                    if let Expr::Ident(obj_name) = object.as_ref() {
                        if obj_name.as_str() == "self" && self.is_api_function(method.as_str()) {
                            let args_js: Vec<String> = args
                                .iter()
                                .map(|a| self.expr_to_js(a))
                                .collect::<Result<Vec<_>, _>>()?;
                            return Ok(format!("await {}({})", method, args_js.join(", ")));
                        }
                    }
                    // Case 3: Any method call where method name is an API function
                    if self.is_api_function(method.as_str()) {
                        let args_js: Vec<String> = args
                            .iter()
                            .map(|a| self.expr_to_js(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        return Ok(format!("await {}({})", method, args_js.join(", ")));
                    }

                    let object_js = self.expr_to_js(object)?;
                    // plan 013 follow-up: a binop/unary RECEIVER keeps its
                    // parens — `(a + b).toLowerCase()`, not
                    // `a + b.toLowerCase()` (the AST drops explicit parens).
                    let object_js = if matches!(object.as_ref(), Expr::Bina(..) | Expr::Unary(..)) {
                        format!("({})", object_js)
                    } else {
                        object_js
                    };
                    let args_js: Vec<String> = args
                        .iter()
                        .map(|a| self.expr_to_js(a))
                        .collect::<Result<Vec<_>, _>>()?;
                    // Plan 012 Batch A (gap 19 audit): gate `.contains →
                    // .includes` on proven string/array receivers, mirroring
                    // the ts_adapter policy. Facade/store receivers pass
                    // through unchanged with an R010 note.
                    if method.as_str() == "contains" {
                        match self.method_map_decision_for_expr(object) {
                            crate::ui_gen::ts_adapter::MethodMapDecision::Map => {}
                            decision => {
                                if matches!(
                                    decision,
                                    crate::ui_gen::ts_adapter::MethodMapDecision::PassWarn
                                ) {
                                    self.warn(
                                        "R010",
                                        crate::ui_gen::validators::Severity::Info,
                                        format!(
                                            "`.contains()` on `{}` passed through unchanged: the receiver is not a proven string/array, so the `.includes` mapping no longer applies. If this IS a string/array, declare it with that type; if it's a facade/ext object, its own `.contains` method is now called as intended.",
                                            object_js
                                        ),
                                    );
                                }
                                return Ok(format!(
                                    "{}.contains({})",
                                    object_js,
                                    args_js.join(", ")
                                ));
                            }
                        }
                    }
                    // Plan 028 F4: Regex 子集（Rust 语法为规范 + a2ts 机械转换，
                    // 已决②）。test/match/replace 三形态；命名组 `(?P<n>` →
                    // `(?<n>`；flags 不进模式、走 API 参数。`$1` 反向引用两端
                    // 一致，直通。match 补 `|| []` 对齐 Rust 侧 Vec 返回语义。
                    // 注意必须先于 map_method_to_js：后者会把裸 `replace` 重写
                    // 成 `replaceAll`，吞掉 `Regex.replace`。
                    if object_js == "Regex" {
                        let str_arg = |i: usize| -> Option<String> {
                            args.get(i).and_then(|a| match a {
                                auto_lang_core::ast::Expr::Str(s) => Some(s.as_str().to_string()),
                                _ => None,
                            })
                        };
                        let re_for = |pat_idx: usize, flag_idx: usize| -> Option<String> {
                            let pat = str_arg(pat_idx)?;
                            // Plan 028 T11 对拍缺陷修复：模式嵌入 TS 字符串前必须
                            // 转义反斜杠（`\\s` 直插 `'\\s'` 会被 JS 求值成 `s`）。
                            let js_pat = pat
                                .replace("(?P<", "(?<")
                                .replace('\\', "\\\\")
                                .replace('\'', "\\'");
                            match str_arg(flag_idx) {
                                Some(f) => Some(format!("new RegExp('{}', '{}')", js_pat, f)),
                                None => Some(format!("new RegExp('{}')", js_pat)),
                            }
                        };
                        let subject = args_js.first().cloned().unwrap_or_else(|| "''".to_string());
                        match method.as_str() {
                            "test" => {
                                if let Some(re) = re_for(1, 2) {
                                    return Ok(format!("{}.test({})", re, subject));
                                }
                            }
                            "match" => {
                                if let Some(re) = re_for(1, 2) {
                                    return Ok(format!("({}.match({}) || [])", subject, re));
                                }
                            }
                            "split" => {
                                if let Some(re) = re_for(1, 2) {
                                    return Ok(format!("{}.split({})", subject, re));
                                }
                            }
                            "replace" => {
                                if let Some(re) = re_for(1, 3) {
                                    let to =
                                        args_js.get(2).cloned().unwrap_or_else(|| "''".to_string());
                                    return Ok(format!("{}.replace({}, {})", subject, re, to));
                                }
                            }
                            _ => {}
                        }
                    }
                    // Plan 053 M1/P5-3: 方法映射表收敛到 map_method_to_js 单源，
                    // 与 view 表达式路径共享同一张表。
                    if let Some(mapped) =
                        self.map_method_to_js(method.as_str(), &object_js, &args_js)
                    {
                        return Ok(mapped);
                    }
                    // Plan 028 F3: `Date.format(ts, "HH:mm")` — 平台日期 API 的
                    // 窄面（避免暴露整个 Date），映射到 toLocaleTimeString 的
                    // 2-digit 选项。模式支持 HH:mm / HH:mm:ss。
                    // PLAN-536 T4(题3): 秒/毫秒双口径归一（1e11 阈值,与 VM
                    // format_date_ms 同口径）——epoch 秒直入亦可,调用方无需
                    // 自行 ×1000（VM int i32 回绕,musk msgTimeLabel 实录）。
                    if method.as_str() == "format" && object_js == "Date" {
                        let ts_arg = args_js.first().cloned().unwrap_or_else(|| "0".to_string());
                        let pattern = args_js
                            .get(1)
                            .map(|p| p.trim_matches('\'').trim_matches('"'))
                            .unwrap_or("HH:mm");
                        let mut opts = vec!["hour: '2-digit'", "minute: '2-digit'"];
                        if pattern.contains("ss") {
                            opts.push("second: '2-digit'");
                        }
                        let norm = format!(
                            "(({}) < 100000000000 ? ({}) * 1000 : ({}))",
                            ts_arg, ts_arg, ts_arg
                        );
                        return Ok(format!(
                            "new Date({}).toLocaleTimeString([], {{ {} }})",
                            norm,
                            opts.join(", ")
                        ));
                    }
                    Ok(format!("{}.{}({})", object_js, method, args_js.join(", ")))
                } else {
                    // Plain function call: func(args)
                    let name_js = self.expr_to_js(&call.name)?;
                    let args_js: Vec<String> = call
                        .args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            auto_lang_core::ast::Arg::Pos(e)
                            | auto_lang_core::ast::Arg::Pair(_, e) => Some(e.clone()),
                            _ => None,
                        })
                        .map(|a| self.expr_to_js(&a))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(format!("{}({})", name_js, args_js.join(", ")))
                }
            }
            Expr::Array(elems) => {
                let elems_js: Vec<String> = elems
                    .iter()
                    .map(|e| self.expr_to_js(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elems_js.join(", ")))
            }
            Expr::Object(pairs) => {
                // Plan 028 F1: StrKey needs quoting in JS — bare output broke
                // non-identifier keys ("task-plan"). Mirrors ts_adapter's arm.
                let pairs_js: Vec<String> = pairs
                    .iter()
                    .map(|p| {
                        let v_js = self.expr_to_js(&p.value)?;
                        let key = match &p.key {
                            auto_lang_core::ast::Key::StrKey(s) => {
                                format!("'{}'", s.as_str().replace('\'', "\\'"))
                            }
                            other => other.to_astr().to_string(),
                        };
                        Ok(format!("{}: {}", key, v_js))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", pairs_js.join(", ")))
            }
            Expr::Closure(closure) => {
                let params: Vec<String> =
                    closure.params.iter().map(|p| p.name.to_string()).collect();
                let body_js = self.expr_to_js(&closure.body)?;
                Ok(format!("({}) => {}", params.join(", "), body_js))
            }
            Expr::Index(target, index) => {
                let target_js = self.expr_to_js(target)?;
                let index_js = self.expr_to_js(index)?;
                Ok(format!("{}[{}]", target_js, index_js))
            }
            // Plan 028 F1: missing-key/default chains (`colors[k] ?? "#999"`)
            // used to hit the catch-all `undefined`. Parens keep the `??`
            // safe inside concatenation and ternaries.
            Expr::NullCoalesce(l, r) => {
                let l_js = self.expr_to_js(l)?;
                let r_js = self.expr_to_js(r)?;
                Ok(format!("({} ?? {})", l_js, r_js))
            }
            Expr::NavCall { path, params } => {
                let path_js = self.expr_to_js(path)?;
                if params.is_empty() {
                    Ok(format!("router.push({})", path_js))
                } else {
                    let params_js: Vec<String> = params
                        .iter()
                        .map(|p| {
                            self.expr_to_js(&p.value)
                                .map(|v_js| format!("{}: {}", p.key.to_astr(), v_js))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(format!(
                        "router.push({{ path: {}, query: {{ {} }} }})",
                        path_js,
                        params_js.join(", ")
                    ))
                }
            }
            // Plan 043 M5 #2: render a multi-statement computed body. The
            // computed wrapper at the call site is `computed(() => {expr_js})`,
            // so we emit `{ stmts; return tail; }` to produce a valid statement
            // arrow-body. Reuse the handler-body transpiler (same state/prop
            // rewriting rules) so the body sees `state.value` etc.
            // Plan 448 H1: the tail-return half was aspirational until now —
            // transpile_handler_body rendered the trailing expression as a
            // bare statement (computed yielded undefined; 018-book-reader's
            // NOTE was right). transpile_body_as_return (Plan 043 H1) turns
            // the trailing expression statement into `return e;`; a block
            // that already ends in an explicit `return` renders unchanged.
            Expr::Block(body) => {
                let mut ctx = crate::ui_gen::ts_adapter::AuraTsContext::new(
                    self.state_names.iter().cloned().collect(),
                )
                .with_props(self.prop_names.iter().cloned().collect())
                .with_refs(self.template_refs.iter().cloned().collect())
                .with_computed(self.computed_names.iter().cloned().collect());
                if !self.project_api_functions.is_empty() {
                    ctx = ctx.with_api_functions(self.project_api_functions.clone());
                }
                let (arrays, strings) = self.typed_collection_names();
                ctx = ctx
                    .with_typed_collections(arrays, strings)
                    .with_typed_ints(self.int_names.iter().cloned().collect())
                    .with_facade_names(self.facade_local_names())
                    .with_facade_ref_fields(self.facade_ref_fields_map());
                let body_js =
                    crate::ui_gen::ts_adapter::transpile_body_as_return(&body.stmts, &ctx);
                self.drain_ctx_warnings(&ctx);
                Ok(format!("{{ {} }}", body_js.trim()))
            }
            // Plan 043 M5 / Plan 408 P9 (§7.2 缺陷 3): if/else-if/else in
            // expression position (e.g. `computed { x => if ... else ... }`).
            // When every branch body is a single expression, emit a ternary
            // `cond ? then : else` — clean, type-inferable, no IIFE wrapper.
            // Fall back to the IIFE form for non-expression bodies (multi-stmt
            // branches), so complex cases still evaluate to a value.
            Expr::If(if_expr) => {
                // Try the ternary path: every branch + else must be a single
                // Stmt::Expr so each side is a pure expression.
                let branch_exprs: Vec<Option<String>> = if_expr
                    .branches
                    .iter()
                    .map(|b| self.single_body_expr_js(&b.body.stmts))
                    .collect();
                let else_expr = if_expr
                    .else_
                    .as_ref()
                    .and_then(|b| self.single_body_expr_js(&b.stmts));
                let all_single = branch_exprs.iter().all(|e| e.is_some())
                    && (if_expr.else_.is_none() || else_expr.is_some());
                if all_single {
                    // Build the ternary from the inside out (last branch first).
                    let mut acc = else_expr
                        .as_deref()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "undefined".to_string());
                    for (branch, then_expr) in
                        if_expr.branches.iter().rev().zip(branch_exprs.iter().rev())
                    {
                        let cond = self.expr_to_js(&branch.cond)?;
                        let then_str = then_expr.clone().unwrap();
                        // Plan 054 M6: wrap the whole ternary in parens so it's
                        // safe inside string concatenation (`'fw:' + (c ? a : b)`).
                        // Without the outer parens, JS `+` binds tighter than `?:`,
                        // so `'fw:' + c ? a : b` parses as `('fw:' + c) ? a : b`
                        // (always truthy → always 'a', e.g. CodeView恒 bold).
                        acc = format!("({} ? {} : {})", cond, then_str, acc);
                    }
                    return Ok(acc);
                }
                // Fallback: IIFE (multi-statement branches need a block + return).
                let mut ctx = crate::ui_gen::ts_adapter::AuraTsContext::new(
                    self.state_names.iter().cloned().collect(),
                )
                .with_props(self.prop_names.iter().cloned().collect())
                .with_refs(self.template_refs.iter().cloned().collect())
                .with_computed(self.computed_names.iter().cloned().collect());
                if !self.project_api_functions.is_empty() {
                    ctx = ctx.with_api_functions(self.project_api_functions.clone());
                }
                let (arrays, strings) = self.typed_collection_names();
                ctx = ctx
                    .with_typed_collections(arrays, strings)
                    .with_typed_ints(self.int_names.iter().cloned().collect())
                    .with_facade_names(self.facade_local_names())
                    .with_facade_ref_fields(self.facade_ref_fields_map());
                let mut out = String::from("(() => { ");
                for (i, branch) in if_expr.branches.iter().enumerate() {
                    let kw = if i == 0 { "if" } else { "else if" };
                    out.push_str(&format!("{} (", kw));
                    out.push_str(&self.expr_to_js(&branch.cond)?);
                    out.push_str(") { ");
                    // Plan 043 H1: branch bodies must RETURN their value so the
                    // IIFE evaluates to the expression (e.g. status_glyph '✓'),
                    // not undefined.
                    out.push_str(
                        crate::ui_gen::ts_adapter::transpile_body_as_return(
                            &branch.body.stmts,
                            &ctx,
                        )
                        .trim(),
                    );
                    out.push_str(" }");
                }
                if let Some(else_body) = &if_expr.else_ {
                    out.push_str(" else { ");
                    out.push_str(
                        crate::ui_gen::ts_adapter::transpile_body_as_return(&else_body.stmts, &ctx)
                            .trim(),
                    );
                    out.push_str(" }");
                }
                out.push_str(" })()");
                self.drain_ctx_warnings(&ctx);
                Ok(out)
            }
            // PLAN-604 T08 (KD-VM4): `expr.as(Type)` 的 JS 降级——VM 侧
            // TYPE_CAST_I32 执行 Rust `f as i32` 截断(engine.rs),此前 handler
            // 内 as-cast 落 catch-all 产 `undefined`(双端分叉)。整型目标降级
            // Math.trunc 对齐截断;浮点目标 JS 原生 f64 直通;其余类型与 VM
            // codegen 一致保持值不变。
            Expr::Cast { expr, target_type } => {
                let mut inner = self.expr_to_js(expr)?;
                if matches!(expr.as_ref(), Expr::Bina(..) | Expr::Unary(..)) {
                    inner = format!("({})", inner);
                }
                match target_type {
                    auto_lang_core::ast::Type::Int
                    | auto_lang_core::ast::Type::I64
                    | auto_lang_core::ast::Type::Uint
                    | auto_lang_core::ast::Type::U64
                    | auto_lang_core::ast::Type::USize
                    | auto_lang_core::ast::Type::Byte => Ok(format!("Math.trunc({})", inner)),
                    _ => Ok(inner),
                }
            }
            _ => Ok("undefined".to_string()),
        }
    }

    /// Convert binary operator (auto_val::Op) to JS
    pub(super) fn op_to_js(op: &auto_val::Op) -> &'static str {
        match op {
            auto_val::Op::Add => "+",
            auto_val::Op::Sub => "-",
            auto_val::Op::Mul => "*",
            auto_val::Op::Div => "/",
            auto_val::Op::Mod => "%",
            auto_val::Op::Eq => "===",
            auto_val::Op::Neq => "!==",
            auto_val::Op::Lt => "<",
            auto_val::Op::Le => "<=",
            auto_val::Op::Gt => ">",
            auto_val::Op::Ge => ">=",
            auto_val::Op::And => "&&",
            auto_val::Op::Or => "||",
            _ => "+",
        }
    }

    // ========================================================================
    // Plan 132: API Call Detection for Async Handlers
    // ========================================================================

    /// List of known API function names (from @/lib/api)
    pub(crate) const API_FUNCTIONS: &'static [&'static str] = &[
        "listusers",
        "getuser",
        "getUser",
        "createUser",
        "updateUser",
        "deleteUser",
    ];

}
