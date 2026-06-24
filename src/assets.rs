use anyhow::Result;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Asset;

pub fn list_supported_kmi() -> Vec<String> {
    let mut list = Vec::new();
    for file in Asset::iter() {
        if let Some(kmi) = file.strip_suffix("-lkmloader.ko") {
            list.push(kmi.to_string());
        }
    }
    list
}

pub fn get_asset_data(name: &str) -> Result<std::borrow::Cow<'static, [u8]>> {
    let asset = Asset::get(name).ok_or_else(|| anyhow::anyhow!("asset not found: {name}"))?;

    Ok(asset.data)
}
