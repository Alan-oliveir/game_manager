//! Busca e montagem dos dados técnicos de um jogo no PCGamingWiki.
//!
//! Orquestra 7 queries sequenciais à Cargo API, cobrindo as tabelas:
//! `Infobox_game`, `API`, `Video`, `Input`, `Audio`, `L10n` e `Tags`.
//!
//! O rate limiting é aplicado automaticamente em cada query via `cargo_query`.

use crate::errors::AppError;
use crate::models::{GameExtras, PcgwScrapedData};
use crate::providers::technical::pcgamingwiki::client::{build_http_client, cargo_query};
use crate::providers::technical::pcgamingwiki::parsers::{
    extract_field, normalize_bool_field, parse_l10n_rows,
};
use crate::providers::technical::pcgamingwiki::scraper::scrape_pcgw_page;
use chrono::Utc;
use tracing::info;

/// Busca todos os dados técnicos de um jogo no PCGamingWiki pelo Steam AppID.
///
/// Faz até 8 queries sequenciais (com delay entre elas) cobrindo todas as
/// tabelas Cargo relevantes. Retorna `None` se o jogo não for encontrado na PCGW.
///
/// # Erros
/// - `AppError::NetworkError` — falha de rede (sem internet, timeout, rate limit)
/// - `AppError::ParseError` — resposta inesperada da API
/// - `AppError::NotFound` — jogo não encontrado na PCGW pelo AppID fornecido
pub async fn fetch_pcgw_data(
    steam_app_id: &str,
) -> Result<(GameExtras, Option<PcgwScrapedData>), AppError> {
    let client = build_http_client()?;

    // ------------------------------------------------------------------
    // 1. Infobox_game — resolve page_id, page_name e engine
    // ------------------------------------------------------------------
    let infobox_rows = cargo_query(
        &client,
        "Infobox_game",
        "Infobox_game._pageID=pageID,\
         Infobox_game._pageName=pageName,\
         Infobox_game.Engines=engine, \
         Infobox_game.Available_on=available_on",
        &format!("Infobox_game.Steam_AppID HOLDS \"{}\"", steam_app_id),
        3, // Limit baixo porque esperamos no máximo 1 resultado; 3 é margem para casos inesperados
    )
        .await?;

    if infobox_rows.is_empty() {
        return Err(AppError::NotFound(format!(
            "Jogo com Steam AppID {} não encontrado no PCGamingWiki",
            steam_app_id
        )));
    }

    let page_id = extract_field(&infobox_rows, "pageID").ok_or_else(|| {
        AppError::ParseError("pageID ausente na resposta da Infobox_game".to_string())
    })?;

    let page_name = extract_field(&infobox_rows, "pageName");
    let engine = extract_field(&infobox_rows, "engine");

    info!(
        "PCGW: jogo encontrado — page_id={}, name={:?}",
        page_id, page_name
    );

    // Filtro por page_id usado nas demais queries
    let page_filter = format!("_pageID=\"{}\"", page_id);

    // Busca wikitext (system requirements + game data paths) em paralelo com as
    // queries Cargo restantes — salvo separadamente pelo chamador via save_scraped_data
    let scraped = scrape_pcgw_page(&client, &page_id, steam_app_id).await.ok();

    // ------------------------------------------------------------------
    // 2. OS — suporte a sistemas operacionais
    // ------------------------------------------------------------------
    let api_rows = cargo_query(
        &client,
        "API",
        "API.Direct3D_versions=dx_versions,\
     API.Vulkan_versions=vulkan_versions,\
     API.OpenGL_versions=opengl_versions,\
     API.Windows_64bit_executable=win64,\
     API.Linux_64bit_executable=linux64,\
     API.macOS_ARM_app=macos_arm,\
     API.macOS_Intel_64bit_app=macos_intel64",
        &page_filter,
        3,
    )
        .await?;

    let available_on = extract_field(&infobox_rows, "available_on");
    let dx_versions = extract_field(&api_rows, "dx_versions");
    let vulkan_versions = extract_field(&api_rows, "vulkan_versions");
    let opengl_versions = extract_field(&api_rows, "opengl_versions");
    let win64 = normalize_bool_field(extract_field(&api_rows, "win64"));
    let linux64 = normalize_bool_field(extract_field(&api_rows, "linux64"));
    let macos_arm = normalize_bool_field(extract_field(&api_rows, "macos_arm"));
    let macos_intel64 = normalize_bool_field(extract_field(&api_rows, "macos_intel64"));

    // ------------------------------------------------------------------
    // 3. Video_settings — gráficos, display e acessibilidade visual
    // ------------------------------------------------------------------
    let video_rows = cargo_query(
        &client,
        "Video",
        "Video.Ray_tracing=ray_tracing,\
     Video.Upscaling=upscaling,\
     Video.Frame_gen=frame_gen,\
     Video.Ultrawidescreen=ultrawide,\
     Video.4K_Ultra_HD=four_k,\
     Video.HDR=hdr,\
     Video.120_FPS=high_fps,\
     Video.Field_of_view=fov,\
     Video.Borderless_fullscreen_windowed=borderless,\
     Video.Color_blind=color_blind",
        &page_filter,
        3,
    )
        .await?;

    let ray_tracing = normalize_bool_field(extract_field(&video_rows, "ray_tracing"));
    let upscaling = extract_field(&video_rows, "upscaling");
    let frame_gen = extract_field(&video_rows, "frame_gen");
    let ultrawidescreen = normalize_bool_field(extract_field(&video_rows, "ultrawide"));
    let four_k_support = normalize_bool_field(extract_field(&video_rows, "four_k"));
    let hdr = normalize_bool_field(extract_field(&video_rows, "hdr"));
    let high_fps = normalize_bool_field(extract_field(&video_rows, "high_fps"));
    let fov = normalize_bool_field(extract_field(&video_rows, "fov"));
    let borderless_windowed = normalize_bool_field(extract_field(&video_rows, "borderless"));
    let color_blind = normalize_bool_field(extract_field(&video_rows, "color_blind"));

    // ------------------------------------------------------------------
    // 4. Input — suporte a controles
    // ------------------------------------------------------------------
    let input_rows = cargo_query(
        &client,
        "Input",
        "Input.Controller_support=ctrl_support,\
         Input.Full_controller_support=full_ctrl,\
         Input.Playstation_controller_support=ps_ctrl,\
         Input.XInput_controller_support=xinput",
        &page_filter,
        3,
    )
        .await?;

    let controller_support = normalize_bool_field(extract_field(&input_rows, "ctrl_support"));
    let full_controller = normalize_bool_field(extract_field(&input_rows, "full_ctrl"));
    let playstation_controllers = normalize_bool_field(extract_field(&input_rows, "ps_ctrl"));
    let xinput_controllers = normalize_bool_field(extract_field(&input_rows, "xinput"));

    // ------------------------------------------------------------------
    // 5. Audio_settings — surround sound, legendas e acessibilidade
    // ------------------------------------------------------------------
    let audio_rows = cargo_query(
        &client,
        "Audio",
        "Audio.Surround_sound=surround,\
         Audio.Subtitles=subtitles,\
         Audio.Closed_captions=closed_captions",
        &page_filter,
        3,
    )
        .await?;

    let surround_sound = normalize_bool_field(extract_field(&audio_rows, "surround"));
    let subtitles = normalize_bool_field(extract_field(&audio_rows, "subtitles"));
    let closed_captions = normalize_bool_field(extract_field(&audio_rows, "closed_captions"));

    // ------------------------------------------------------------------------------------------
    // 6. L10n — idiomas suportados - A tabela L10n tem uma linha por idioma - Agrupa por tipo.
    // ------------------------------------------------------------------------------------------
    let l10n_rows = cargo_query(
        &client,
        "L10n",
        "L10n.Language=language,\
         L10n.Interface=interface,\
         L10n.Audio=audio,\
         L10n.Subtitles=subtitles",
        &page_filter,
        50, // Muitos idiomas possíveis
    )
        .await?;

    let (languages_interface, languages_audio, languages_subtitles) = parse_l10n_rows(&l10n_rows);

    // ------------------------------------------------
    // 7. Tags — presença de dados de save e config
    // ------------------------------------------------
    let tags_rows = cargo_query(
        &client,
        "Tags",
        "Tags.Save_data=save_data,\
     Tags.Config_data=config_data",
        &page_filter,
        3,
    )
        .await?;

    let has_save_data = normalize_bool_field(extract_field(&tags_rows, "save_data"));
    let has_config_data = normalize_bool_field(extract_field(&tags_rows, "config_data"));

    // ------------------------------------------------------------------
    // Montagem do struct final
    // ------------------------------------------------------------------
    let fetched_at = Utc::now().to_rfc3339();

    Ok((
        GameExtras {
            steam_app_id: steam_app_id.to_string(),
            pcgw_page_id: Some(page_id),
            pcgw_page_name: page_name,
            engine,
            available_on,
            // API
            dx_versions,
            vulkan_versions,
            opengl_versions,
            win64,
            linux64,
            macos_arm,
            macos_intel64,
            // Video
            ray_tracing,
            upscaling,
            frame_gen,
            ultrawidescreen,
            four_k_support,
            hdr,
            high_fps,
            fov,
            borderless_windowed,
            color_blind,
            // Input
            controller_support,
            full_controller,
            playstation_controllers,
            xinput_controllers,
            // Audio
            surround_sound,
            subtitles,
            closed_captions,
            // Tags
            has_save_data,
            has_config_data,
            // L10n
            languages_interface,
            languages_audio,
            languages_subtitles,
            fetched_at: Some(fetched_at),
        },
        scraped,
    ))
}
