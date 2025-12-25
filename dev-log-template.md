# 🛠️ Diário de Desenvolvimento - Game Manager

Este documento registra a jornada técnica, decisões de arquitetura e desafios superados durante o desenvolvimento do Game Manager.

---

## 📊 Visão Geral do Projeto

**Versão Atual:** 0.1.0 (MVP Desktop)  
**Status:** 🟡 Em Desenvolvimento  
**Início:** 24 de Dezembro de 2025

### Stack Tecnológica
- **Backend:** Tauri v2 + Rust + SQLite (rusqlite)
- **Frontend:** React 19 + TypeScript + Vite
- **Estilização:** Tailwind CSS v4 + Shadcn/UI
- **Ícones:** Lucide React

### Métricas
- **Linhas de código (Rust):** ~200
- **Linhas de código (TypeScript):** ~800
- **Componentes React:** 4
- **Comandos Tauri:** 4
- **Tempo investido:** ~10 horas

---

## 📅 Fase 1: O Nascimento do MVP (Desktop)

**Período:** 24/12/2025 - Em andamento  
**Foco:** Configuração do ambiente, arquitetura híbrida (Rust + React) e persistência de dados local.

### 🚀 1. Setup e Escolhas Tecnológicas

Para garantir performance nativa com uma interface web moderna, a stack escolhida foi:

* **Core:** Tauri v2 (Rust) - *Pelo baixo consumo de RAM e binário pequeno.*
* **Frontend:** React + TypeScript (Vite) - *Pela robustez e tipagem segura.*
* **Estilização:** Tailwind CSS v4 + Shadcn/UI - *Para UI moderna e acessível.*
* **Banco de Dados:** SQLite (via `rusqlite`) - *Embarcado no executável, sem necessidade de instalação externa.*

### 🏗️ 2. Arquitetura do Backend (Rust)

Uma das primeiras decisões importantes foi estruturar o projeto pensando no futuro suporte Mobile.

* **Refatoração `main.rs` vs `lib.rs`:** Em vez de manter toda a lógica no `main.rs` (padrão desktop antigo), migrei a lógica de negócio e comandos para o `lib.rs` usando a macro `#[cfg_attr(mobile, tauri::mobile_entry_point)]`. Isso deixará o porte para Android/iOS muito mais simples na Fase 4.
* **Gerenciamento de Estado:** Implementei uma `struct AppState` protegida por um `Mutex` para garantir que a conexão com o SQLite seja thread-safe entre as chamadas do frontend.
* **Comandos Implementados:**
    * `init_db`: Criação idempotente da tabela `games`.
    * `get_games`: Leitura e mapeamento de SQL para Structs Rust.
    * `add_game`: Inserção de dados.
    * `toggle_favorite`: Toggle booleano direto via SQL para otimização.

### 🎨 3. Frontend e UI/UX

O objetivo era fugir do visual "página web" e criar uma experiência de aplicativo nativo (App-like).

* **Tailwind v4:** Configuração das variáveis CSS para suportar temas Claro/Escuro nativamente com cores `oklch`.
* **Layout Responsivo:** Criação de uma Sidebar fixa e Header flutuante inspirados na Microsoft Store.
* **Componentização:**
    * `Sidebar.tsx`: Navegação lateral com indicador de seção ativa e área de usuário.
    * `Header.tsx`: Barra de busca, botão de adicionar e toggle de tema dark/light.
    * `GameGrid.tsx`: Grid responsivo de cards com hover effects e badges.
    * `App.tsx`: Orquestrador principal que integra todos os componentes.
* **Integração:** Uso de `useEffect` para inicializar o banco de dados silenciosamente ao abrir o app.

### 🐛 4. Desafios e Soluções

#### Problema 1: Compatibilidade Tauri v1 → v2
- **Erro:** Importações antigas `@tauri-apps/api/tauri` não funcionavam
- **Solução:** Migrei para `@tauri-apps/api/core` conforme nova documentação

#### Problema 2: Tailwind CSS v4 - Comando init não existe
- **Erro:** `npx tailwindcss init -p` retornava "could not determine executable to run"
- **Causa:** Tailwind v4 mudou completamente o sistema de configuração
- **Solução:** Instalei `@tailwindcss/vite` e configurei via `@import "tailwindcss"` no CSS

#### Problema 3: Shadcn/UI - Import alias não encontrado
- **Erro:** "No import alias found in your tsconfig.json"
- **Solução:** Adicionei configuração de paths no `tsconfig.json` e alias no `vite.config.ts`

