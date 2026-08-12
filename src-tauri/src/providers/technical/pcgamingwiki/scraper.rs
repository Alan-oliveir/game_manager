//! Parser de wikitext do PCGamingWiki para dados não disponíveis via Cargo API.
//!
//! Extrai dois tipos de dados da wikitext bruta da página via `action=parse`:
//!
//! - **System requirements** — blocos `{{System requirements}}`, um por OS.
//!   Suporta parâmetros padrão (`minCPU`, `recGPU`, etc.), campos duplos
//!   (`minCPU2`, `minGPU2` para alternativas AMD/Intel) e tiers extras
//!   (`alt1Title`/`alt1CPU`/etc., comuns em jogos AAA modernos como Witcher 3).
//!
//! - **Game data paths** — blocos `{{Game data/config|OS|path}}` e `{{Game data/saves|OS|path}}`.
//!
//! # Estratégia de parsing
//!
//! O wikitext não é parseado como árvore — isso seria frágil e lento.
//! Em vez disso, identificamos blocos de template por nome (`{{System requirements`),
//! extraímos o conteúdo até o `}}` de fechamento correspondente (respeitando
//! aninhamento), e dentro do bloco extraímos parâmetros com regex simples
//! `|chave = valor`.
//!
//! # Normalização de valores
//!
//! Todos os valores booleanos da PCGW (`true`, `false`, `unknown`, `n/a`,
//! `hackable`, `limited`, `always on`) são preservados como `Option<String>`
//! para que o frontend decida a semântica de exibição.
//!
//! # Integração com fetch.rs
//!
//! Este módulo é chamado em paralelo com as queries Cargo existentes.
//! A busca do wikitext usa `action=parse&prop=wikitext` e requer uma chamada
//! HTTP adicional — deve respeitar o mesmo rate limiting de 250ms.

use crate::errors::AppError;
use crate::models::{GameDataPath, PcgwScrapedData, SystemRequirements};
use once_cell::sync::Lazy;
use regex::Regex;

/// Captura `|chave = valor` dentro de um bloco de template.
/// Grupos: 1 = chave (sem espaços), 2 = valor (sem espaços nas bordas).
/// O valor é delimitado por `[^|}\n]*` — a própria classe de caracteres já
/// para antes de `|`, `}` ou newline, tornando o lookahead desnecessário
/// (e incompatível com a crate `regex`, que não suporta look-around).
static RE_PARAM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\|([^=|}\n]+?)\s*=\s*([^|}\n]*)").expect("RE_PARAM: regex inválida"));

/// Captura apenas o início de `{{Game data/config|` ou `{{Game data/saves|`.
/// O path completo é extraído manualmente para suportar `{{p|variavel}}` aninhado.
static RE_GAME_DATA_START: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{Game data/(config|saves)\|([^|]+)\|")
        .expect("RE_GAME_DATA_START: regex inválida")
});

/// Extrai o conteúdo do 3º argumento de um template `{{Game data/...}}`,
/// respeitando aninhamento de `{{p|...}}` internos.
///
/// `rest` deve começar logo após o `|` que precede o path.
/// Retorna o path bruto e o índice após o `}}` de fechamento.
fn extract_path_arg(rest: &str) -> Option<&str> {
    let bytes = rest.as_bytes();
    let mut depth = 1usize; // já estamos dentro do template externo
    let mut pos = 0;
    let path_start = 0;

    while pos < bytes.len().saturating_sub(1) {
        if bytes[pos] == b'{' && bytes[pos + 1] == b'{' {
            depth += 1;
            pos += 2;
        } else if bytes[pos] == b'}' && bytes[pos + 1] == b'}' {
            depth -= 1;
            if depth == 0 {
                // pos é onde começa o }} de fechamento do template externo
                return Some(rest[path_start..pos].trim());
            }
            pos += 2;
        } else if bytes[pos] == b'|' && depth == 1 {
            // Pipe no nível do template externo = fim do argumento
            return Some(rest[path_start..pos].trim());
        } else {
            pos += 1;
        }
    }

    None
}

/// Busca o wikitext bruto de uma página do PCGamingWiki pelo `page_id`.
///
/// Usa `action=parse&prop=wikitext` — retorna o wikitext completo da página.
/// Deve ser chamado após a query `Infobox_game` que resolve o `page_id`.
///
/// # Rate limiting
/// Adiciona o delay padrão de 250ms antes da requisição.
pub async fn fetch_wikitext(client: &reqwest::Client, page_id: &str) -> Result<String, AppError> {
    use std::time::Duration;
    use tokio::time::sleep;

    // Mesmo delay das queries Cargo para respeitar rate limit de 30 req/min
    sleep(Duration::from_millis(250)).await;

    let response = client
        .get("https://www.pcgamingwiki.com/w/api.php")
        .query(&[
            ("action", "parse"),
            ("pageid", page_id),
            ("prop", "wikitext"),
            ("format", "json"),
        ])
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::NetworkError(format!(
            "PCGW wikitext API retornou HTTP {}",
            response.status()
        )));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    let wikitext = json
        .pointer("/parse/wikitext/*")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ParseError("Campo wikitext ausente na resposta".to_string()))?
        .to_string();

    Ok(wikitext)
}

