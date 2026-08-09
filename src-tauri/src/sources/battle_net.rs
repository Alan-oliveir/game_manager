//! Source para importar jogos instalados via Battle.net (Blizzard/Activision)
//!
//! Detecta jogos instalados lendo `product.db`, o arquivo binário (protobuf) que o próprio
//! Battle.net Agent usa para controlar o que está instalado. Enriquece com `aggregate.json`
//! quando disponível (nome legível, caminho do executável, último jogado).
//!
//! **Por que duas fontes:**
//! - `product.db` é o arquivo que o Battle.net Agent usa, então é a fonte mais confiável. Não tem nome legível do jogo.
//! - `aggregate.json` não é documentado oficialmente, é ligado à integração com o app Xbox ("Play Anywhere") pós-aquisição da Microsoft.
//! - Não há garantia de que `aggregate.json` exista, esteja completo, ou seja mantido no futuro. Por isso é usado para enriquecimento, e não fonte primária.
//!
//! **Observações:**
//! - **Windows:** `C:\ProgramData\Battle.net\Agent\product.db` e `aggregate.json` no mesmo dir.
//! - **Linux:** não suportado por ora (Battle.net não roda de forma confiável via Wine).

use crate::errors::AppError;
use crate::sources::providers::{GameSource, SourceGame};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// === CONSTANTS ===

const BATTLE_NET_AGENT_DIR_WINDOWS: &str = r"C:\ProgramData\Battle.net\Agent";

/// Pseudo-produtos internos do próprio client Battle.net (o Agent e o Desktop App), que aparecem em `product.db` mas não são jogos.
const BATTLE_NET_INTERNAL_PRODUCT_IDS: &[&str] = &["agent", "bna"];

/// Catálogo estático de fallback ProductId -> Nome, extraído do Playnite (fonte aberta).
/// Usado apenas quando `aggregate.json` não existe ou não cobre aquele produto — não é a fonte primária de nome.
const BATTLE_NET_GAMES_CATALOG_JSON: &str = include_str!("../data/battle_net_games.json");

// === STRUCTS ===

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CatalogEntry {
    product_id: String,
    name: String,
}

// === HELPERS ===

static BATTLE_NET_CATALOG: Lazy<HashMap<String, String>> = Lazy::new(
    || match serde_json::from_str::<Vec<CatalogEntry>>(BATTLE_NET_GAMES_CATALOG_JSON) {
        Ok(entries) => entries
            .into_iter()
            .map(|e| (e.product_id.to_lowercase(), e.name))
            .collect(),
        Err(err) => {
            log::warn!("Falha ao parsear catálogo estático battle_net_games.json: {err}");
            HashMap::new()
        }
    },
);

// === AGGREGATE.JSON (enriquecimento opcional) ===

#[derive(Debug, Deserialize)]
struct AggregateFile {
    installed: Vec<AggregateGame>,
}

#[derive(Debug, Deserialize)]
struct AggregateGame {
    name: Option<String>,
    product_id: Option<String>,
    icon_path: Option<String>,
    last_played_timestamp: Option<i64>,
}

// === PRODUCT.DB (protobuf, fonte primária) ===

/// Uma entrada decodificada de `product.db`.
struct ProductDbEntry {
    #[allow(dead_code)]
    internal_id: String,
    product_id: String,
    install_path: Option<String>,
}

/// Source responsável por importar jogos instalados via Battle.net
#[derive(Default)]
pub struct BattleNetSource {}

