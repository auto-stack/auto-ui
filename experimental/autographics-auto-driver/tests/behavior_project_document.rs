use autographics_auto_driver::authoring::{behavior::VERSION, behavior_project::*, document::*};
use serde_json::{json, Value};
fn binding() -> Binding {
    Binding {
        api: VERSION.into(),
        source: CodeRef {
            format: "auto-source".into(),
            sha256: "a".repeat(64),
            byte_length: 12,
        },
        actors: Actors {
            two_d: Some(1),
            three_d: Some(3),
        },
    }
}
#[test]
fn strict_scene4_and_legacy_migration() {
    let mut d = Document::sample();
    d.objects[0].id = u64::MAX;
    let mut b = binding();
    b.actors.two_d = Some(u64::MAX);
    d.behavior = Some(b);
    let raw = d.encode().unwrap();
    assert_eq!(Document::decode(&raw).unwrap(), d);
    let good: Value = serde_json::from_slice(&raw).unwrap();
    for key in [
        "behavior",
        "api",
        "source",
        "sha256",
        "byte_length",
        "format",
        "actors",
        "two_d",
        "three_d",
    ] {
        let mut v = good.clone();
        match key {
            "behavior" => {
                v.as_object_mut().unwrap().remove(key);
            }
            "api" | "source" | "actors" => {
                v["behavior"].as_object_mut().unwrap().remove(key);
            }
            "two_d" | "three_d" => {
                v["behavior"]["actors"].as_object_mut().unwrap().remove(key);
            }
            _ => {
                v["behavior"]["source"].as_object_mut().unwrap().remove(key);
            }
        }
        assert!(
            Document::decode(&serde_json::to_vec(&v).unwrap()).is_err(),
            "missing {key}"
        );
    }
    for (pointer, value) in [
        ("/behavior/api", json!("wrong")),
        ("/behavior/source/sha256", json!("A".repeat(64))),
        ("/behavior/source/byte_length", json!(0)),
        ("/behavior/source/byte_length", json!(262145)),
        ("/behavior/actors/two_d", json!(3)),
        ("/behavior/actors/three_d", json!(1)),
        ("/behavior/extra", json!(true)),
    ] {
        let mut v = good.clone();
        if pointer.ends_with("extra") {
            v["behavior"]["extra"] = value;
        } else {
            *v.pointer_mut(pointer).unwrap() = value;
        }
        assert!(
            Document::decode(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{pointer}"
        );
    }
    let duplicate = String::from_utf8(raw)
        .unwrap()
        .replace("\"api\":", "\"api\":null,\"api\":");
    assert!(Document::decode(duplicate.as_bytes()).is_err());
    for version in [1, 2, 3] {
        let mut v = serde_json::to_value(Document::sample()).unwrap();
        v["schema"] = json!(format!("autographics.scene/{version}"));
        v.as_object_mut().unwrap().remove("behavior");
        if version < 3 {
            v.as_object_mut().unwrap().remove("assets");
            for o in v["objects"].as_array_mut().unwrap() {
                o.as_object_mut().unwrap().remove("resource");
            }
        }
        if version == 1 {
            v.as_object_mut().unwrap().remove("groups");
            for o in v["objects"].as_array_mut().unwrap() {
                o.as_object_mut().unwrap().remove("parent");
            }
        }
        let bytes = serde_json::to_vec(&v).unwrap();
        let restored = Document::decode(&bytes).unwrap();
        assert_eq!(restored, Document::sample());
        assert_eq!(serde_json::to_vec(&v).unwrap(), bytes);
        v["behavior"] = Value::Null;
        assert!(Document::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    d.behavior.as_mut().unwrap().actors = Actors {
        two_d: None,
        three_d: None,
    };
    d.encode().unwrap();
}
#[test]
fn bound_group_and_ancestor_deletion_are_invalid() {
    let mut d = Document::sample();
    d.groups.push(Group::new(20, Dimension::Two));
    d.objects[0].parent = Some(20);
    d.behavior = Some(binding());
    d.validate().unwrap();
    d.behavior.as_mut().unwrap().actors.two_d = Some(20);
    assert!(d.validate().is_err());
}
