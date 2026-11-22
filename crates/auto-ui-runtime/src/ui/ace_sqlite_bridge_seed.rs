//! PLAN-113 T-03 真实 SQLite 宿主桥种子 fixture。
//!
//! 逐行复制自冻结 026 原样源 `examples/ui/026-database/src/front/app.at`
//! 模型字面量（T-01 钉版 SHA256 前 16 = D37E703351DEDFEB；review 可逐行
//! 对回：`customers = [` 15 行 / `products = [` 12 行 / `orders = [` 10 行）。
//! 列值保持源字面量原串（含 `$18.00`、`39 in stock` 显示形态），
//! id 为整型。DDL 与 `.schemaDdl` 同款（customers/products/orders 三表）。

/// 一行种子：整型 id + 源字面量五列（col1..col5 原串）。
pub struct SeedRow {
    pub id: i64,
    pub cols: [&'static str; 5],
}

pub const CUSTOMERS: &[SeedRow] = &[
    SeedRow { id: 1, cols: ["Alfreds Futterkiste", "Maria Anders", "Berlin", "Germany", "030-0074321"] },
    SeedRow { id: 2, cols: ["Ana Trujillo Emparedados", "Ana Trujillo", "México D.F.", "Mexico", "(5) 555-4729"] },
    SeedRow { id: 3, cols: ["Antonio Moreno Taquería", "Antonio Moreno", "México D.F.", "Mexico", "(5) 555-3932"] },
    SeedRow { id: 4, cols: ["Around the Horn", "Thomas Hardy", "London", "UK", "(171) 555-7788"] },
    SeedRow { id: 5, cols: ["Berglunds snabbköp", "Christina Berglund", "Luleå", "Sweden", "0921-12 34 65"] },
    SeedRow { id: 6, cols: ["Blauer See Delikatessen", "Hanna Moos", "Mannheim", "Germany", "0621-08460"] },
    SeedRow { id: 7, cols: ["Blondel père et fils", "Frédérique Citeaux", "Strasbourg", "France", "88.60.15.31"] },
    SeedRow { id: 8, cols: ["Bólido Comidas preparadas", "Martín Sommer", "Madrid", "Spain", "(91) 555 22 82"] },
    SeedRow { id: 9, cols: ["Bon app'", "Laurence Lebihan", "Marseille", "France", "91.24.45.40"] },
    SeedRow { id: 10, cols: ["Bottom-Dollar Markets", "Elizabeth Lincoln", "Tsawwassen", "Canada", "(604) 555-4729"] },
    SeedRow { id: 11, cols: ["B's Beverages", "Victoria Ashworth", "London", "UK", "(171) 555-1212"] },
    SeedRow { id: 12, cols: ["Cactus Comidas para llevar", "Patricio Simpson", "Buenos Aires", "Argentina", "(1) 135-5555"] },
    SeedRow { id: 13, cols: ["Centro comercial Moctezuma", "Francisco Chang", "México D.F.", "Mexico", "(5) 555-3392"] },
    SeedRow { id: 14, cols: ["Chop-suey Chinese", "Yang Wang", "Bern", "Switzerland", "0452-076545"] },
    SeedRow { id: 15, cols: ["Comércio Mineiro", "Pedro Afonso", "Sao Paulo", "Brazil", "(11) 555-7647"] },
];

pub const PRODUCTS: &[SeedRow] = &[
    SeedRow { id: 1, cols: ["Chai Tea", "Beverages", "$18.00", "39 in stock", "Active"] },
    SeedRow { id: 2, cols: ["Chang Beer", "Beverages", "$19.00", "17 in stock", "Active"] },
    SeedRow { id: 3, cols: ["Aniseed Syrup", "Condiments", "$10.00", "13 in stock", "Active"] },
    SeedRow { id: 4, cols: ["Chef Anton's Cajun Seasoning", "Condiments", "$22.00", "53 in stock", "Active"] },
    SeedRow { id: 5, cols: ["Chef Anton's Gumbo Mix", "Condiments", "$21.35", "0 in stock", "Discontinued"] },
    SeedRow { id: 6, cols: ["Grandma's Boysenberry Spread", "Condiments", "$25.00", "120 in stock", "Active"] },
    SeedRow { id: 7, cols: ["Uncle Bob's Organic Dried Pears", "Produce", "$30.00", "15 in stock", "Active"] },
    SeedRow { id: 8, cols: ["Northwoods Cranberry Sauce", "Condiments", "$40.00", "6 in stock", "Active"] },
    SeedRow { id: 9, cols: ["Mishi Kobe Niku", "Meat/Poultry", "$97.00", "29 in stock", "Active"] },
    SeedRow { id: 10, cols: ["Ikura Salmon Caviar", "Seafood", "$31.00", "31 in stock", "Active"] },
    SeedRow { id: 11, cols: ["Queso Cabrales", "Dairy Products", "$21.00", "22 in stock", "Active"] },
    SeedRow { id: 12, cols: ["Queso Manchego La Pastora", "Dairy Products", "$38.00", "86 in stock", "Active"] },
];

pub const ORDERS: &[SeedRow] = &[
    SeedRow { id: 10248, cols: ["VINET", "1996-07-04", "France", "$32.38", "Shipped"] },
    SeedRow { id: 10249, cols: ["TOMSP", "1996-07-05", "Germany", "$11.61", "Shipped"] },
    SeedRow { id: 10250, cols: ["HANAR", "1996-07-08", "Brazil", "$65.83", "Shipped"] },
    SeedRow { id: 10251, cols: ["VICTE", "1996-07-08", "France", "$41.34", "Shipped"] },
    SeedRow { id: 10252, cols: ["SUPRD", "1996-07-09", "Belgium", "$51.30", "Shipped"] },
    SeedRow { id: 10253, cols: ["HANAR", "1996-07-10", "Brazil", "$58.17", "Delivered"] },
    SeedRow { id: 10254, cols: ["CHOPS", "1996-07-11", "Switzerland", "$22.98", "Shipped"] },
    SeedRow { id: 10255, cols: ["RICSU", "1996-07-12", "Switzerland", "$148.33", "Delivered"] },
    SeedRow { id: 10256, cols: ["WELLI", "1996-07-15", "Brazil", "$13.97", "Processing"] },
    SeedRow { id: 10257, cols: ["HILAA", "1996-07-16", "Venezuela", "$81.91", "Processing"] },
];

/// 三表 DDL（与 app `.schemaDdl` 同款形状）。
pub const DDL: &[&str] = &[
    "CREATE TABLE customers (id INTEGER PRIMARY KEY AUTOINCREMENT, company_name TEXT NOT NULL, contact_name TEXT, city TEXT, country TEXT, phone TEXT)",
    "CREATE TABLE products (id INTEGER PRIMARY KEY AUTOINCREMENT, product_name TEXT NOT NULL, category TEXT NOT NULL, unit_price REAL NOT NULL DEFAULT 0.0, units_in_stock INTEGER NOT NULL DEFAULT 0, status TEXT NOT NULL DEFAULT 'Active')",
    "CREATE TABLE orders (id INTEGER PRIMARY KEY AUTOINCREMENT, customer_id TEXT NOT NULL, order_date TEXT NOT NULL, ship_country TEXT, freight REAL DEFAULT 0.0, status TEXT NOT NULL)",
];
