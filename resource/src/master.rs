use std::path::PathBuf;

use prost::Message;
use prost_reflect::{DeserializeOptions, DynamicMessage};

use types::reflection::DESCRIPTOR_POOL;

use crate::config::repo_root;

pub trait MasterTable: Message + Default + Clone + 'static {
    fn full_name() -> &'static str;

    // rows of this table. panics if not loaded yet
    fn table() -> &'static [Self];

    // rows if loaded, else None, for optional master reads that must not panic when never loaded.
    fn try_table() -> Option<&'static [Self]> {
        Self::cache().get().map(Vec::as_slice)
    }

    // cache for this table, filled by load() / load_all()
    fn cache() -> &'static ::std::sync::OnceLock<Vec<Self>>;
}

fn json_path(short_name: &str) -> PathBuf {
    repo_root()
        .join("resource")
        .join("master")
        .join(format!("{short_name}.json"))
}

// (json file name, proto name)
fn names<T: MasterTable>() -> (&'static str, String) {
    let rust_path = T::full_name();
    let short = rust_path.rsplit("::").next().expect("master type path");
    let proto_name = rust_path
        .strip_prefix("types::")
        .unwrap_or(rust_path)
        .replace("::", ".");
    (short, proto_name)
}

fn parse<T: MasterTable>(bytes: &[u8]) -> Result<Vec<T>, String> {
    let (_, proto_name) = names::<T>();

    let desc = DESCRIPTOR_POOL
        .get_message_by_name(&proto_name)
        .ok_or_else(|| format!("no descriptor for {proto_name}"))?;

    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;

    let rows = value
        .as_array()
        .ok_or_else(|| "expected a top-level JSON array".to_string())?;

    // jsons carry extra fields (eg. _row_id); they are ignored.
    let options = DeserializeOptions::new().deny_unknown_fields(false);
    rows.iter()
        .map(|row| {
            DynamicMessage::deserialize_with_options(desc.clone(), row.clone(), &options)
                .map_err(|e| format!("row: {e}"))
        })
        .map(|m| m.and_then(|m| m.transcode_to::<T>().map_err(|e| format!("transcode: {e}"))))
        .collect()
}

fn lang_file_matches(short: &str, file: &str) -> bool {
    if short == "Lang" {
        file.starts_with("Lang") && !file.starts_with("LangClient")
    } else {
        file.starts_with(short)
    }
}

// load the union of `Lang<Table>_<Locale>.json` rows for one Lang table
async fn load_lang_split<T: MasterTable>(short: &str) -> Result<(), String> {
    let dir = repo_root().join("resource").join("master");

    let mut files = Vec::new();

    let mut entries = match tokio::fs::read_dir(&dir).await {
        Ok(entries) => entries,
        Err(e) => return Err(format!("cannot read dir {}: {e}", dir.display())),
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json")
            && path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| lang_file_matches(short, n))
        {
            files.push(path);
        }
    }

    files.sort();

    let mut all = Vec::with_capacity(files.len() * 100);

    for path in files {
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;

        all.extend(parse::<T>(&bytes).map_err(|e| format!("parse {}: {e}", path.display()))?);
    }

    let _ = T::cache().set(all);

    Ok(())
}

pub async fn load<T: MasterTable>() -> Result<(), String> {
    if T::cache().get().is_some() {
        return Ok(());
    }

    let (short, _) = names::<T>();
    let path = json_path(short);

    let bytes = match tokio::fs::read(&path).await {
        Ok(bytes) => bytes,

        Err(e) if e.kind() == std::io::ErrorKind::NotFound && short.starts_with("Lang") => {
            // Lang/LangClient are dumped per table + locale (eg. LangCard_Jpn.json).
            return load_lang_split::<T>(short).await;
        }

        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };

    let rows = parse::<T>(&bytes).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let _ = T::cache().set(rows);

    Ok(())
}

macro_rules! master_tables {
    ($(($name:ident, $t:ty)),* $(,)?) => {
        $(
            impl MasterTable for $t {
                fn full_name() -> &'static str {
                    stringify!($t)
                }

                fn table() -> &'static [Self] {
                    Self::cache().get().expect("table not loaded")
                }

                fn cache() -> &'static ::std::sync::OnceLock<Vec<$t>> {
                    static CACHE: ::std::sync::OnceLock<Vec<$t>> =
                        ::std::sync::OnceLock::new();
                    &CACHE
                }
            }
        )*

        pub async fn load_all() -> Result<usize, String> {
            let mut loaded = 0usize;
            // tokio::join!()-ing these needs changing recursion limit
            $(
                load::<$t>().await?;
                loaded += 1;
            )*
            Ok(loaded)
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/master_tables.rs"));

/// raw Master/Get response captured from the original server; run `bin.ps1 master` to init.
pub const MASTER_GET_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/master_get.bin"));

#[cfg(test)]
mod tests {
    use types::entity::master::Card;
    use types::enums::CardRarity;

    use super::*;

    #[tokio::test]
    async fn card_loads_with_enum_name_strings() {
        let _ = load::<Card>().await;
        let cards = Card::table();
        assert!(!cards.is_empty());
        let first = &cards[0];
        assert_eq!(first.id, "card-00012-5-uniq-0062-00");
        assert_eq!(first.performance_permil_multiply, 306);
        assert_eq!(
            CardRarity::try_from(first.rarity).unwrap(),
            CardRarity::Rarity5
        );
    }

    #[tokio::test]
    async fn cache_is_stable() {
        let _ = load::<Card>().await;
        assert!(std::ptr::eq(Card::table(), Card::table()));
    }

    #[tokio::test]
    async fn several_tables_load() {
        use types::entity::master::{Area, Character, Gacha, Item, Music};
        let _ = load::<Area>().await;
        let _ = load::<Character>().await;
        let _ = load::<Gacha>().await;
        let _ = load::<Item>().await;
        let _ = load::<Music>().await;
        for (name, rows) in [
            ("Area", Area::table().len()),
            ("Character", Character::table().len()),
            ("Gacha", Gacha::table().len()),
            ("Item", Item::table().len()),
            ("Music", Music::table().len()),
        ] {
            assert!(rows > 0, "{name} loaded empty");
        }
    }

    #[tokio::test]
    async fn load_all_loads_every_table() {
        let n = load_all().await.expect("load_all failed");
        assert!(n >= 200, "load_all loaded only {n} tables");
        assert!(!Card::table().is_empty());
    }

    #[test]
    fn master_get_bin_decodes() {
        use prost::Message;
        assert!(!MASTER_GET_BIN.is_empty(), "MASTER_GET_BIN not init");
        let resp = types::rpc::api::MasterGetResponse::decode(MASTER_GET_BIN).expect("decodes");
        assert!(!resp.version.is_empty());
        assert!(resp.master_tag_packs.len() > 100);
    }
}
