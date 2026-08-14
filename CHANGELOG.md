# Changelog

All notable changes to this project will be documented in this file.

## [4.4.0] - 2026-08-09

### Added

- Region-aware pricing: the OS system locale (via `sys-locale`) is now used to detect the user's region independently of
  the frontend's UI language, and passed as the `country` parameter to IsThereAnyDeal price requests. Falls back to
  "US" when no region can be detected, and can be manually overridden and persisted in `app_config`.
- Backend/frontend language sync: the frontend now notifies the backend whenever the UI language changes, persisting it
  in `app_config` so background tasks (e.g. AI translation) know the target language without depending on a same-request
  parameter. Falls back to system locale detection if the frontend hasn't set a language yet.
- Structured, per-field AI description translation: `game_descriptions` now stores `summary`, `storyline`,
  `short_description`, and `description` translations independently (replacing the previous single `description_ptbr`
  column), each with a `translated_lang` marker used to invalidate stale translations when the UI language changes.
- Game description screen now displays Summary and Storyline as separate labeled sections when both are available
  (falling back to Steam's short description or the platform's generic description for games without IGDB data),
  translating each section independently and sequentially to respect the Gemini free-tier rate limit.
- AI translation now detects the source language automatically (via structured JSON output from Gemini) instead of
  assuming English input, correctly handling non-English source descriptions (e.g. Japanese or Chinese indie game
  listings) that previously failed silently.
- IGDB integration as the primary metadata source (genres, description, developer/publisher, critic score, cover art,
  alternative names, franchise, game modes, player perspectives, themes, keywords, age ratings, and expansion/DLC
  listings), replacing RAWG after the RAWG API became unreachable.
- New `game_dlcs` table storing IGDB-reported expansions and standalone expansions per game.
- ProtonDB integration: Linux/Steam Deck compatibility tier (Platinum/Gold/Silver/Bronze/Borked) shown in the Extras tab
  when running on Linux, fetched on demand and cached locally. Complements PCGamingWiki's static technical data with
  real-world Proton compatibility reports.
- Automatic post-import metadata enrichment: newly imported games from any platform (Steam, Epic, GOG, Ubisoft,
  Battle.net, EA, Amazon, Xbox, IndieGala, Itch.io, Legacy Games) are now enriched with metadata immediately after
  import, without requiring a separate manual "update metadata" action.
- Shared API rate limiter with per-service concurrency limits and exponential backoff on throttling responses (HTTP
  429/403/502/503/504), applied to RAWG, Steam Store, and IGDB requests, replacing fixed per-call delays.
- Toast notifications for platform imports and metadata enrichment (import started/completed/failed, in progress, and
  completed per platform), routed to native OS notifications when the app window is unfocused.
- Scan sources for the local folder scanner: each scanned root folder is now saved as a named source (defaulting to the
  folder name), letting locally-imported games be tracked, labeled, and managed independently of a one-off scan.
- New `scan_sources` table and a `source_label` column on `games`, propagated through the import pipeline so manually
  scanned games carry their originating folder's label.
- Local Scanner settings screen now includes a section listing all saved scan sources, with rename and delete actions
  (deleting a source can optionally remove its associated games) and a manual refresh control.
- Folder re-scans now flag previously imported games (`alreadyImported`), so the discovery list distinguishes new finds
  from games already in the library instead of re-presenting everything for manual selection.
- Game cards for locally scanned games now display their source label alongside the platform badge.
- Platform grouping view for the Libraries screen: games can now be grouped by platform (Steam, Epic, GOG, etc.) in the
  virtualized grid, with collapsible section headers showing the game count per platform.
- New "Hide not installed" view filter, consolidated into a single dropdown menu alongside the existing "hide
  duplicates" and "hide adult content".
- Nexus Mods integration: a `nexus_games` reference table, refreshed from the Nexus API `games.json` endpoint , is used
  to match library games against Nexus Mods' supported game catalog. Matched games receive a `nexus` entry in
  `external_links` — no per-game API call required, matching is resolved entirely against the cached catalog.
