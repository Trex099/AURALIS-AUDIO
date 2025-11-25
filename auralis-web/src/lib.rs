use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/assets/"]
pub struct Assets;

pub fn get_asset(path: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
    Assets::get(path).map(|f| f.data)
}