/// Encontra todas as ocorrências de `{{NomeDoTemplate` no wikitext e retorna
/// o conteúdo interno de cada bloco (sem as chaves externas), respeitando
/// aninhamento de templates.
///
/// Exemplo: para `{{System requirements\n|OSfamily = Windows\n}}` retorna
/// `"System requirements\n|OSfamily = Windows\n"`.
fn extract_template_blocks<'a>(wikitext: &'a str, template_name: &str) -> Vec<&'a str> {
    let marker = format!("{{{{{}", template_name);
    let mut blocks = Vec::new();
    let mut search_from = 0;

    while let Some(start) = wikitext[search_from..].find(&marker) {
        let abs_start = search_from + start;

        // Encontra o }} de fechamento respeitando aninhamento
        let content_start = abs_start + 2; // após {{
        let mut depth = 1usize;
        let mut pos = content_start;
        let bytes = wikitext.as_bytes();

        while pos < bytes.len().saturating_sub(1) && depth > 0 {
            if bytes[pos] == b'{' && bytes[pos + 1] == b'{' {
                depth += 1;
                pos += 2;
            } else if bytes[pos] == b'}' && bytes[pos + 1] == b'}' {
                depth -= 1;
                if depth == 0 {
                    // Inclui o conteúdo sem as {{ e }}
                    blocks.push(&wikitext[content_start..pos]);
                    search_from = pos + 2;
                    break;
                }
                pos += 2;
            } else {
                pos += 1;
            }
        }

        if depth > 0 {
            // Bloco sem fechamento — para de buscar
            break;
        }
    }

    blocks
}

/// Extrai todos os parâmetros `|chave = valor` de um bloco de template.
/// Retorna um mapa de chave normalizada (lowercase, trim) → valor (trim).
fn parse_params(block: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

    for cap in RE_PARAM.captures_iter(block) {
        let key = cap[1].trim().to_lowercase();
        let value = cap[2].trim().to_string();

        // Ignora linhas de notas e refs — só queremos os valores primários
        if key.ends_with(" notes") || key == "ref" {
            continue;
        }

        // Ignora valores vazios
        if !value.is_empty() {
            map.insert(key, value);
        }
    }

    map
}

/// Retorna `Some(value)` se a chave existe e o valor não é vazio.
fn get_param(params: &std::collections::HashMap<String, String>, key: &str) -> Option<String> {
    params.get(key).cloned()
}