#### Problema 4: tsconfig.json referenciando arquivos inexistentes
- **Erro:** "ENOENT: tsconfig.app.json não encontrado"
- **Solução:** Simplifiquei o `tsconfig.json` removendo as referências a arquivos separados

### ✅ Status Atual

- [x] Comunicação Rust <-> React funcionando (Bridge)
- [x] Banco de dados SQLite criando tabelas e persistindo dados
- [x] Interface base (Sidebar, Grid, Header, Dark Mode) implementada
- [x] Estrutura de componentes modular e reutilizável
- [ ] CRUD completo (falta delete e update)
- [ ] Modal de cadastro com formulário completo
- [ ] Sistema de busca/filtro funcional
- [ ] Integração com API externa (IGDB/RAWG)

### 🔜 Próximos Passos

1. Implementar carregamento real dos jogos do banco (substituir mock data)
2. Criar modal de cadastro com campos completos (cover_url, rating, etc.)
3. Adicionar funcionalidade de deletar jogos
4. Implementar edição de jogos existentes
5. Sistema de busca em tempo real
6. Filtros por gênero, plataforma e favoritos

---

## 📝 Template para Novas Entradas

```markdown
### 📅 [DATA] - [TÍTULO DA SESSÃO]

**Tempo investido:** Xh  
**Objetivo:** Breve descrição do que foi planejado

#### ✨ Implementações
- Item implementado 1
- Item implementado 2

#### 🐛 Problemas Encontrados
**Problema:** Descrição do erro/desafio
- **Causa:** O que causou o problema
- **Solução:** Como foi resolvido
- **Aprendizado:** O que aprendi com isso

#### 💡 Decisões Técnicas
- **Decisão:** O que foi decidido
- **Justificativa:** Por que essa escolha foi feita
- **Alternativas consideradas:** Outras opções que foram analisadas

#### 📚 Recursos Úteis
- [Link útil 1](url)
- [Link útil 2](url)

#### ⏭️ Próxima Sessão
- [ ] Tarefa 1
- [ ] Tarefa 2
```

---

## 🔗 Links Úteis

### Documentação Oficial
- [Tauri v2 Docs](https://tauri.app)
- [React 19 Docs](https://react.dev)
- [Tailwind CSS v4](https://tailwindcss.com/blog/tailwindcss-v4)
- [Shadcn/UI](https://ui.shadcn.com)
- [Rusqlite](https://docs.rs/rusqlite)

### Tutoriais e Referências
- [Tauri + React Setup](https://tauri.app/v1/guides/getting-started/setup/vite)
- [Lucide Icons](https://lucide.dev)
- [IGDB API](https://www.igdb.com/api) - Para integração futura
- [RAWG API](https://rawg.io/apidocs) - Alternativa de API de jogos

### Inspirações de Design
- Microsoft Store (Windows 11)
- Epic Games Launcher
- Hydra

---

## 💡 Lições Aprendidas

### Arquitetura
- Separar lógica em `lib.rs` desde o início economiza refatoração futura
- `Mutex<Connection>` é essencial para thread-safety com SQLite
- Componentização React facilita manutenção e escalabilidade

### Ferramentas
- Tailwind v4 é mais rápido mas tem documentação limitada (ainda novo)
- Shadcn/UI acelera desenvolvimento de UI mas requer configuração cuidadosa

### Desenvolvimento
- Mock data é útil para testar UI antes do backend estar pronto
- Documentar problemas economiza tempo em problemas similares
- TypeScript evita muitos bugs em runtime

### Performance
- SQLite embarcado elimina necessidade de servidor externo
- Tauri gera binários ~10x menores que Electron
- React 19 tem melhorias significativas de performance

---

## 🎯 Roadmap Futuro

### Fase 2: Features Avançadas (Desktop)
- [ ] Integração com IGDB/RAWG API
- [ ] Sistema de tags customizadas
- [ ] Estatísticas e gráficos de playtime
- [ ] Backup/Export da biblioteca
- [ ] Importação de bibliotecas (Steam, Epic, GOG)

### Fase 3: Sincronização Cloud
- [ ] Backend com Rust (Actix-web/Axum)
- [ ] Sistema de autenticação
- [ ] Sincronização entre dispositivos
- [ ] Compartilhamento de bibliotecas

### Fase 4: Mobile (Android/iOS)
- [ ] Adaptação de componentes para mobile
- [ ] Gestos touch otimizados
- [ ] Sincronização offline-first
- [ ] Notificações push

---

*Autor: Alan de Oliveira Gonçalves*  
*Última atualização: 24/12/2025*