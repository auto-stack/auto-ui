use auto_val::AutoStr;
use auto_lang::AutoResult;
use rust_embed::RustEmbed;
use gpui::AssetSource;
use anyhow::anyhow;

#[derive(RustEmbed)]
#[folder = "../../assets/templates"]
#[include = "*.at.rs"]
pub struct Templates;

#[inline]
fn load_template(name: &str) -> AutoResult<AutoStr> {
    let file = Templates::get(name).unwrap();
    let str = String::from_utf8(file.data.as_ref().to_vec()).unwrap();
    Ok(AutoStr::from(str))
}

impl Templates {
    pub fn story() -> AutoResult<AutoStr> {
        load_template("story.at.rs")
    }

    pub fn app() -> AutoResult<AutoStr> {
        load_template("app.at.rs")
    }

    pub fn table() -> AutoResult<AutoStr> {
        load_template("table.at.rs")
    }
}

impl AssetSource for Templates {

    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{}\"", path))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| {
                if p.starts_with(path) {
                    Some(p.into())
                } else {
                    None
                }
            })
            .collect())
    }
}