- Integration with HowLongToBeat (HLTB) to automatically fetch detailed completion time metrics (Main Story, Main +
  Extra, Completionist, and Co-op).
- Local playtime tracking for platforms without an official playtime API (Amazon, Battle.net, EA, Epic, GOG, Legacy
  Games, Ubisoft, Xbox, and manually scanned folders): while a game is running, a background process watcher accrues
  playtime in one-minute increments — matching the granularity already reported by Steam, Itch.io, and IndieGala —
  persisting incrementally rather than only at session end.
- New `slug` column on the `games` table, storing a normalized version of each game's name (lowercase, ASCII, trademark
  symbols and punctuation stripped, words joined by hyphens) generated at import time and used for duplicate detection.

### Improved

- `configs.rs`/`system.rs` responsibilities reorganized into a `commands` → `services` → `providers` → `database`
  layering: wishlist price refresh, missing-cover backfill, AI translation orchestration, and region/language detection
  logic were extracted out of Tauri commands into dedicated service modules (`services/wishlist.rs`,
  `services/translation.rs`, `services/locale.rs`), leaving commands as thin wrappers and `database/configs.rs` as a
  generic key-value store.
- Gemini translation requests now disable "thinking" (`thinkingBudget: 0`) and use an explicit 60s timeout, fixing
  request timeouts that occurred when combining structured JSON output with the default reasoning behavior.
- Network error messages from the Gemini and ITAD providers no longer leak the API key embedded in the request URL
  (previously exposed via `reqwest::Error`'s default error formatting).
- Steam library import: name resolution for non-installed games (read from the Steam library cache) now runs
  concurrently instead of one request at a time, reducing import time for libraries with many non-installed titles.
- `steam_app_id` resolution for non-Steam platforms: games imported from other stores are now correlated to their Steam
  counterpart via the public Steam Store Search endpoint, with staged fallback matching (exact name, then
  edition-suffix-stripped name, then best remaining non-DLC candidate) and a confidence level recorded per match.
- Post-import enrichment now commits results in incremental batches instead of a single transaction at the end, limiting
  potential data loss to at most one batch if the app closes or crashes mid-run.
- DLC/demo/edition keyword filtering (used when importing from GOG, Ubisoft, and when correlating Steam IDs) now
  respects word boundaries for short keywords, preventing titles like "Soulstice" or "Trials Fusion" from being
  misclassified as demos or trials.
- `add_game_from_scan` and `add_games_from_scan` now resolve and persist the originating scan source's label
  automatically, falling back to the folder name if the source hasn't been explicitly labeled yet.
- `normalize_for_matching` now also strips straight and typographic apostrophes (`'` / `’`), improving cross-source name
  matching (e.g. Steam Store data using typographic quotes) for series inference, Steam ID resolution and Nexus Mods
  matcher that share this utility.
- Game sidebar interface now displays specific HowLongToBeat time categories instead of a generic estimated playtime.
- New `playtime_source` column on the `games` table, recording whether a game's playtime came from an official platform
  API (Steam, Itch.io, IndieGala) or from local tracking.

### Fixed

- Steam Store Search requests were silently returning empty results due to a missing `User-Agent` header, causing
  `steam_app_id` resolution to fail for every non-Steam game without any visible error.
- Steam library name-resolution requests (for non-installed games) were not covered by the shared Steam rate limiter,
  occasionally triggering temporary throttling from the Steam Store during large imports.
- Duplicate filter ("hide duplicates") no longer fails to group the same game across platforms with differing name
  formatting — e.g. trademark symbols (™, ®), colons, or other punctuation (`BioShock™` vs `BioShock`, `BioShock
  Infinite: Complete Edition` vs `BioShock Infinite Complete Edition`) previously caused an exact-name comparison to
  treat them as different games.

### Removed

- SteamSpy integration (previously used to estimate median playtime): the service has stopped returning
  `median_forever`/`average_forever` data for all games, making it non-functional as a data source.

## [4.3.0] - 2026-07-25

### Added

- Itch.io integration: local library detection of owned games, installed games, and playtime synchronization with
  `butler.db` integrated with the new standardized platform enumerators
- IndieGala integration: installed-game detection via IGClient's `installed.json`, with an optional full-library mode
  reading the account's owned-games list from `config.json` and cross-referencing it with installed titles to reuse
  their complete metadata; game tags are run through the existing RAWG tag-classification pipeline
- Xbox App / Microsoft Store integration: installed-game detection via Gaming Services, locating game folders through
  each drive's `.GamingRoot` marker and reading `MicrosoftGame.config` manifests — covers both first-party and
  third-party titles, with downloadable content and non-standalone add-ons automatically excluded based on manifest
  structure
- Amazon Games integration: full library import via account login (device-registration authentication flow),
  cross-referenced with the Amazon Games App's local database for installed games — installed titles are matched by
  exact product ID and flagged accordingly within the imported library, rather than imported as a separate list
- Epic Games Store full library import via account login (OAuth2 device-registration flow), in addition to the existing
  installed-game detection via local manifest files — library titles are resolved through the Epic catalog API and
  deduplicated by namespace to correctly separate base games from DLC and soundtracks
- EA App integration: installed-game detection via a user-configured install folder, now also recovering previously
  installed (currently uninstalled) titles from EA's local install history — an intentional reflection of how the EA App
  itself retains this information, not a detection gap
- Battle.net integration: installed-game detection via the Agent's `product.db` (parsed as raw protobuf, no external
  dependency), enriched with `aggregate.json` when present for display name, executable path, and last played time, with
  a static fallback catalog (adapted from Playnite) for titles `aggregate.json` doesn't cover — internal Agent/Desktop
  app entries are excluded from the imported list