impl BattleNetSource {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve o diretório do Battle.net Agent (onde ficam `product.db` e `aggregate.json`).
    fn resolve_agent_dir(&self) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let path = PathBuf::from(BATTLE_NET_AGENT_DIR_WINDOWS);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Importa todos os jogos instalados detectados via `product.db`, enriquecidos com `aggregate.json` quando disponível.
    pub async fn import_installed(&self) -> Result<Vec<SourceGame>, AppError> {
        let Some(agent_dir) = self.resolve_agent_dir() else {
            return Ok(vec![]);
        };

        let product_db_path = agent_dir.join("product.db");
        if !product_db_path.exists() {
            return Ok(vec![]);
        }

        let db_bytes = fs::read(&product_db_path)?;
        let entries = parse_product_db(&db_bytes);
        let aggregate_map = Self::load_aggregate_map(&agent_dir.join("aggregate.json"));

        let mut games = Vec::with_capacity(entries.len());

        for entry in entries {
            if BATTLE_NET_INTERNAL_PRODUCT_IDS
                .iter()
                .any(|id| entry.product_id.eq_ignore_ascii_case(id))
            {
                continue;
            }

            let Some(install_path) = entry.install_path.as_ref() else {
                log::warn!(
                    "product.db: entrada '{}' sem install_path, ignorando",
                    entry.product_id
                );
                continue;
            };

            if !Path::new(install_path).is_dir() {
                log::warn!(
                    "product.db: pasta de instalação de '{}' não existe mais ({}), ignorando",
                    entry.product_id,
                    install_path
                );
                continue;
            }

            let enrichment = aggregate_map.get(&entry.product_id.to_lowercase());

            let name = enrichment
                .and_then(|g| g.name.clone())
                .or_else(|| {
                    BATTLE_NET_CATALOG
                        .get(&entry.product_id.to_lowercase())
                        .cloned()
                })
                .unwrap_or_else(|| entry.product_id.clone());

            let executable_path = enrichment.and_then(|g| g.icon_path.clone());

            let last_played = enrichment
                .and_then(|g| g.last_played_timestamp)
                .filter(|&ms| ms > 0)
                .map(|ms| ms / 1000);

            games.push(SourceGame {
                platform: "BattleNet".to_string(),
                platform_game_id: entry.product_id,
                name: Some(name),
                installed: true,
                executable_path,
                install_path: Some(install_path.clone()),
                playtime_minutes: None,
                last_played,
                source_label: None,
            });
        }

        Ok(games)
    }

    /// Carrega `aggregate.json`, indexado por `product_id` em minúsculas.
    ///
    /// Ausência do arquivo (ou erro de parse) não é um erro fatal — só significa importar sem enriquecimento.
    fn load_aggregate_map(path: &Path) -> HashMap<String, AggregateGame> {
        let mut map = HashMap::new();

        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => {
                log::info!("aggregate.json não encontrado, seguindo sem enriquecimento");
                return map;
            }
        };

        let parsed: AggregateFile = match serde_json::from_str(&content) {
            Ok(parsed) => parsed,
            Err(err) => {
                log::warn!("Falha ao parsear aggregate.json: {err}");
                return map;
            }
        };

        for game in parsed.installed {
            if let Some(product_id) = &game.product_id {
                map.insert(product_id.to_lowercase(), game);
            }
        }

        map
    }
}

#[async_trait]
impl GameSource for BattleNetSource {
    async fn fetch_games(&self) -> Result<Vec<SourceGame>, AppError> {
        self.import_installed().await
    }
}

// === PARSER MANUAL DE PROTOBUF PARA PRODUCT.DB ===
//
// Implementação mínima e defensiva: só o suficiente do wire format do protobuf para extrair os três campos.
// Evita puxar uma dependência de protobuf (prost + build.rs) para um caso de uso tão pequeno.

/// Decodifica todas as entradas de `product.db`.
///
/// - O arquivo é uma mensagem raiz com um campo 'repetido de número 1 (`InstalledProductInfo`).
/// - Cada entrada é lida como qualquer campo length-delimited: tag + length (varint) + conteúdo.
/// - Uma tag ou length ilegível interrompe o parse (perda de sincronia com o restante).
/// - Entradas malformadas são puladas com log.
fn parse_product_db(data: &[u8]) -> Vec<ProductDbEntry> {
    let mut entries = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        let Some((field_number, wire_type, tag_end)) = read_tag(data, pos) else {
            log::warn!("product.db: falha ao ler tag em pos {pos}, interrompendo parse");
            break;
        };
        let Some((value, new_pos)) = read_field_value(data, tag_end, wire_type) else {
            log::warn!(
                "product.db: falha ao ler valor do campo {field_number} em pos {pos}, interrompendo parse"
            );
            break;
        };

        // Só o campo 1 (repeated InstalledProductInfo) interessa no nível raiz;
        // Qualquer outro campo já foi consumido/pulado por `read_field_value` acima.
        if field_number == 1 && wire_type == 2 {
            if let Some(bytes) = value {
                match parse_installed_product_info(bytes) {
                    Some(entry) => entries.push(entry),
                    None => log::warn!("product.db: entrada malformada ignorada em pos {pos}"),
                }
            }
        }

        pos = new_pos;
    }

    entries
}

