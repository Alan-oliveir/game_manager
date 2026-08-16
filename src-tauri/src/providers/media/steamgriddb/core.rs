use super::client::SteamGridDbClient;
use crate::providers::media::steamgriddb::models::SgdbSearchResult;
use crate::utils::text::normalize_for_matching;

const SIMILARITY_THRESHOLD: f64 = 0.92;

pub struct CoverResult {
    pub url: String,
    pub thumb_url: String,
    pub width: u32,
    pub height: u32,
}

/// Encontra o melhor candidato da autocomplete da SteamGridDB para o nome do jogo.
/// A autocomplete da SGDB é "solta" (retorna resultados mesmo pra termos sem
/// relação nenhuma), então aqui SEMPRE exigimos o threshold — não existe atalho
/// de "match exato" sem normalizar e comparar, diferente do fluxo do Nexus.
fn find_best_match<'a>(
    game_name: &str,
    candidates: &'a [SgdbSearchResult],
) -> Option<&'a SgdbSearchResult> {
    let normalized_target = normalize_for_matching(game_name);

    candidates
        .iter()
        .map(|c| {
            let score = strsim::jaro_winkler(&normalized_target, &normalize_for_matching(&c.name));
            (c, score)
        })
        .filter(|(_, score)| *score >= SIMILARITY_THRESHOLD)
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(c, _)| c)
}

/// Resolve a capa de um jogo via SteamGridDB.
/// Retorna None se não achar (caller deve cair pro fallback IGDB/Steam).
pub async fn resolve_cover(
    client: &SteamGridDbClient,
    game_name: &str,
    steam_app_id: Option<u32>,
) -> Result<Option<CoverResult>, String> {
    let grids = if let Some(appid) = steam_app_id {
        // Caminho direto, sem necessidade de fuzzy match
        client.get_grids_by_steam_appid(appid).await?
    } else {
        let candidates = client.search_autocomplete(game_name).await?;

        match find_best_match(game_name, &candidates) {
            Some(matched) => client.get_grids_by_game_id(matched.id).await?,
            None => return Ok(None), // nenhum candidato passou o threshold
        }
    };

    Ok(grids.into_iter().next().map(|g| CoverResult {
        url: g.url,
        thumb_url: g.thumb,
        width: g.width,
        height: g.height,
    }))
}
