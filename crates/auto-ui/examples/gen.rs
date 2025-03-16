use auto_gen::*;
use auto_val::{Obj, AutoPath};
use auto_atom::Atom;
use auto_lang::config::AutoConfig;
fn main() {
    let mut data = AutoConfig::from_file(AutoPath::new("crates/auto-ui/examples/hello.at").path(), &Obj::new()).unwrap();
    data.root.props.set("name", "story");
    let atom = Atom::node(data.root);
    println!("{}", atom);
    let gen = AutoGen::new()
        .molds(vec![Mold::from_file(AutoPath::new("assets/templates/story.at.rs"))])
        .data(atom)
        .out(AutoPath::new("crates/auto-ui/examples/"));
    let result = gen.gen();
    println!("{}", result);
}