- GOG integration: full library import via account login (OAuth2 with PKCE) alongside installed-game detection via a
  user-configured games folder, matched against store titles by prefix instead of exact name to account for store-only
  suffixes (e.g. "Deluxe Edition") that don't appear in the installed folder name

### Fixed

- The manual "Add Game" platform dropdown still offered the outdated `"Battle.net"` and `"Itch.io"`
  values, which no longer match the standardized enum; manually added games using those options would have become
  invisible to platform filtering.
- Platform filtering errors resolved by strictly standardizing platform identifiers to `PascalCase` (e.g., `BattleNet`,
  `LegacyGames`, `Itch`) across Rust enums/structs and frontend types, eliminating spaces and special characters that
  broke filter matching
- Rust documentation corrected and updated in `legacy.rs` and `gamebrain/models.rs`

### Improved

- Import command success/empty messages/login success messages standardized across all platform sources via shared
  `format_import_summary`/`format_import_empty`/`format_login_success` helpers in `commands/platforms/core.rs`,
  replacing inconsistent per-platform phrasing with a single consistent format.
- Frontend rendering of platform names now utilizes a centralized `PlatformDisplayNames` dictionary to map internal
  `PascalCase` codes to their proper commercial names (e.g., `Battle.net`, `Itch.io`) across all UI components
- Platform settings screens standardized across all integrations: page titles now show only the platform name,
  descriptions and account-connection labels use consistent imperative phrasing, and the import button includes the
  platform name and is disabled (with an explanatory tooltip) rather than hidden when a required connection or
  credentials are missing, `ImportedItemsBox` now renders its checkmark from the component itself instead of relying on
  it being hardcoded into each translated string, Detected-path listings and icons unified across platforms with a fixed
  local path
- GOG's import button is no longer hidden entirely while logged out; it now stays visible but disabled, with a tooltip
  explaining that an account connection is required — consistent with how other platforms present unmet prerequisites.
- Manually selected paths (EA, GOG, Legacy Games) now persist consistently across sessions through a shared
  `useLocalStoragePlatformPath` hook, replacing per-component `localStorage` handling
