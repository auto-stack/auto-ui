//! PLAN-113 T-03 真实 SQLite 宿主桥（V-03 冻结合同见
//! `docs/plans/113-autoui-database-av-vue.md` T-03）。
//!
//! 026 原样源的 RunSql/CommitTx/RollbackTx 是内存样本语义（T-01 冻结）；
//! 本桥在 **宿主侧** 把同一批调度消息镜像为对真实临时 SQLite 库的读写，
//! 真实结果集/错误诊断/事务终态入 `evidence()`——样本展示与真实 SQL
//! 结果两层证据互相独立。opt-in：仅当测试入口显式安装（
//! `--sqlite-db <path>` / [`NodeDesktopBridge::install_sqlite_bridge`]），
//! 其他 app 零行为变化。库实现复用 auto-lang `a2r-std::sqlite`
//! （rusqlite 0.30 bundled，Plan 415-B1 受检面）；种子 fixture 逐行复制自
//! 冻结 026 源模型字面量（`ace_sqlite_bridge_seed`，review 对回 T-01 钉版）。

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ui::ace_sqlite_bridge_seed as seed;
use crate::ui::ace_state_bridge::AceStateValue;

/// 宿主桥：一条真实 SQLite 连接 + 事务状态 + 事件 evidence。
pub struct SqliteHostBridge {
    db: a2r_std::sqlite::SqliteDb,
    db_path: PathBuf,
    in_tx: bool,
    events: Vec<serde_json::Value>,
}

fn state_str<'a>(states: &'a [(String, AceStateValue)], name: &str) -> Option<&'a str> {
    states
        .iter()
        .find(|(n, _)| n.trim_start_matches('.') == name)
        .and_then(|(_, v)| match v {
            AceStateValue::Str(s) => Some(s.as_str()),
            _ => None,
        })
}

/// SQL 字符串字面量转义（单引号翻倍）。
fn quote(s: &str) -> String {
    s.replace('\'', "''")
}

