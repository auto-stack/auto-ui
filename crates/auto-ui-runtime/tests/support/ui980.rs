pub fn require_prepared() -> String {
    std::env::var("ACE_NODE_DESKTOP_PREPARED_RUSTC").expect(
        "PREPARATION_REQUIRED: prepare the UI980 provider with ace_node_desktop_window --prep-only \
         and set ACE_NODE_DESKTOP_PREPARED_RUSTC to the build rustc --version",
    )
}