- Reorganized platform-name translation keys into a single contiguous block for easier maintenance
- OAuth token storage extended to support provider-specific auxiliary data (e.g. Amazon Games' device serial) without
  affecting previously stored tokens.
- Catalog lookups for the Epic Games library parallelized (up to 8 concurrent requests), reducing full-library import
  time from several minutes to a few seconds on larger libraries

### Removed

- The per-platform icon from the library/favorites game card badge (`StandardGameCard`), which now shows the platform
  name as text only.

## [4.2.0] - 2026-07-13

### Added

- Automatic advance for the Hero carousel on the Home and Trending pages (manual navigation via the arrow buttons
  remains available at any time)

### Fixed

- Hero banner on Home repeatedly canceling in-progress cover image requests whenever the recommendation-based slide
  resolved after other sources — the "highlighted game" was being recalculated on every render instead of computed once
  with a stable, append-only slide order
- Rendered more/fewer hooks violation on the Trending page caused by hooks being called after conditional early `return`
  statements; all hooks now run unconditionally before any early return
- Toggling a favorite in the Library or Favorites pages re-rendering every card in the grid instead of just the one
  affected, caused by the full games array being listed as a dependency of an unrelated playlist callback

### Improved

- Library and Favorites pages now render through a virtualized grid (`react-window` v2), mounting only the cards
  currently visible on screen instead of all of them at once — removes the 300+ms render commits seen on larger
  libraries and keeps performance flat as more platform integrations add more games over time
- Game card components (`StandardGameCard`, `ActionButton`, `GameActionsMenu`, `CachedImage`) memoized with
  `React.memo`, with callbacks stabilized end-to-end across `GameLibraryContext`, `UIContext`, and every page that
  renders game cards
- Duplicated card markup and logic between Library and Favorites consolidated into a shared `LibraryGameCard` component
- `CachedImage` no longer reads `localStorage` on every render (moved to a one-time lazy initializer) and skips its
  async local-cache resolution cycle entirely when the "save covers locally" setting is off

## [4.1.2] - 2026-07-07

### Improved

- Platforms configuration window internals fully refactored: the monolithic `useStoresConfig` hook was split into one
  dedicated hook per platform (Steam, Epic, Heroic, Ubisoft, Legacy Games), now living under `hooks/plataforms/`
  and only loading/persisting state relevant to that platform instead of all five at once on every tab switch
- Shared UI building blocks extracted for the platform settings screens (headers, detected-paths boxes, import progress
  indicators, action buttons/footers, path pickers), reducing duplicated markup across Steam, Epic, Heroic, Ubisoft,
  Legacy Games, and Wine tabs
- External links and auto-detected file/config paths for each platform moved into dedicated constants files instead of
  being hardcoded inline in the components
- Credential inputs on the Steam tab are now disabled while previously saved credentials are loading, preventing a rare
  race condition where in-progress typing could be overwritten once the stored credentials arrived
- Accessible labels (`aria-label`) added to path and credential inputs across all platform settings screens for screen
  reader support

## [4.1.1] - 2026-07-02

### Fixed

- GameBrain similar-games response failing to parse when a game's rating was `null`, causing the "Similar to My Profile"
  section to silently return empty results for affected anchor games
- Missing `becauseOf` field in profile-similar recommendations due to incomplete camelCase serialization, resulting in
  an empty badge/tooltip on similar game cards

## [4.1.0] - 2026-07-02

### Added

- Ko-fi donation button in the app header, always visible for quick access
- "Support Playlite" section in Settings → About, with Ko-fi donation link and alternative ways to support (GitHub
  Sponsors, official website, bug reports)
- Official landing page link in the Quick Settings documentation section
- Custom Ko-fi icon component added to the icons library, consistent with the existing icons

### Improved

- Steam settings help callout redesigned to match the visual language of other info/warning callouts in the app (icon +
  colored title + tinted border box), replacing the previous detached Badge component
- Frontend type definitions standardized to camelCase across four modules (scanner, PCGW, GameBrain, subscriptions),
  with corresponding `#[serde(rename_all = "camelCase")]` added to the matching Rust structs — internal Rust field names
  unchanged

## [4.0.0] - 2026-06-26

### Added

- Game search by characteristics in Wishlist using the GameBrain API
- Media tab in game details window with screenshots, trailers, and videos
- Similar games tab in game details window
- Technical details tab in game details window with system requirements, language support, and controller compatibility
  (data sourced from PCGamingWiki)

### Improved

- Game details window restructured into tabbed navigation for better content organization
- PCGamingWiki integration expanded to surface technical data directly in the UI

## [3.4.0] - 2026-05-24

### Added

- Full internationalization (i18n) support using i18next and react-i18next
- Brazilian Portuguese (pt-BR) and English (en) language support
- Automatic language detection based on operating system locale
- Language selector in Settings → About section
- User language preference persisted across sessions via localStorage
- 11 translation namespaces covering all UI strings: common, settings, library, playlist, trending, wishlist, errors,
  dialog, updater, game_detail, plataforms
- 630 translation keys mapped across all views, components, dialogs, and windows
- Translation documentation and guidelines for users and contributors

### Improved

- Steam review labels now fully translated via i18n instead of hardcoded map
- Error messages in non-React modules migrated to i18n instance
- Confirmation dialog fallback strings internationalized
- Architecture ready for additional languages — only locale JSON files required

## [3.3.1] - 2026-05-11

### Added

- Automated CI/CD release pipeline with GitHub Actions
- Automatic updater artifact generation
- Cryptographic signing for updater packages
- Automated latest.json generation
- Improved release distribution workflow

### Improved

- Release engineering and deployment process
- Cross-platform packaging pipeline
- Update reliability and integrity verification

## [3.1.0] - 2026-02-12

### Added

- **Local Directory Scanner**: New feature that allows monitoring PC folders to automatically add games to the Playlite
  library.
- **Steam Installation Detection**: The Steam importer now identifies which games are already installed on the system,
  marking them correctly in the interface.
- **New UI Structure**: Complete refactoring of interface components, including a new Dialog system and lighter, more
  responsive Tooltips.

### Changed

- **Backend Refactoring (Rust)**: Restructuring of integrations in the Rust core for greater stability and performance
  in IPC (Inter-Process Communication).
- **Asset Optimization**: Improved loading of cover art from the GamerPower API, fixing display failures on unstable
  connections.
- **Import Flow**: The Steam sync process is now more resilient, skipping corrupted entries without interrupting the
  entire task.

### Fixed

- **Cover Art Loading**: Fixed the error that prevented images from displaying correctly in the "Free Games"
  (GamerPower) tab.
- **Tooltip Positioning**: Fine-tuned tooltip coordinate calculation to prevent them from going off-screen on smaller
  resolutions.
- **Import Stability**: Resolved a bug where the local scanner could enter an infinite loop in folders containing
  symbolic links (symlinks).

## [3.0.0] - 2026-02-01

### Added

- **Hybrid Recommendation System (v4.0)**: The algorithm now cross-references your local profile (Content-Based)
  with data obtained from Steam users (Collaborative Filtering) to suggest games.
- **Transparency (XAI)**: Added *Smart Tooltips* on recommendations that explain the reason for the suggestion (e.g.
  "Favorite Series", "Community Trend", "High Tag Affinity").
- **Feedback Loop**: "Not Useful" (Dislike) button on recommendations, allowing the user to train the algorithm by
  ignoring specific games.
- **Automatic Update System**: Full integration with Tauri Updater. The app now checks, downloads and installs updates,
  creating **Automatic Backups** of the database before critical changes (Major Updates).
- **Resilient Offline Mode**: The "Trending", "Upcoming" and "Free Games" pages now work without internet, using a smart
  cache ("Stale-while-revalidate") and displaying an informational banner.
- **Algorithm Settings**: New section in Settings allowing adjustment of weights (Profile vs Community), time penalty
  (Nostalgia) and series prioritization.
- **Hybrid Image Cache**: Option to save cover art locally for offline viewing or save space by using only remote URLs.
- **Giveaways**: GamerPower integration for discovering free games.
- **AI Auto-Translation**: Game description translation using the Gemini API.

### Changed

- **GameDetailModal Refactoring**: Component restructured into smaller files, with performance and UX improvements.
- **Hooks Refactoring**: `useTrending`, `useUpcoming` and `useGiveaways` rewritten to handle network failures and
  transparently serve data from the local cache (`api_cache`).
- **metadata.rs Refactoring**: File split into smaller modules, with improvements in error handling and detailed
  logging.
- **Advanced Wishlist**: The Wishlist now has the option to import lists from Steam and IsThereAnyDeal, in addition to
  monitoring prices and discount coupons.
- **Database Architecture**: Introduction of the `app_config` table for generic system settings (Installation Date,
  Schema Version).
- **Settings Interface**: Replaced standard checkboxes with the visual `ToggleSwitch` component for better UX.
- **Cache Handling**: Optimized differentiated TTL (Time-To-Live) for lists (24h) vs game details (30 days).

### Fixed

- Fixed persistence of settings where certain algorithm weights were not being saved correctly.
- Resolved issue where the "Trending" page showed a fatal error screen upon losing connection; it now degrades
  gracefully to the cache.

## [2.0.0] - 2026-01-12

### Added

- **SQLite Database**: Complete migration of storage to SQLite (`library.db` and `secrets.db`), enabling complex
  relationships between games and details.
- **Recommendation System v2 (Rust)**: New native backend algorithm that calculates affinity based on genre, tags and
  series, applying a time penalty (Age Decay) for games that haven't been played in a long time.
- **IsThereAnyDeal Integration**: The Wishlist now fetches prices from multiple stores, historical lowest price, and
  automatically identifies **Discount Coupons**.
- **HLTB Backend Support**: Implementation of the search service for *HowLongToBeat* in the backend (preparation for
  future UI).
- **Voucher Column**: Added visual support to display coupon codes directly on game cards in the Wishlist.

### Changed

- **Agnostic Architecture**: The system no longer relies exclusively on Steam for metadata, prioritizing the RAWG API
  for cover art and descriptions.
- **Hooks Refactoring**: `useRecommendation` and `useHome` rewritten to consume data processed by Rust, removing heavy
  calculations from JavaScript.
- **Backup System**: Updated to include the new `wishlist` and `game_details` tables in JSON export/import.
- **Detailed Logs**: Improved tracing logs for database operations and HTTP requests.

### Removed

- **Legacy v1 Logic**: Removed old affinity calculation functions from the frontend.
- **Deprecated Fields**: Cleanup of unused fields in configuration interfaces.

## [1.2.0] - 2026-01-06

### Added

- **ConfirmProvider**: Global system of custom confirmation dialogs (replacing the native `window.confirm`).
- Complete Backend (Rust) documentation, generatable via `cargo doc`.
- Visual feedback toasts for all delete and edit operations.
- JSDocs on the main reusable Hooks and Components.

### Changed

- Complete refactoring of the types folder structure (`src/types/`), now split by domain.
- Modernization of the Rust module structure (Rust 2018+ standard).
- Visual standardization of action buttons (Play, Favorite, Menu) using the new `ActionButton` component.
- Responsiveness adjustments in the Details Modal for windows with reduced height.

### Fixed

- "Race Condition" bug on deletion: The delete action was occurring before the user's confirmation. Fixed with proper
  implementation of `async/await` in the confirmation flow.

## [1.1.0] - 2026-01-02

### Added

- Error logging to facilitate debugging and future improvements.
- Button to manually add a game to the wishlist.
- ChangeLog.md for documenting project changes.
- ErrorBoundary component to catch errors in React components and display a friendly message to the user.

### Changed

- UI improvements on the Library page, now with a custom empty state when no games have been imported.
- UI improvements on the Trending page, now with a custom empty state indicating the type of error that occurred
  (connection error, API error, etc.).
- Performance improvements for Steam game metadata import, reducing load time.

### Removed

- CheapShark API integration for game prices, due to instability and prices in USD only.

## [1.0.1] - 2026-01-02

### Added

- Animated loading screen on the Home page with Playlite's visual identity.

### Removed

- Native splashscreen to improve perceived loading speed.

## [1.0.0] - 2026-01-01

### Added

- Initial release of Playlite (Desktop MVP).
- Steam integration for library import.
- Content-based Recommendation System.
- Database Backup and Restore support (JSON).
- Local encryption (AES-256) for credentials.
