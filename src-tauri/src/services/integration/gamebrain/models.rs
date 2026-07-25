//! Tipos publicos usados pelo frontend e pela integracao.

use serde::{Deserialize, Serialize};

/// Resultado simplificado usado pelo frontend.
/// Estrutura compatível com: Wishlist, Busca, Descoberta e Similares
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBrainSearchResult {
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub rating: Option<f64>,
    pub platforms: Vec<String>,
    pub link: Option<String>,
}

/// Todos os campos são opcionais — use apenas o que precisar.
///
/// # Exemplo
///
/// ```rust
/// # use game_manager_lib::services::integration::gamebrain::{GameBrainSearchParams, GameBrainFilter, GameBrainFilterValue, GameBrainSort, GameBrainSortOrder};
/// let params = GameBrainSearchParams {
///     filters: vec![
///         GameBrainFilter {
///             key: "platform".into(),
///             values: vec![GameBrainFilterValue { value: "pc".into() }],
///             connection: Some("OR".into()),
///         },
///     ],
///     sort: Some(GameBrainSort::Rating),
///     sort_order: Some(GameBrainSortOrder::Desc),
///     limit: Some(20),
///     offset: None,
/// };
/// ```
#[derive(Debug, Default, Clone)]
pub struct GameBrainSearchParams {
    /// Filtros a aplicar na busca. Os valores válidos vêm do campo `filter_options` da resposta da API.
    pub filters: Vec<GameBrainFilter>,
    pub sort: Option<GameBrainSort>,
    /// Direção da ordenação. Padrão da API: descendente.
    pub sort_order: Option<GameBrainSortOrder>,
    /// Número máximo de resultados. Padrão da API: 10.
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Um filtro a ser aplicado na busca.
#[derive(Debug, Clone, Serialize)]
pub struct GameBrainFilter {
    /// Chave do filtro (ex: "platform", "genre", "play_mode").
    pub key: String,
    pub values: Vec<GameBrainFilterValue>,
    /// Operador lógico entre os valores: "OR" ou "AND". A maioria dos filtros usa "OR". Omita para usar o padrão da API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<String>,
}

/// Um valor individual de filtro (ex: "pc", "action", "single_player").
#[derive(Debug, Clone, Serialize)]
pub struct GameBrainFilterValue {
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum GameBrainSort {
    Rating,
    ReleaseDate,
    Price,
}

impl GameBrainSort {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            GameBrainSort::Rating => "computed_rating",
            GameBrainSort::ReleaseDate => "release_date",
            GameBrainSort::Price => "price",
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameBrainSortOrder {
    Asc,
    Desc,
}

impl GameBrainSortOrder {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            GameBrainSortOrder::Asc => "asc",
            GameBrainSortOrder::Desc => "desc",
        }
    }
}

/// Jogo similar retornado para o frontend.
///
/// Subset dos campos disponíveis na resposta de /similar.
/// Inclui screenshots e micro_trailer para a UI da aba Descoberta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarGame {
    /// ID no formato "gamebrain:{id}" para consistência com GameBrainSearchResult.
    pub id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    /// Rating em percentual (0–100), convertido de 0.0–1.0.
    pub rating: Option<f64>,
    pub link: Option<String>,
    /// Primeiros screenshots disponíveis (máx 4 pela API).
    pub screenshots: Vec<String>,
    /// URL do micro-trailer em .webm (Steam CDN), se disponível.
    pub micro_trailer: Option<String>,
    /// Flag de conteúdo adulto — o frontend pode usar para bloquear a exibição automática de imagens.
    pub adult_only: bool,
}

/// Mídia de um jogo retornada para a aba Mídia do GameWindow.
///
/// Separa os vídeos em dois grupos já classificados:
/// - `trailers`: `.webm` do Steam CDN — reprodução direta no `<video>`
/// - `youtube_embeds`: URLs `youtube-nocookie.com` — renderizadas em `<iframe>`
///
/// Essa separação acontece no backend para não vazar lógica de parsing de URL para o frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMedia {
    pub screenshots: Vec<String>,
    /// `.webm` diretos (Steam CDN). Prontos para `<video src=...>`.
    pub trailers: Vec<String>,
    /// Embeds do YouTube (`youtube-nocookie.com/embed/...`). Prontos para `<iframe src=...>`.
    pub youtube_embeds: Vec<String>,
    /// Micro-trailer de loop curto, se disponível.
    pub micro_trailer: Option<String>,
}
