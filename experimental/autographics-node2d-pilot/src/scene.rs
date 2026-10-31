//! 场景文档生成与源身份（合同 §2 段 5/8）。
//!
//! autographics.scene/6 工程（无音频/HUD 负载的最小玩法场景）：player 模板在位（char 绑定），ground 静态，
//! coin/hazard 模板停靠在玩法区外（实例从模板 spawn，杜绝 born-overlap）。
//! 行为绑定 = triggers2d + 生成源（sha256/byte_length 精确绑定）。

use auto_parser_api::ast::Expr;
use auto_ui_dialect::node2d::View2dElement;

use crate::load::{node_of, Project, ResourceKind};
use crate::lower::Instance;

/// 冻结场景模板表：与生成源里的模板 id 一致。
pub fn template_id(inst: &Instance) -> usize {
    match inst.node.as_str() {
        "Player" => 1,
        "Coin" => 3,
        "Hazard" => 4,
        other => panic!("untemplated node {other} (scene table is frozen: 1/3/4)"),
    }
}

// ---- 纯 Rust sha256（无新增依赖）----

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bitlen = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

fn rgba_of(project: &Project, node: &str, element: &str) -> (f64, f64, f64, f64) {
    // 元素 material: resources.<name> → 颜色四元组。
    let node_decl = node_of(project, node);
    let view = node_decl.view2d.as_ref().expect("view-2d");
    let mut found = None;
    for el in &view.elements {
        let (ename, props) = match el {
            View2dElement::Solid { name, props } => (name, props),
            View2dElement::Trigger { name, props } => (name, props),
            _ => continue,
        };
        if ename == element {
            for pr in props {
                if pr.name == "material" {
                    found = Some(pr.value.clone());
                }
            }
        }
    }
    let material = found.expect("element material");
    // PropValue::One(Dot(Ident("resources"), name)) → 资源名。
    if let auto_ui_dialect::node2d::PropValue::One(Expr::Dot(_, m)) = &material {
        let module = &project.modules["resources"];
        for r in &module.resources {
            if &r.name == m.as_str() {
                if let ResourceKind::Color(r, g, b, a) = r.kind {
                    return (r, g, b, a);
                }
            }
        }
    }
    panic!("material color not found for {node}.{element}")
}

fn shape_of(project: &Project, node: &str, element: &str) -> (String, f64, f64) {
    let node_decl = node_of(project, node);
    let view = node_decl.view2d.as_ref().expect("view-2d");
    for el in &view.elements {
        let (ename, props) = match el {
            View2dElement::Solid { name, props } => (name, props),
            View2dElement::Trigger { name, props } => (name, props),
            _ => continue,
        };
        if ename == element {
            let mut mesh = None;
            for pr in props {
                if pr.name == "mesh" {
                    mesh = Some(pr.value.clone());
                }
            }
            let mesh = mesh.expect("element mesh");
            if let auto_ui_dialect::node2d::PropValue::One(Expr::Dot(_, m)) = &mesh {
                let module = &project.modules["resources"];
                for r in &module.resources {
                    if &r.name == m.as_str() {
                        return match r.kind {
                            ResourceKind::Quad(w, h) => ("quad".into(), w, h),
                            ResourceKind::Circle(d) => ("circle".into(), d / 2.0, d / 2.0),
                            ResourceKind::Color(..) => panic!("mesh is a color"),
                        };
                    }
                }
            }
            panic!("mesh resource not found for {node}.{element}")
        }
    }
    panic!("element {element} not found on {node}")
}

fn obj_json(
    id: usize,
    name: &str,
    kind: &str,
    tx: f64,
    ty: f64,
    sx: f64,
    sy: f64,
    rgba: (f64, f64, f64, f64),
) -> String {
    format!(
        "  {{\n   \"id\": {id},\n   \"name\": \"{name}\",\n   \"kind\": \"{kind}\",\n   \"translation\": [{}, {}, 0.0],\n   \"scale\": [{}, {}, 1.0],\n   \"yaw\": 0.0,\n   \"pitch\": 0.0,\n   \"rgba\": [{}, {}, {}, {}],\n   \"visible\": true,\n   \"parent\": null,\n   \"resource\": null,\n   \"sprite\": null\n  }}",
        crate::scene::fmt_f(tx),
        crate::scene::fmt_f(ty),
        crate::scene::fmt_f(sx),
        crate::scene::fmt_f(sy),
        rgba.0,
        rgba.1,
        rgba.2,
        rgba.3
    )
}

pub fn fmt_f(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

/// 场景文档 JSON（contract：player 在位、ground 静态、coin/hazard 模板停靠）。
pub fn build_scene_json(
    project: &Project,
    instances: &[Instance],
    source_sha256: &str,
    source_bytes: usize,
) -> String {
    // 冻结模板表要求全部四定义在场（minimal 工程）；补充工程只消费
    // world.at，不建场景。
    if !project.node_modules.contains_key("Hazard") {
        let _ = instances;
        return String::new();
    }
    // 模板几何/颜色来自定义 view（第一个 solid/trigger 元素）。
    let player_shape = shape_of(project, "Player", "body");
    let player_rgba = rgba_of(project, "Player", "body");
    let ground_shape = shape_of(project, "Main", "ground");
    let ground_rgba = rgba_of(project, "Main", "ground");
    let coin_shape = shape_of(project, "Coin", "body");
    let coin_rgba = rgba_of(project, "Coin", "body");
    let hazard_shape = shape_of(project, "Hazard", "body");
    let hazard_rgba = rgba_of(project, "Hazard", "body");
    let _ = instances;

    // 停靠点（合同 S3：模板本体在玩法区外）。
    let park = -100.0;

    let objects = [
        obj_json(
            1,
            "player",
            &player_shape.0,
            2.0,
            1.0,
            player_shape.1,
            player_shape.2,
            player_rgba,
        ),
        obj_json(
            2,
            "ground",
            &ground_shape.0,
            15.0,
            -0.5,
            ground_shape.1,
            ground_shape.2,
            ground_rgba,
        ),
        obj_json(
            3,
            "coin_template",
            &coin_shape.0,
            park,
            park,
            coin_shape.1,
            coin_shape.2,
            coin_rgba,
        ),
        obj_json(
            4,
            "hazard_template",
            &hazard_shape.0,
            park,
            park,
            hazard_shape.1,
            hazard_shape.2,
            hazard_rgba,
        ),
    ]
    .join(",\n");

    format!(
        "{{\n \"behavior\": {{\n  \"api\": \"autographics.triggers2d/1\",\n  \"source\": {{\n   \"format\": \"auto-source\",\n   \"sha256\": \"{source_sha256}\",\n   \"byte_length\": {source_bytes}\n  }},\n  \"actors\": {{\n   \"two_d\": 1,\n   \"three_d\": null\n  }}\n }},\n \"schema\": \"autographics.scene/6\",\n \"document_id\": \"plan016-minimal-node2d\",\n \"resources\": [\n  \"circle\",\n  \"quad\",\n  \"cylinder\",\n  \"player\",\n  \"mob\"\n ],\n \"objects\": [\n{objects}\n ],\n \"groups\": [],\n \"assets\": []\n}}\n"
    )
}
