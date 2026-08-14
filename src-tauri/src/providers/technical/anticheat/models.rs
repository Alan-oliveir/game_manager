use serde::{Deserialize, Serialize};

/// Estrutura bruta de um jogo no games.json do AWACY (deserialização do fetch).
#[derive(Debug, Deserialize)]
pub struct AwacyGame {
    pub name: String,
    pub slug: String,
    pub status: String,
    pub native: bool,
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub anticheats: Vec<String>,
    #[serde(default)]
    pub store_ids: AwacyStoreIds,
    #[serde(rename = "dateChanged", default)]
    pub date_changed: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AwacyStoreIds {
    pub steam: Option<String>,
    pub epic: Option<AwacyEpicId>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AwacyEpicId {
    pub namespace: String,
    pub slug: String,
}

/// Estrutura retornada ao frontend após lookup no cache local — faltava no
/// código original, era referenciada em `lookup.rs` mas nunca declarada.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnticheatInfo {
    pub name: String,
    pub slug: String,
    pub status: String,
    pub anticheats: Vec<String>,
    pub native: bool,
    pub reference: Option<String>,
    pub date_changed: Option<String>,
}