impl SqliteHostBridge {
    /// 打开（不存在则创建）真实库文件并写入种子。失败具名返回
    /// （a2r-std open 失败时落内存库并记 `last_error`——此处 fail-closed，
    /// 真实库是本桥的验收对象，不允许静默降级）。
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("sqlite bridge mkdir {}: {e}", parent.display()))?;
        }
        let db = a2r_std::sqlite::open(path.to_string_lossy().as_ref());
        let err = a2r_std::sqlite::last_error();
        if !err.is_empty() {
            return Err(format!("sqlite open {}: {err}", path.display()));
        }
        let mut bridge = Self {
            db,
            db_path: path.to_path_buf(),
            in_tx: false,
            events: Vec::new(),
        };
        bridge.seed()?;
        Ok(bridge)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    fn seed(&mut self) -> Result<(), String> {
        for ddl in seed::DDL {
            a2r_std::sqlite::clear_error();
            self.db.exec(ddl);
            let err = a2r_std::sqlite::last_error();
            if !err.is_empty() {
                return Err(format!("sqlite seed ddl `{ddl}`: {err}"));
            }
        }
        self.insert_seed_rows(
            "customers",
            "(id, company_name, contact_name, city, country, phone)",
            seed::CUSTOMERS,
        )?;
        self.insert_seed_rows(
            "products",
            "(id, product_name, category, unit_price, units_in_stock, status)",
            seed::PRODUCTS,
        )?;
        self.insert_seed_rows(
            "orders",
            "(id, customer_id, order_date, ship_country, freight, status)",
            seed::ORDERS,
        )?;
        self.push_event("seed", serde_json::json!({
            "tables": ["customers", "products", "orders"],
            "rows": [seed::CUSTOMERS.len(), seed::PRODUCTS.len(), seed::ORDERS.len()],
        }));
        Ok(())
    }

    fn insert_seed_rows(
        &mut self,
        table: &str,
        columns: &str,
        rows: &[seed::SeedRow],
    ) -> Result<(), String> {
        for r in rows {
            let sql = format!(
                "INSERT INTO {table} {columns} VALUES ({}, '{}', '{}', '{}', '{}', '{}')",
                r.id,
                quote(r.cols[0]),
                quote(r.cols[1]),
                quote(r.cols[2]),
                quote(r.cols[3]),
                quote(r.cols[4]),
            );
            a2r_std::sqlite::clear_error();
            self.db.exec(&sql);
            let err = a2r_std::sqlite::last_error();
            if !err.is_empty() {
                return Err(format!("sqlite seed insert {table}#{}: {err}", r.id));
            }
        }
        Ok(())
    }

    /// 调度消息镜像入口：按 V-03 合同把消息映射为真实库操作。
    /// 在 `NodeDesktopBridge::dispatch_event` 的引擎派发**之前**调用——
    /// app 面惰性 handler 跳过不阻塞真实执行（两层独立）。
    pub fn notify(&mut self, msg: &str, states: &[(String, AceStateValue)]) {
        match msg {
            "RunSql" => self.run_sql(states),
            "SaveNewRow" => self.save_new_row(states),
            "CommitTx" => self.end_tx(true),
            "RollbackTx" => self.end_tx(false),
            "ResetDb" => self.reset_db(),
            _ => {}
        }
    }

    fn run_sql(&mut self, states: &[(String, AceStateValue)]) {
        let sql = state_str(states, "sqlInput").unwrap_or_default().to_string();
        if sql.trim().is_empty() {
            return;
        }
        let started = Instant::now();
        a2r_std::sqlite::clear_error();
        let upper = sql.trim_start().to_ascii_uppercase();
        let is_query = upper.starts_with("SELECT")
            || upper.starts_with("WITH")
            || upper.starts_with("PRAGMA")
            || upper.starts_with("EXPLAIN");
        let (rows, error) = if is_query {
            let rows = self.db.query(&sql);
            (rows.len(), a2r_std::sqlite::last_error())
        } else {
            let n = self.db.exec(&sql);
            let err = a2r_std::sqlite::last_error();
            (if err.is_empty() { n.max(0) as usize } else { 0 }, err)
        };
        self.push_event("RunSql", serde_json::json!({
            "sql": sql,
            "ok": error.is_empty(),
            "rows": rows,
            "error": error,
            "duration_ms": started.elapsed().as_millis(),
        }));
    }

    fn save_new_row(&mut self, states: &[(String, AceStateValue)]) {
        let table = state_str(states, "activeTable").unwrap_or("customers");
        if table != "customers" {
            self.push_event("SaveNewRow", serde_json::json!({
                "skipped": format!("activeTable={table}（合同仅 customers 新增表单）"),
            }));
            return;
        }
        let name = state_str(states, "newName").unwrap_or_default();
        let contact = state_str(states, "newContact").unwrap_or_default();
        let city = state_str(states, "newCity").unwrap_or_default();
        let country = state_str(states, "newCountry").unwrap_or_default();
        // 首个 staging 写开真实事务（CommitTx/RollbackTx 收尾）。
        if !self.in_tx {
            a2r_std::sqlite::clear_error();
            self.db.exec("BEGIN");
            self.in_tx = true;
        }
        let sql = format!(
            "INSERT INTO customers (company_name, contact_name, city, country, phone) \
             VALUES ('{}', '{}', '{}', '{}', 'N/A')",
            quote(name),
            quote(contact),
            quote(city),
            quote(country),
        );
        a2r_std::sqlite::clear_error();
        let affected = self.db.exec(&sql);
        let error = a2r_std::sqlite::last_error();
        self.push_event("SaveNewRow", serde_json::json!({
            "sql": sql,
            "ok": error.is_empty(),
            "affected": affected,
            "in_tx": true,
            "error": error,
        }));
    }

    fn end_tx(&mut self, commit: bool) {
        if !self.in_tx {
            self.push_event(
                if commit { "CommitTx" } else { "RollbackTx" },
                serde_json::json!({ "ok": true, "noop": true, "in_tx": false }),
            );
            return;
        }
        let op = if commit { "COMMIT" } else { "ROLLBACK" };
        a2r_std::sqlite::clear_error();
        self.db.exec(op);
        let error = a2r_std::sqlite::last_error();
        self.in_tx = false;
        self.push_event(
            if commit { "CommitTx" } else { "RollbackTx" },
            serde_json::json!({ "ok": error.is_empty(), "in_tx": false, "error": error }),
        );
    }

    fn reset_db(&mut self) {
        if self.in_tx {
            self.db.exec("ROLLBACK");
            self.in_tx = false;
        }
        a2r_std::sqlite::clear_error();
        for table in ["customers", "products", "orders"] {
            self.db.exec(&format!("DELETE FROM {table}"));
        }
        let err = a2r_std::sqlite::last_error();
        self.push_event("ResetDb", serde_json::json!({ "ok": err.is_empty(), "error": err }));
        let _ = self.seed();
    }

    fn push_event(&mut self, msg: &str, detail: serde_json::Value) {
        self.events.push(serde_json::json!({
            "seq": self.events.len(),
            "msg": msg,
            "detail": detail,
        }));
    }

    /// 真实 SQL 结果 evidence（V-03 与样本展示证据独立成层）。
    pub fn evidence(&self) -> serde_json::Value {
        serde_json::json!({
            "db_path": self.db_path.to_string_lossy(),
            "in_tx": self.in_tx,
            "events": self.events,
        })
    }
}
