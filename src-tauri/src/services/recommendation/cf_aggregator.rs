//! Módulo de agregação de recomendações baseado em Collaborative Filtering (CF)
//!
//! Carrega um índice CF pré-computado a partir de um arquivo JSON
//! e agrega scores de recomendação baseados na biblioteca do usuário.
//! Esses scores são ponderados pela importância do jogo na biblioteca do usuário.
//! O resultado é um mapa de scores CF por jogo candidato.

use serde::Deserialize;
use std::collections::HashMap;

use once_cell::sync::OnceCell;

use crate::services::recommendation::{calculate_game_weight, GameWithDetails};

// Incluir JSON em compile-time
const COLLABORATIVE_INDEX_JSON: &str = include_str!("../../../data/collaborative_index.json");

// === Estruturas de leitura do JSON ===

/// Estrutura bruta do índice CF lido do JSON
#[derive(Debug, Deserialize)]
struct CollaborativeIndexRaw {
    index: HashMap<String, Vec<Similar>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Similar {
    pub app_id: u32,
    pub score: f32,
}

/// Índice CF final usado pelo backend
/// steam_app_id -> similares
pub type CFIndex = HashMap<u32, Vec<Similar>>;

/// Cache global do índice CF
static CF_INDEX: OnceCell<CFIndex> = OnceCell::new();

// === Inicialização / Cache ===

/// Inicializa o índice CF no startup do app
///
/// Deve ser chamado UMA vez (ex: AppState)
pub fn init_cf_index() -> anyhow::Result<()> {
    let raw: CollaborativeIndexRaw = serde_json::from_str(COLLABORATIVE_INDEX_JSON)?;

    let index: CFIndex = raw
        .index
        .into_iter()
        .filter_map(|(k, v)| k.parse::<u32>().ok().map(|id| (id, v)))
        .collect();

    let total_games = index.len();
    let total_pairs: usize = index.values().map(|v| v.len()).sum();

    CF_INDEX
        .set(index)
        .map_err(|_| anyhow::anyhow!("CF_INDEX já foi inicializado"))?;

    log::info!(
        "[CF] Índice carregado com sucesso | jogos={} pares={} média={:.2}",
        total_games,
        total_pairs,
        total_pairs as f32 / total_games.max(1) as f32
    );

    Ok(())
}

/// Acesso seguro ao índice CF
#[inline]
fn get_cf_index() -> Option<&'static CFIndex> {
    CF_INDEX.get()
}

// === Agregação de candidatos CF ===

/// Agrega candidatos CF a partir da biblioteca do usuário
///
/// Retorna:
/// - scores CF por steam_app_id
/// - métricas para logging
pub fn build_cf_candidates(user_games: &[GameWithDetails]) -> (HashMap<u32, f32>, CFStats) {
    let mut scores: HashMap<u32, f32> = HashMap::new();

    let Some(cf_index) = get_cf_index() else {
        return (scores, CFStats::empty(user_games.len()));
    };

    let mut games_with_steam_id = 0;
    let mut games_with_cf_match = 0;

    for game in user_games {
        let Some(steam_id) = game.steam_app_id else {
            continue;
        };
        games_with_steam_id += 1;

        let Some(similars) = cf_index.get(&steam_id) else {
            continue;
        };
        games_with_cf_match += 1;

        let source_weight = calculate_game_weight(&game.game);

        for similar in similars {
            if similar.score <= 0.0 {
                continue;
            }

            *scores.entry(similar.app_id).or_insert(0.0) += similar.score * source_weight;
        }
    }

    let stats = CFStats {
        total_games: user_games.len(),
        games_with_steam_id,
        games_with_cf_match,
    };

    stats.log();

    (scores, stats)
}

// === Métricas / Logging ===

#[derive(Debug, Clone)]
pub struct CFStats {
    pub total_games: usize,
    pub games_with_steam_id: usize,
    pub games_with_cf_match: usize,
}

/// Métricas de cobertura do CF na biblioteca do usuário
impl CFStats {
    fn empty(total: usize) -> Self {
        Self {
            total_games: total,
            games_with_steam_id: 0,
            games_with_cf_match: 0,
        }
    }

    pub fn log(&self) {
        if self.total_games == 0 {
            log::info!("[CF] Biblioteca vazia");
            return;
        }

        let pct_steam = self.games_with_steam_id as f32 / self.total_games as f32 * 100.0;
        let pct_cf = self.games_with_cf_match as f32 / self.total_games as f32 * 100.0;
        let pct_fallback = 100.0 - pct_cf;

        log::info!(
            "[CF] Jogos na biblioteca={} | SteamID={:.1}% | CF ativo={:.1}% | Fallback CB={:.1}%",
            self.total_games,
            pct_steam,
            pct_cf,
            pct_fallback
        );
    }
}
