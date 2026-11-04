use autographics_auto_driver::authoring::behavior::{Api, SourceSnapshot};
use autographics_auto_driver::Session;
use std::path::Path;
#[test]
fn comment_theory() {
    let tail = "fn app_start() {\n}\nfn app_update(dt float) {\n}\n";
    for (tag, pre) in [("c0",""),("c1","// a\n"),("c2","// a\n// bb\n"),("c3","// a\n// b\n// c\n// d\n// e\n")] {
        let user = format!("{pre}fn foo() {{\n  return missing_fn(1)\n}}\n{tail}");
        let expect = user.find("missing_fn").unwrap();
        let snap = SourceSnapshot::from_bytes_for(Path::new("u.at"), user.as_bytes().to_vec(), Api::Objects2V1).unwrap();
        let got = std::cell::RefCell::new(None);
        let _ = Session::new_with_diagnostics(&snap.source, |_, e| { *got.borrow_mut() = Some(format!("{e:?}")); String::new() });
        let region = snap.regions.iter().find(|r| r.file == "u.at").unwrap();
        let text = got.into_inner().unwrap();
        let p = text.find("offset: SourceOffset(").unwrap();
        let rel: usize = text[p+21..].split(')').next().unwrap().parse().unwrap() - region.start;
        eprintln!("DBG {tag}: expect={expect} rel={rel} shift={}", rel as isize - expect as isize);
    }
}
