# 🎮 Game Manager

![Status](https://img.shields.io/badge/status-em%20desenvolvimento-yellow)
![License](https://img.shields.io/badge/license-MIT-green)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-24C8DB?logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-20232A?logo=react&logoColor=61DAFB)
![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?logo=typescript&logoColor=white)

Gerenciador de biblioteca de jogos desktop (local-first) com sistema inteligente de recomendação baseado em Machine Learning clássico.

## 💡 Motivação

Tenho uma biblioteca grande de jogos e frequentemente fico em dúvida sobre qual jogar depois.
Este projeto nasceu para resolver esse problema real, ao mesmo tempo em que serve como um projeto completo de portfólio para explorar Rust, Tauri, React e sistemas de recomendação.

## ✨ Funcionalidades

- Gerenciamento completo de biblioteca de jogos (CRUD)
- Persistência local com SQLite (offline-first)
- Interface desktop inspirada na Microsoft Store
- Sistema de favoritos, avaliações e tempo de jogo
- Base para sistema de recomendação inteligente

## 🛠️ Stack

- **Desktop:** Tauri v2 + Rust
- **Frontend:** React + TypeScript + Vite
- **UI:** Tailwind CSS v4 + Shadcn/UI
- **Database:** SQLite (local)

## 🧱 Arquitetura (High-level)

- Aplicação desktop local-first
- Core de negócio em Rust
- UI desacoplada em React
- Comunicação via Tauri Commands
- Banco SQLite embarcado
  
➡️ Veja mais em [`docs/architecture.md`](docs/architecture.md)

## 📚 Documentação

- Arquitetura técnica: [`docs/architecture.md`](docs/architecture.md)
- Decisões arquiteturais (ADR): [`ADR.md`](ADR.md)
- Diário de desenvolvimento: [`DEV_LOG.md`](DEV_LOG.md)

## 🤖 Sistema de Recomendação

O sistema de recomendação é baseado em **Machine Learning clássico**, priorizando performance, privacidade e funcionamento offline.

- Content-based filtering
- Similaridade entre jogos
- Regras de negócio
- LLM opcional apenas para explicação das sugestões

📄 Detalhes técnicos em: [`docs/recommendation-system.md`](docs/recommendation-system.md)

## 🚀 Como rodar localmente

### Requisitos

- Node.js 18+
- Rust (rustup)
- npm ou pnpm

### Desenvolvimento

```bash
npm install
npm run tauri dev
```

## 🗺️ Roadmap

- [x] CRUD local de jogos
- [x] UI desktop base
- [x] Integração com Steam
- [ ] Sync opcional em nuvem
- [ ] Sistema de recomendação (ML clássico)

## 🤝 Contribuição

Sugestões e contribuições são bem-vindas!
Veja o arquivo [`CONTRIBUTING.md`](CONTRIBUTING.md).

## 📄 Licença

MIT
