#![cfg(feature = "ace-bridge")]

use auto_ui_runtime::ui::ace_package_sfc::generate_linked_front_sfc;
use std::path::Path;

#[test]
fn checked_vue_embeds_official_image_viewport() {
    let package = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/image-viewer")
        .canonicalize()
        .expect("official image-viewer package");
    let entry = package.join("viewport_test_host.at");
    let source = r#"
widget Viewer {
    use { package: image_viewer from "." }
    view { ImageViewport (src: "/photo.png", resourceId: "sample") {} }
}
"#;
    let product = generate_linked_front_sfc(&entry, source, &[])
        .expect("checked Vue generation")
        .expect("the package entry selects the checked lane");
    assert_eq!(
        product
            .embedded_widget_sources
            .into_iter()
            .collect::<Vec<_>>(),
        [package.join("image_viewport.at")]
    );
    assert!(product.app.contains("image_viewport_view_request"));
}