/// Decodifica uma mensagem `InstalledProductInfo`:
///
/// - Campo 1 = InternalId (string)
/// - Campo 2 = ProductId (string)
/// - Campo 3 = Data { campo 1 = Path (string) }
fn parse_installed_product_info(data: &[u8]) -> Option<ProductDbEntry> {
    let mut internal_id: Option<String> = None;
    let mut product_id: Option<String> = None;
    let mut install_path: Option<String> = None;

    let mut pos = 0;
    while pos < data.len() {
        let (field_number, wire_type, tag_end) = read_tag(data, pos)?;
        let (value, new_pos) = read_field_value(data, tag_end, wire_type)?;

        match (field_number, value) {
            (1, Some(bytes)) => internal_id = Some(String::from_utf8_lossy(bytes).into_owned()),
            (2, Some(bytes)) => product_id = Some(String::from_utf8_lossy(bytes).into_owned()),
            (3, Some(bytes)) => install_path = parse_install_data(bytes),
            _ => {} // campo desconhecido: valor já foi "consumido" por read_field_value
        }

        pos = new_pos;
    }

    Some(ProductDbEntry {
        internal_id: internal_id.unwrap_or_default(),
        product_id: product_id?,
        install_path,
    })
}

/// Decodifica a submensagem `Data`: campo 1 = Path (string).
fn parse_install_data(data: &[u8]) -> Option<String> {
    let mut path = None;
    let mut pos = 0;

    while pos < data.len() {
        let (field_number, wire_type, tag_end) = read_tag(data, pos)?;
        let (value, new_pos) = read_field_value(data, tag_end, wire_type)?;

        if field_number == 1 {
            if let Some(bytes) = value {
                path = Some(String::from_utf8_lossy(bytes).into_owned());
            }
        }

        pos = new_pos;
    }

    path
}

/// Lê a tag de um campo protobuf (varint) e devolve (número_do_campo, wire_type, novo_pos).
fn read_tag(data: &[u8], pos: usize) -> Option<(u32, u8, usize)> {
    let (tag, new_pos) = read_varint(data, pos)?;
    let field_number = (tag >> 3) as u32;
    let wire_type = (tag & 0x07) as u8;
    Some((field_number, wire_type, new_pos))
}

/// Lê o valor de um campo de acordo com seu wire type, avançando `pos` corretamente mesmo para campos que serão ignorados.
/// Para wire_type 2 (length-delimited), devolve o slice do valor.
fn read_field_value(data: &[u8], pos: usize, wire_type: u8) -> Option<(Option<&[u8]>, usize)> {
    match wire_type {
        0 => {
            // varint
            let (_, new_pos) = read_varint(data, pos)?;
            Some((None, new_pos))
        }
        1 => {
            // 64-bit fixo
            let new_pos = pos.checked_add(8)?;
            (new_pos <= data.len()).then_some((None, new_pos))
        }
        2 => {
            // length-delimited: string, bytes ou submensagem
            let (len, new_pos) = read_varint(data, pos)?;
            let len = len as usize;
            let end = new_pos.checked_add(len)?;
            (end <= data.len()).then_some((Some(&data[new_pos..end]), end))
        }
        5 => {
            // 32-bit fixo
            let new_pos = pos.checked_add(4)?;
            (new_pos <= data.len()).then_some((None, new_pos))
        }
        _ => None, // wire type desconhecido: não sabemos avançar com segurança
    }
}

/// Lê um varint (LEB128 base 128) a partir de `pos`. Devolve (valor, novo_pos).
fn read_varint(data: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0;

    loop {
        let byte = *data.get(pos)?;
        pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some((result, pos));
        }
        shift += 7;
        if shift >= 64 {
            return None; // varint malformado (excede 64 bits)
        }
    }
}
