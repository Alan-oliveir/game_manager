-- Baseline do schema de games.db no momento da introdução do sistema de migrations.
-- Reúne o que antes era criado em: database/core.rs (create_schema),
-- database/pcgamingwiki.rs, database/nexus.rs, database/cloud_gaming.rs.
--
-- Usa CREATE TABLE IF NOT EXISTS deliberadamente: essa migration é segura de
-- rodar tanto em banco novo quanto em banco já populado (ver database/migrations.rs).
--
-- NUNCA edite este arquivo depois de lançado — mudanças de schema viram m0002, m0003...

-- === GAMES (tabela raiz) ===

CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL DEFAULT '',
    library TEXT NOT NULL,
    source_label TEXT,
    library_game_id TEXT NOT NULL,
    alternative_names TEXT,
    installed BOOLEAN DEFAULT 0,
    import_confidence TEXT,
    install_path TEXT,
    executable_path TEXT,
    launch_args TEXT,
    user_rating INTEGER,
    favorite BOOLEAN DEFAULT 0,
    status TEXT,
    playtime INTEGER,
    playtime_source TEXT,
    last_played TEXT,
    added_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_name ON games(name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_library ON games(library);
CREATE INDEX IF NOT EXISTS idx_favorite ON games(favorite);
CREATE INDEX IF NOT EXISTS idx_status ON games(status);
CREATE INDEX IF NOT EXISTS idx_slug ON games(slug);

-- === TABELAS COM FK PARA games(id) ===

CREATE TABLE IF NOT EXISTS game_details (
    game_id TEXT PRIMARY KEY,
    steam_app_id TEXT,
    developer TEXT,
    publisher TEXT,
    release_date TEXT,
    genres TEXT,
    tags TEXT,
    series TEXT,
    critic_score INTEGER,
    steam_review_label TEXT,
    steam_review_count INTEGER,
    steam_review_score REAL,
    steam_review_updated_at TEXT,
    is_adult BOOLEAN DEFAULT 0,
    adult_tags TEXT,
    external_links TEXT,
    hltb_main_story REAL,
    hltb_main_extra REAL,
    hltb_completionist REAL,
    hltb_coop_time REAL,
    franchise TEXT,
    game_modes TEXT,
    player_perspectives TEXT,
    themes TEXT,
    keywords TEXT,
    age_ratings TEXT,
    display_name TEXT,
    updated_at TEXT,
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS game_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL,
    image_type TEXT NOT NULL CHECK (image_type IN ('cover', 'background')),
    source TEXT NOT NULL CHECK (source IN ('manual', 'steamgriddb', 'igdb', 'steam', 'steam_cdn', 'itch', 'legacy')),
    url TEXT NOT NULL,
    thumb_url TEXT,
    width INTEGER,
    height INTEGER,
    priority INTEGER NOT NULL DEFAULT 0,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(game_id, image_type, source),
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_game_images_lookup ON game_images(game_id, image_type, priority);

-- Controla se já tentamos resolver a capa na SteamGridDB — evita reconsultar a cada enrichment.
-- TTL de 30 dias é checado no lado do covers.rs.
CREATE TABLE IF NOT EXISTS steamgriddb_cache_meta (
    game_id TEXT PRIMARY KEY,
    checked_at TEXT NOT NULL,
    found INTEGER NOT NULL,
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS game_descriptions (
    game_id TEXT PRIMARY KEY,
    summary TEXT,
    storyline TEXT,
    short_description TEXT,
    description TEXT,
    summary_translated TEXT,
    storyline_translated TEXT,
    short_description_translated TEXT,
    description_translated TEXT,
    translated_lang TEXT,
    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- === TABELAS INDEPENDENTES DO DOMÍNIO GAMES ===

CREATE TABLE IF NOT EXISTS wishlist (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cover_url TEXT,
    store_url TEXT,
    store TEXT,
    current_price REAL,
    normal_price REAL,
    lowest_price REAL,
    currency TEXT,
    on_sale BOOLEAN DEFAULT 0,
    voucher TEXT,
    added_at TEXT,
    itad_id TEXT
);

CREATE TABLE IF NOT EXISTS subscriptions (
    service TEXT PRIMARY KEY,
    enabled BOOLEAN DEFAULT 0,
    last_synced TEXT
);

-- game_id/igdb_id sem FK declarada (mantido como estava — DLCs podem referenciar
-- um standalone ainda não importado como jogo próprio).
CREATE TABLE IF NOT EXISTS game_dlcs (
    game_id TEXT NOT NULL,
    igdb_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    slug TEXT,
    cover_image_id TEXT,
    kind TEXT NOT NULL,
    owned INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (game_id, igdb_id)
);

CREATE TABLE IF NOT EXISTS scan_sources (
    id TEXT PRIMARY KEY,
    folder_path TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_scanned_at TEXT
);

CREATE TABLE IF NOT EXISTS anticheat_meta (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    etag TEXT,
    last_fetched TEXT NOT NULL,
    game_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS anticheat_games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    anticheats TEXT NOT NULL,
    steam_id TEXT,
    epic_namespace TEXT,
    epic_slug TEXT,
    native INTEGER NOT NULL DEFAULT 0,
    reference TEXT,
    date_changed TEXT
);

CREATE INDEX IF NOT EXISTS idx_anticheat_slug ON anticheat_games(slug);
CREATE INDEX IF NOT EXISTS idx_anticheat_steam ON anticheat_games(steam_id);

CREATE TABLE IF NOT EXISTS achievements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library TEXT NOT NULL,
    game_id TEXT NOT NULL,
    game_name TEXT NOT NULL,
    achievement_key TEXT NOT NULL,
    achievement_name TEXT NOT NULL,
    achievement_description TEXT,
    unlocked_at INTEGER NOT NULL,
    icon_url TEXT,
    UNIQUE(library, game_id, achievement_key)
);

CREATE INDEX IF NOT EXISTS idx_achievements_unlocked_at ON achievements(unlocked_at DESC);

CREATE TABLE IF NOT EXISTS achievement_sync_state (
    library TEXT NOT NULL,
    game_id TEXT NOT NULL,
    last_synced_at INTEGER NOT NULL,
    has_achievements INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (library, game_id)
);

-- === PCGAMINGWIKI (database/pcgamingwiki.rs) ===

CREATE TABLE IF NOT EXISTS game_extras (
    steam_app_id            TEXT PRIMARY KEY,
    pcgw_page_id            TEXT,
    pcgw_page_name          TEXT,
    engine                  TEXT,
    available_on            TEXT,
    dx_versions             TEXT,
    vulkan_versions         TEXT,
    opengl_versions         TEXT,
    win64                   TEXT,
    linux64                 TEXT,
    macos_arm               TEXT,
    macos_intel64           TEXT,
    ray_tracing             TEXT,
    upscaling               TEXT,
    frame_gen               TEXT,
    ultrawidescreen         TEXT,
    four_k_support          TEXT,
    hdr                     TEXT,
    high_fps                TEXT,
    fov                     TEXT,
    borderless_windowed     TEXT,
    color_blind             TEXT,
    controller_support      TEXT,
    full_controller         TEXT,
    playstation_controllers TEXT,
    xinput_controllers      TEXT,
    surround_sound          TEXT,
    subtitles               TEXT,
    closed_captions         TEXT,
    has_save_data           TEXT,
    has_config_data         TEXT,
    languages_interface     TEXT,
    languages_audio         TEXT,
    languages_subtitles     TEXT,
    fetched_at              TEXT
);

CREATE INDEX IF NOT EXISTS idx_game_extras_fetched_at ON game_extras(fetched_at);

CREATE TABLE IF NOT EXISTS system_requirements (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    steam_app_id TEXT NOT NULL,
    os_family    TEXT NOT NULL,
    tier_title   TEXT,
    target       TEXT,
    min_os       TEXT,
    min_cpu      TEXT,
    min_cpu2     TEXT,
    min_ram      TEXT,
    min_gpu      TEXT,
    min_gpu2     TEXT,
    min_vram     TEXT,
    min_dx       TEXT,
    min_storage  TEXT,
    rec_os       TEXT,
    rec_cpu      TEXT,
    rec_cpu2     TEXT,
    rec_ram      TEXT,
    rec_gpu      TEXT,
    rec_gpu2     TEXT,
    rec_vram     TEXT,
    rec_dx       TEXT,
    rec_storage  TEXT,
    fetched_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sysreq_app_id ON system_requirements(steam_app_id);

CREATE TABLE IF NOT EXISTS game_data_paths (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    steam_app_id TEXT NOT NULL,
    kind         TEXT NOT NULL,
    os           TEXT NOT NULL,
    raw_path     TEXT NOT NULL,
    fetched_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_gamedata_app_id ON game_data_paths(steam_app_id);

-- === NEXUS MODS (database/nexus.rs) ===

CREATE TABLE IF NOT EXISTS nexus_games (
    domain_name TEXT PRIMARY KEY,
    nexus_id    INTEGER NOT NULL,
    name        TEXT NOT NULL,
    genre       TEXT,
    approved_date INTEGER
);

CREATE INDEX IF NOT EXISTS idx_nexus_games_name ON nexus_games(name);

CREATE TABLE IF NOT EXISTS nexus_games_cache_meta (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    fetched_at INTEGER NOT NULL
);

-- === CLOUD GAMING — GEFORCE NOW + XBOX CLOUD (database/cloud_gaming.rs) ===

CREATE TABLE IF NOT EXISTS gfn_games (
    steam_app_id TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    store        TEXT NOT NULL,
    status       TEXT
);

CREATE TABLE IF NOT EXISTS gfn_games_meta (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    fetched_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS xbox_cloud_ids (
    store_id TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS xbox_cloud_meta (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    fetched_at INTEGER NOT NULL
);
