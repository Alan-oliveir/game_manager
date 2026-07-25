# 🗺️ Playlite Roadmap

This document outlines the next steps for Playlite based on the project's current state.
The priority remains maintaining the local-first philosophy as the app evolves from a manager into a more complete
library hub.

## ✅ Consolidated Base

The following milestones are already implemented in the code and the project's recent history:

- Local library with SQLite and backup/restore.
- Steam integration and local directory scanner.
- Hybrid recommendation system.
- Cache and graceful degradation for online content.
- Auto-updates, code signing, and release pipeline.
- More modular UI and backend structure.

## 🎯 Next Focus: Enriching Metadata and Stabilizing the Catalog Layer

The goal of this phase is to improve the visual and informative quality of games without abandoning local execution.

- [x] **More efficient image caching**: Improve local storage, expiration, and cover reuse to reduce network calls.
- [x] **Metadata normalization**: Unify enrichment rules, fallbacks, and prioritization across sources.

## 🕹️ Game Hub Expansion: Other Stores and Local Execution

After consolidating the catalog layer, the next evolution is to expand the app's ability to discover and track games
installed on other platforms.

- [x] **Epic Games Store**: Local import via manifests and/or detected installations.
- [x] **Ubisoft Connect**: Local reading of installed games and library integration.
- [x] **Origin / EA Desktop**: Local reading of installed games and library integration.
- [x] **GOG Galaxy / GOG**: Local reading of installed games and library integration.
- [x] **Local sync improvements**: Reduce inconsistencies between imports, manual edits, and data enrichment.

## ☁️ Optional Future Synchronization

Cloud synchronization remains in the long-term plan, but only after integration with other stores and the stabilization
of the local catalog.

- [ ] **Cloud Sync (Supabase)**: Synchronize library, preferences, and settings as an optional/opt-in feature.
- [ ] **Conflict resolution**: Define clear rules for local data versus synchronized data.
- [ ] **Security and privacy**: Keep the local-first model as the default, with cloud only when the user opts in.

## 📦 Platform and Distribution

With the functional base consolidated, the distribution focus can move toward broader support.

- [ ] **Robust Linux build**: Improve the experience on Debian/Ubuntu and Flatpak, with special attention to the Steam
  Deck.
- [ ] **Desktop integration adjustments**: Validate icons, launchers, and behavior in Wayland/X11 environments.

## 📱 Satellite Projects

Parallel explorations that do not block the evolution of the main app.

- [ ] **Playlite Companion (Android)**: A separate app for library browsing and remote tracking.

## 📝 Note

This roadmap is intentionally flexible. As new features are stabilized, items should be reordered to reflect the actual
priorities of the project.
