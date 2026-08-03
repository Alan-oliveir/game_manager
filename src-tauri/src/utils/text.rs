//! Utilitários para manipulação de Strings em nomes de jogos, séries e tags.

// === CONSTANTS ===

const EDITION_SUFFIXES: &[&str] = &[
    "Collector's Edition",
    "Collectors Edition",
    "Complete Edition",
    "Game of the Year Edition",
    "Definitive Edition",
    "Special Edition",
    "Deluxe Edition",
    "Premium Edition",
    "Enhanced Edition",
    "GOTY Edition",
    "GOTY",
    "Ultimate Edition",
];

// === FUNÇÕES ===

/// Remove símbolos de marca registrada, preservando capitalização e conteúdo.
/// Uso: limpar nome antes de persistir (import), exibição.
pub fn strip_trademark_symbols(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '™' | '®' | '©'))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normaliza para comparação/matching: minúsculo, sem símbolos, sem pontuação de separador.
/// Uso: cache keys, dedup, series matching, comparação com resultado de busca externa.
pub fn normalize_for_matching(name: &str) -> String {
    strip_trademark_symbols(name)
        .to_lowercase()
        .replace([':', ',', '\'', '\u{2019}'], "")
        .trim()
        .to_string()
}

/// Verifica se `needle` aparece em `haystack` com fronteira de palavra nos dois lados
/// (sem letra/dígito colado antes ou depois). Evita falsos positivos como "demo"
/// dentro de "demon's" ou "trial" dentro de "trials".
pub fn contains_word_boundary(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || n.len() > h.len() {
        return false;
    }
    for start in 0..=(h.len() - n.len()) {
        if &h[start..start + n.len()] == n {
            let before_ok = start == 0 || !(h[start - 1] as char).is_alphanumeric();
            let end = start + n.len();
            let after_ok = end == h.len() || !(h[end] as char).is_alphanumeric();
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

/// Heurística por palavra-chave para identificar nomes que provavelmente
/// não são o jogo-base (DLC, edição especial, trilha sonora, demo, pré-venda).
/// Não é definitivo — é um filtro barato para descartar candidatos óbvios
/// antes de aplicar critérios mais confiáveis (match exato, dados estruturados).
pub fn is_likely_non_base_game(name: &str) -> bool {
    let lower = name.to_lowercase();
    if name.trim().len() <= 2 {
        return true;
    }

    // Palavras curtas com risco comprovado de colisão (demon's, trials) — exigem fronteira de palavra nos dois lados.
    let boundary_keywords = ["demo", "trial"];
    if boundary_keywords
        .iter()
        .any(|kw| contains_word_boundary(&lower, kw))
    {
        return true;
    }

    // Demais keywords: substring simples é seguro, pois não há risco de colisão com nomes de jogos conhecidos.
    let substring_keywords = [
        "season pass",
        "dlc",
        "add-on",
        "addon",
        "expansion",
        "pre-order",
        "preorder",
        "pre order",
        "starter pack",
        "pack",
        "soundtrack",
        "artbook",
        "art book",
        "playtest",
        "goodie",
    ];
    substring_keywords.iter().any(|kw| lower.contains(kw))
}

/// Remove sufixo de edição conhecido do final do nome, se houver.
/// Usado como segunda tentativa de match quando o nome completo (com edição) não bate.
pub fn strip_edition_suffix(name: &str) -> String {
    let trimmed = name.trim();
    for suffix in EDITION_SUFFIXES {
        if let Some(rest) = trimmed.to_lowercase().strip_suffix(&suffix.to_lowercase()) {
            return trimmed[..rest.len()]
                .trim_end_matches(['-', ':', ',', ' '])
                .trim()
                .to_string();
        }
    }
    trimmed.to_string()
}

/// Gera um slug normalizado para comparação/dedup: minúsculo, sem símbolos,
/// palavras separadas por hífen. Não usar para exibição — preserva apenas
/// o essencial para matching entre plataformas.
/// Ex: "BioShock Infinite: Complete Edition" -> "bioshock-infinite-complete-edition"
pub fn slugify(name: &str) -> String {
    strip_trademark_symbols(name)
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}