/// Combina dois campos opcionais em uma string "valor1 / valor2" quando ambos
/// existem, ou retorna apenas o primeiro. Usado para `minCPU` + `minCPU2`.
fn combine_fields(primary: Option<String>, secondary: Option<String>) -> Option<String> {
    match (primary, secondary) {
        (Some(a), Some(b)) if !b.is_empty() => Some(format!("{} / {}", a, b)),
        (Some(a), _) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Extrai todos os blocos `{{System requirements}}` do wikitext.
///
/// Retorna um vetor com uma entrada por bloco encontrado. Cada bloco
/// representa um OS (`OSfamily`) e opcionalmente um tier extra (`alt1`, `alt2`).
///
/// # Tiers extras (Witcher 3 style)
///
/// Quando `alt1Title` está presente, o bloco padrão (min/rec) é retornado como
/// primeiro elemento e os tiers extras como elementos adicionais. Isso permite
/// ao frontend exibir "Mínimo / Recomendado / High / Ultra" separadamente.
pub fn parse_system_requirements(wikitext: &str, steam_app_id: &str) -> Vec<SystemRequirements> {
    let blocks = extract_template_blocks(wikitext, "System requirements");
    let mut all_reqs = Vec::new();

    for block in blocks {
        let params = parse_params(block);

        let os_family = get_param(&params, "osfamily").unwrap_or_else(|| "Unknown".to_string());

        // --- Requisitos padrão (min / rec) ---
        let standard = SystemRequirements {
            steam_app_id: steam_app_id.to_string(),
            os_family: os_family.clone(),
            tier_title: None,
            target: get_param(&params, "mintgt"),
            min_os: get_param(&params, "minos"),
            min_cpu: combine_fields(get_param(&params, "mincpu"), get_param(&params, "mincpu2")),
            min_cpu2: None, // já combinado acima
            min_ram: get_param(&params, "minram"),
            min_gpu: combine_fields(get_param(&params, "mingpu"), get_param(&params, "mingpu2")),
            min_gpu2: None,
            min_vram: get_param(&params, "minvram"),
            min_dx: get_param(&params, "mindx"),
            min_storage: get_param(&params, "minhd"),
            rec_os: get_param(&params, "recos"),
            rec_cpu: combine_fields(get_param(&params, "reccpu"), get_param(&params, "reccpu2")),
            rec_cpu2: None,
            rec_ram: get_param(&params, "recram"),
            rec_gpu: combine_fields(get_param(&params, "recgpu"), get_param(&params, "recgpu2")),
            rec_gpu2: None,
            rec_vram: get_param(&params, "recvram"),
            rec_dx: get_param(&params, "recdx"),
            rec_storage: get_param(&params, "rechd"),
        };
        all_reqs.push(standard);

        // --- Tiers extras: alt1, alt2, alt3 (Witcher 3 style) ---
        for n in 1..=3u8 {
            let prefix = format!("alt{}", n);
            let title_key = format!("{}title", prefix);

            // Tier só existe se tiver título
            let tier_title = match get_param(&params, &title_key) {
                Some(t) => t,
                None => break,
            };

            let alt = SystemRequirements {
                steam_app_id: steam_app_id.to_string(),
                os_family: os_family.clone(),
                tier_title: Some(tier_title),
                target: get_param(&params, &format!("{}tgt", prefix)),
                min_os: get_param(&params, &format!("{}os", prefix)),
                min_cpu: combine_fields(
                    get_param(&params, &format!("{}cpu", prefix)),
                    get_param(&params, &format!("{}cpu2", prefix)),
                ),
                min_cpu2: None,
                min_ram: get_param(&params, &format!("{}ram", prefix)),
                min_gpu: combine_fields(
                    get_param(&params, &format!("{}gpu", prefix)),
                    get_param(&params, &format!("{}gpu2", prefix)),
                ),
                min_gpu2: None,
                min_vram: get_param(&params, &format!("{}vram", prefix)),
                min_dx: get_param(&params, &format!("{}dx", prefix)),
                min_storage: get_param(&params, &format!("{}hd", prefix)),
                // Tiers extras só têm mínimo — sem rec separado
                rec_os: None,
                rec_cpu: None,
                rec_cpu2: None,
                rec_ram: None,
                rec_gpu: None,
                rec_gpu2: None,
                rec_vram: None,
                rec_dx: None,
                rec_storage: None,
            };
            all_reqs.push(alt);
        }
    }

    all_reqs
}

/// Extrai todos os caminhos de save e config do wikitext.
///
/// Cada `{{Game data/config|OS|path}}` e `{{Game data/saves|OS|path}}` gera
/// uma entrada. Os caminhos preservam a sintaxe `{{p|variavel}}` intacta.
///
/// OS normalizado: "Windows", "Linux", "OS X" (como aparece na PCGW).
pub fn parse_game_data_paths(wikitext: &str, steam_app_id: &str) -> Vec<GameDataPath> {
    let mut paths = Vec::new();

    for cap in RE_GAME_DATA_START.captures_iter(wikitext) {
        let kind = cap[1].trim().to_string();
        let os = cap[2].trim().to_string();

        // Posição logo após o | que precede o path
        let match_end = cap.get(0).unwrap().end();
        let rest = &wikitext[match_end..];

        let raw_path = match extract_path_arg(rest) {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => continue,
        };

        paths.push(GameDataPath {
            steam_app_id: steam_app_id.to_string(),
            kind,
            os,
            raw_path,
            expanded_path: None,
        });
    }

    paths
}

/// Faz o scraping completo de uma página do PCGamingWiki pelo `page_id`.
///
/// Orquestra:
/// 1. Busca do wikitext via `fetch_wikitext`
/// 2. Parse dos blocos `{{System requirements}}`
/// 3. Parse dos blocos `{{Game data/config}}` e `{{Game data/saves}}`
///
/// Retorna `PcgwScrapedData` com todos os dados extraídos.
/// Em caso de falha de rede, propaga `AppError::NetworkError` para que o
/// chamador trate graciosamente (offline-first: nenhum dado parcial é salvo).
pub async fn scrape_pcgw_page(
    client: &reqwest::Client,
    page_id: &str,
    steam_app_id: &str,
) -> Result<PcgwScrapedData, AppError> {
    let wikitext = fetch_wikitext(client, page_id).await?;

    let system_requirements = parse_system_requirements(&wikitext, steam_app_id);
    let game_data_paths = parse_game_data_paths(&wikitext, steam_app_id);

    tracing::debug!(
        "PCGW scraper page_id={}: {} blocos de sysreq, {} paths de game data",
        page_id,
        system_requirements.len(),
        game_data_paths.len()
    );

    Ok(PcgwScrapedData {
        system_requirements,
        game_data_paths,
    })
}
