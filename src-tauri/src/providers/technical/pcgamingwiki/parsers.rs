//! Funções auxiliares de extração e normalização de dados da API do PCGamingWiki.
//!
//! Todos os parsers operam sobre o JSON bruto retornado pela `cargoquery` e
//! são usados exclusivamente por [`crate::providers::technical::pcgamingwiki::fetch`].

use serde_json::Value;

type L10nLists = (
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<Vec<String>>,
);

/// Agrupa os resultados da tabela L10n (uma linha por idioma) em três listas:
/// idiomas com suporte a interface, áudio e legendas.
///
/// Filtra valores negativos ("false", "n/a", "unknown") — só inclui idiomas
/// confirmados ("true") ou com tradução de fã ("hackable").
pub(crate) fn parse_l10n_rows(rows: &[Value]) -> L10nLists {
    let mut interface_langs = Vec::new();
    let mut audio_langs = Vec::new();
    let mut subtitle_langs = Vec::new();

    for row in rows {
        let title = match row.get("title") {
            Some(t) => t,
            None => continue,
        };

        let language = match title.get("language").and_then(|v| v.as_str()) {
            Some(l) if !l.is_empty() => l.to_string(),
            _ => continue,
        };

        let is_supported = |key: &str| -> bool {
            title
                .get(key)
                .and_then(|v| v.as_str())
                .map(|s| s == "true" || s == "hackable")
                .unwrap_or(false)
        };

        if is_supported("interface") {
            interface_langs.push(language.clone());
        }
        if is_supported("audio") {
            audio_langs.push(language.clone());
        }
        if is_supported("subtitles") {
            subtitle_langs.push(language.clone());
        }
    }

    (
        if interface_langs.is_empty() {
            None
        } else {
            Some(interface_langs)
        },
        if audio_langs.is_empty() {
            None
        } else {
            Some(audio_langs)
        },
        if subtitle_langs.is_empty() {
            None
        } else {
            Some(subtitle_langs)
        },
    )
}

/// Extrai um campo `title.<key>` do primeiro resultado de cargoquery.
pub(crate) fn extract_field(rows: &[Value], key: &str) -> Option<String> {
    rows.first()
        .and_then(|row| row.get("title"))
        .and_then(|title| title.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Normaliza valores booleanos da PCGW para string canônica.
///
/// A API retorna strings como "true", "false", "hackable", "unknown", "n/a".
/// Preserva como string para o frontend tratar conforme contexto.
pub(crate) fn normalize_bool_field(value: Option<String>) -> Option<String> {
    value.map(|v| v.to_lowercase()).filter(|v| !v.is_empty())
}
