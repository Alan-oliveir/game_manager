# Game Manager - Proposta de Aplicação

## 📋 Visão Geral

**Game Manager** é um gerenciador de biblioteca de jogos multiplataforma (Desktop e Mobile) com sugestões inteligentes baseadas em IA, que permite aos usuários organizar sua coleção de jogos e receber recomendações personalizadas com base em seu histórico de jogos favoritos.

### Objetivos do Projeto
- Criar uma aplicação completa para aprendizado de tecnologias modernas
- Desenvolver portfólio profissional demonstrando habilidades técnicas variadas
- Implementar solução prática que resolve problema real de organização de biblioteca de jogos
- Explorar integração com IA de forma útil e não superficial

---

## 🛠️ Stack Tecnológica

### Desktop (Aplicação Principal)

**Frontend**
- **Framework**: React 18+ com TypeScript
- **UI Library**: Shadcn/ui + Tailwind CSS
- **Inspiração de Design**: Microsoft Store (Windows)
- **Ícones**: Lucide Icons
- **State Management**: React Hooks (useState, useContext)

**Backend**
- **Framework**: Tauri 1.5+
- **Linguagem**: Rust
- **Database**: SQLite (via Tauri)
- **Build**: Tauri Bundler (~3-10MB executável)

**Inteligência Artificial**
- **Local (Desktop)**: Ollama + Llama 3.1 / Phi-3
- **Cloud (Fallback/Mobile)**: Groq API (gratuita)
- **Estratégia**: Híbrida - detecta disponibilidade e permite escolha do usuário

### Mobile (Versão Futura)

**Framework**: React Native + Expo
- Compartilha 70-80% do código com versão desktop
- UI adaptada para mobile (componentes nativos)
- Sincronização opcional com versão desktop

### Sincronização (Opcional)

**Backend de Sync**
- **Plataforma**: Supabase (PostgreSQL + Auth + Real-time)
- **Autenticação**: Email/senha + OAuth (Google, GitHub)
- **Tier Gratuito**: 500MB storage + 2GB bandwidth/mês
- **Segurança**: Row Level Security (RLS)

---

## ✨ Funcionalidades

### MVP (Fase 1) - Desktop Local

#### Gerenciamento de Jogos (CRUD)
- ✅ Adicionar jogos manualmente
  - Nome, gênero, plataforma
  - Upload de capa personalizada
  - Tags/categorias customizadas
- ✅ Editar informações dos jogos
- ✅ Remover jogos da biblioteca
- ✅ Marcar jogos como favoritos
- ✅ Registrar tempo de jogo (manual)
- ✅ Avaliar jogos (1-5 estrelas)

#### Interface do Usuário
- Layout tipo Microsoft Store
  - Sidebar de navegação
  - Grid responsivo de cards
  - Header com busca
- Seções:
  - 🎮 Biblioteca (todos os jogos)
  - ⭐ Favoritos
  - 🤖 Sugestões IA
  - ⚙️ Configurações
- Busca e filtros
  - Por nome, gênero, plataforma
  - Ordenação (nome, tempo jogado, avaliação)

#### Sugestões com IA (Ollama Local)
- Análise do histórico do usuário
  - Jogos favoritos
  - Jogos mais jogados
  - Avaliações
- Prompt contextual para LLM
- Sugestões de 3-5 jogos similares
- Explicação do motivo de cada sugestão
- Detecção automática de Ollama instalado
- UX guiada para instalação se necessário

#### Persistência de Dados
- SQLite local (sempre funciona offline)
- Backup/restauração de dados
- Export para JSON

### Fase 2 - Integrações com Lojas

#### Steam Integration
- Importação via Steam API (oficial e gratuita)
- Dados importados:
  - Lista completa de jogos
  - Tempo de jogo
  - Capas oficiais
  - ID da Steam (para links)
- Requisitos:
  - Perfil Steam público
  - Steam ID ou username
- Sincronização manual (botão "Sync")

#### Epic Games Store
- Scan de jogos instalados localmente
- Leitura de manifests (arquivos .item)
- Detecção automática se Epic está instalado
- Limitação: apenas jogos instalados

#### GOG Galaxy
- Leitura de banco SQLite local
- Detecção de jogos instalados
- Similar ao Epic

### Fase 3 - Sincronização Multiplataforma

#### Arquitetura Local-First
- **Modo 1: Local Apenas** (padrão)
  - Funciona 100% offline
  - Dados apenas no dispositivo
  - Sem necessidade de conta
  - Máxima privacidade

- **Modo 2: Local + Sync** (opcional)
  - Tudo do Modo 1
  - Sincronização entre dispositivos
  - Backup automático na nuvem
  - Requer conta (email ou OAuth)

#### Sistema de Sincronização
- Upload inicial de dados locais
- Sync incremental (apenas mudanças)
- Resolução de conflitos (Last-Write-Wins ou merge inteligente)
- Status visual de sincronização
- Opção de desativar sync mantendo dados locais

#### Controle de Privacidade
- Usuário escolhe o que sincronizar:
  - Biblioteca de jogos
  - Tempo de jogo
  - Avaliações
  - Favoritos
- Dados sempre criptografados em trânsito
- Sem coleta de dados desnecessários

### Fase 4 - Versão Mobile

#### React Native App
- Interface adaptada para touch
- Mesma lógica de negócio (código compartilhado)
- Sempre usa IA em nuvem (Groq API)
- Sincronização automática com desktop (se habilitado)

#### Funcionalidades Mobile
- Visualizar biblioteca
- Buscar jogos
- Marcar favoritos
- Receber sugestões IA
- Adicionar jogos manualmente
- Sincronizar com desktop

---

## 🔒 Segurança e Privacidade

### Princípios
1. **Local-First**: Aplicação funciona completamente offline
2. **Dados Mínimos**: Coletamos apenas o necessário
3. **Transparência Total**: Usuário sabe exatamente o que é armazenado
4. **Controle do Usuário**: Pode desativar qualquer funcionalidade
5. **Open Source**: Código auditável

### O Que NÃO Coletamos
- ❌ Senhas de lojas (Steam, Epic, etc)
- ❌ Tokens de autenticação sensíveis
- ❌ Dados de pagamento
- ❌ Histórico de compras
- ❌ Lista de amigos
- ❌ Mensagens ou chat

### O Que Coletamos (Apenas Local ou Opt-in)
- ✅ Lista de jogos (local)
- ✅ Tempo de jogo (local, opcional sync)
- ✅ Avaliações pessoais (local, opcional sync)
- ✅ Preferências de UI (local)

### Integrações com Lojas
- Steam: Usa apenas Steam ID público (API oficial)
- Epic/GOG: Lê apenas arquivos locais públicos
- Sem armazenamento de credenciais
- Sem acesso a dados sensíveis

### IA Local (Ollama)
- Processamento 100% no dispositivo
- Dados nunca saem do computador
- Sem telemetria
- Modelos open-source

### IA em Nuvem (Groq - Mobile)
- Usado apenas quando necessário (mobile/fallback)
- Apenas texto da lista de jogos enviado
- Sem identificadores pessoais
- API respeitando privacidade (sem treinamento com dados)

---

## 📊 Arquitetura do Sistema

### Estrutura de Diretórios

```
game-manager/
├── packages/
│   ├── shared/                    # Código compartilhado
│   │   ├── hooks/                # React hooks reutilizáveis
│   │   │   ├── useGames.ts
│   │   │   ├── useAI.ts
│   │   │   └── useSync.ts
│   │   ├── services/             # Lógica de negócio
│   │   │   ├── ai-service.ts
│   │   │   ├── storage-service.ts
│   │   │   ├── steam-api.ts
│   │   │   └── sync-service.ts
│   │   ├── types/                # TypeScript types
│   │   │   ├── game.ts
│   │   │   ├── user.ts
│   │   │   └── integration.ts
│   │   └── utils/                # Funções auxiliares
│   │       └── helpers.ts
│   │
│   ├── desktop/                   # App Tauri
│   │   ├── src/                  # Frontend React
│   │   │   ├── components/
│   │   │   │   ├── ui/          # Shadcn components
│   │   │   │   ├── GameCard.tsx
│   │   │   │   ├── GameGrid.tsx
│   │   │   │   ├── Sidebar.tsx
│   │   │   │   ├── AISuggestions.tsx
│   │   │   │   └── IntegrationManager.tsx
│   │   │   ├── pages/
│   │   │   │   ├── Library.tsx
│   │   │   │   ├── Favorites.tsx
│   │   │   │   ├── AIPage.tsx
│   │   │   │   └── Settings.tsx
│   │   │   ├── lib/
│   │   │   │   └── tauri.ts
│   │   │   └── App.tsx
│   │   │
│   │   ├── src-tauri/            # Backend Rust
│   │   │   ├── src/
│   │   │   │   ├── main.rs
│   │   │   │   ├── commands/
│   │   │   │   │   ├── games.rs
│   │   │   │   │   ├── ollama.rs
│   │   │   │   │   └── integrations.rs
│   │   │   │   └── db/
│   │   │   │       ├── mod.rs
│   │   │   │       └── schema.rs
│   │   │   └── Cargo.toml
│   │   │
│   │   ├── package.json
│   │   └── tauri.conf.json
│   │
│   └── mobile/                    # React Native (Fase 4)
│       ├── src/
│       │   ├── components/
│       │   ├── screens/
│       │   └── navigation/
│       ├── app.json
│       └── package.json
│
├── docs/                          # Documentação
│   ├── API.md
│   ├── SETUP.md
│   └── PRIVACY.md
│
└── README.md
```

### Fluxo de Dados

```
┌─────────────────────────────────────────────────┐
│                  FRONTEND                       │
│  React Components + Shadcn UI + Tailwind       │
└─────────────────┬───────────────────────────────┘
                  │ invoke()
                  ↓
┌─────────────────────────────────────────────────┐
│              TAURI COMMANDS                     │
│  Rust functions expostas ao frontend           │
└─────────────────┬───────────────────────────────┘
                  │
        ┌─────────┴──────────┬──────────────────┐
        ↓                    ↓                  ↓
┌───────────────┐   ┌────────────────┐   ┌─────────────┐
│   SQLITE      │   │   OLLAMA       │   │  STEAM API  │
│   (Local DB)  │   │   (HTTP call)  │   │  (HTTP)     │
└───────────────┘   └────────────────┘   └─────────────┘
        │                    │                  │
        └────────────────────┴──────────────────┘
                             │
                    (Opcional: Sync)
                             ↓
                    ┌────────────────┐
                    │   SUPABASE     │
                    │   PostgreSQL   │
                    └────────────────┘
```

---

## 🗄️ Modelo de Dados

### Tabela: games

```sql
CREATE TABLE games (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  genre TEXT,
  platform TEXT,  -- 'steam', 'epic', 'gog', 'manual'
  cover_url TEXT,
  playtime INTEGER DEFAULT 0,  -- em minutos
  rating INTEGER,  -- 1-5 estrelas
  favorite BOOLEAN DEFAULT FALSE,
  notes TEXT,
  
  -- Integrações
  steam_app_id INTEGER,
  epic_app_name TEXT,
  gog_id TEXT,
  
  -- Metadados
  imported BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  last_synced_at TIMESTAMP
);
```

### Tabela: integrations

```sql
CREATE TABLE integrations (
  id TEXT PRIMARY KEY,
  platform TEXT NOT NULL,  -- 'steam', 'epic', 'gog'
  user_id TEXT,  -- Steam ID, etc
  enabled BOOLEAN DEFAULT TRUE,
  last_sync TIMESTAMP,
  settings TEXT  -- JSON com configurações
);
```

### Tabela: ai_suggestions

```sql
CREATE TABLE ai_suggestions (
  id TEXT PRIMARY KEY,
  suggested_game TEXT NOT NULL,
  reasoning TEXT,
  based_on TEXT,  -- JSON array de game IDs
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  user_action TEXT  -- 'accepted', 'rejected', 'pending'
);
```

### Tabela: sync_queue (para sincronização)

```sql
CREATE TABLE sync_queue (
  id TEXT PRIMARY KEY,
  table_name TEXT NOT NULL,
  record_id TEXT NOT NULL,
  operation TEXT NOT NULL,  -- 'create', 'update', 'delete'
  data TEXT,  -- JSON
  synced BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## 🎯 Roadmap de Desenvolvimento

### Fase 1: MVP Desktop Local (4-6 semanas)

**Semana 1-2: Setup e Infraestrutura**
- [ ] Configurar projeto Tauri + React + TypeScript
- [ ] Instalar e configurar Shadcn/ui + Tailwind
- [ ] Setup SQLite com Tauri
- [ ] Criar estrutura de pastas
- [ ] Configurar Rust environment

**Semana 3-4: Funcionalidades Core**
- [ ] Implementar CRUD de jogos
- [ ] Interface básica (Sidebar + Grid)
- [ ] Sistema de busca e filtros
- [ ] Favoritos e avaliações
- [ ] Persistência SQLite

**Semana 5-6: IA Local**
- [ ] Integração com Ollama
- [ ] Detecção de Ollama instalado
- [ ] Sistema de sugestões
- [ ] UX para instalação do Ollama
- [ ] Prompts otimizados

### Fase 2: Integrações (2-3 semanas)

**Semana 7-8: Steam Integration**
- [ ] Setup Steam API key
- [ ] Implementar import de jogos
- [ ] Buscar capas oficiais
- [ ] Sincronização manual
- [ ] UX de conexão

**Semana 9: Epic/GOG**
- [ ] Scan de arquivos locais Epic
- [ ] Leitura de banco GOG
- [ ] Detecção automática
- [ ] Interface de gerenciamento

### Fase 3: Polish e Melhorias (2 semanas)

**Semana 10-11:**
- [ ] UI refinada (inspiração MS Store)
- [ ] Animações e transições
- [ ] Export/Import dados
- [ ] Estatísticas da biblioteca
- [ ] Tema claro/escuro
- [ ] Testes básicos

### Fase 4: Sincronização (2-3 semanas)

**Semana 12-13:**
- [ ] Setup Supabase
- [ ] Implementar autenticação
- [ ] Sistema de sync
- [ ] Resolução de conflitos
- [ ] UX de status de sync

**Semana 14:**
- [ ] Testes de sincronização
- [ ] Documentação
- [ ] Polimento final

### Fase 5: Mobile (3-4 semanas)

**Semana 15-16:**
- [ ] Setup React Native + Expo
- [ ] Adaptar componentes para mobile
- [ ] Navigation
- [ ] Integração Groq API

**Semana 17-18:**
- [ ] Sincronização mobile ↔ desktop
- [ ] Testes em dispositivos
- [ ] Build Android/iOS
- [ ] Publicação (opcional)

---

## 🚀 Build e Deploy

### Desktop

**Desenvolvimento**
```bash
npm run tauri dev
```

**Build de Produção**
```bash
npm run tauri build
```

**Outputs:**
- Windows: `.exe` (~5-10MB) + `.msi` installer
- Portable: executável único sem instalação

### Mobile (Futuro)

**Android**
```bash
npx expo build:android
```

**iOS**
```bash
npx expo build:ios
```

---

## 📚 Recursos e APIs

### APIs Externas

**Steam Web API**
- Endpoint: `https://api.steampowered.com`
- Documentação: https://steamcommunity.com/dev
- Rate Limit: Generoso (100k requests/dia)
- Custo: Gratuito

**Groq API**
- Endpoint: `https://api.groq.com`
- Modelos: Llama 3.1, Mixtral
- Rate Limit: 6.000 tokens/minuto (tier gratuito)
- Custo: Gratuito até limite, depois ~$0.27/1M tokens

**Supabase**
- Auth + Database + Real-time
- Tier Gratuito: 500MB storage, 2GB bandwidth/mês
- Custo: Gratuito para MVP, depois $25/mês

### Ferramentas de Desenvolvimento

**Obrigatórias:**
- Node.js 18+
- Rust (via rustup)
- Visual Studio Build Tools (Windows)
- Git

**Recomendadas:**
- VS Code
- Extensões: Rust Analyzer, Tailwind CSS IntelliSense, ES7+ React snippets

**Para Testes:**
- Ollama (instalado localmente)
- Modelos: llama3.1 (~4GB) ou phi3 (~2GB)

---

## 💡 Diferenciais do Projeto

### Técnicos
1. **Arquitetura híbrida local/cloud** - demonstra pensamento sobre trade-offs
2. **Multiplataforma desde o design** - código compartilhado
3. **Local-first** - privacidade e performance
4. **IA útil e não superficial** - resolve problema real
5. **Rust + React** - stack moderna e performática

### UX
1. **Zero fricção para começar** - funciona offline sem conta
2. **Sync opcional** - usuário no controle
3. **Integrações automáticas** - Steam API + scan local
4. **Design inspirado em produto real** - Microsoft Store

### Portfólio
1. **Full-stack completo** - frontend, backend, mobile, IA
2. **Problema real resolvido** - não é todo-list
3. **Escalável** - arquitetura pensada para crescimento
4. **Open source** - código pode ser mostrado
5. **Tecnologias valorizadas** - Rust, React, IA, multiplataforma

---

## 📖 Documentação Adicional

### Para Desenvolver
- `docs/SETUP.md` - Guia de instalação e setup
- `docs/API.md` - Documentação das APIs internas
- `docs/CONTRIBUTING.md` - Como contribuir

### Para Usuários
- `README.md` - Visão geral e instalação
- `docs/USER_GUIDE.md` - Manual do usuário
- `docs/PRIVACY.md` - Política de privacidade
- `docs/FAQ.md` - Perguntas frequentes

---

## 🎓 Aprendizados Esperados

### Tecnologias
- ✅ React avançado (hooks, context, performance)
- ✅ TypeScript (tipos complexos, generics)
- ✅ Rust básico (ownership, tipos, async)
- ✅ Tauri (bridge Rust ↔ JavaScript)
- ✅ SQLite (queries, migrations, indexing)
- ✅ React Native (mobile development)
- ✅ Integração com LLMs (Ollama, Groq)

### Conceitos
- ✅ Arquitetura local-first
- ✅ Sincronização de dados
- ✅ Resolução de conflitos
- ✅ State management complexo
- ✅ Segurança e privacidade
- ✅ APIs RESTful
- ✅ Design multiplataforma

### Soft Skills
- ✅ Planejamento de projeto longo
- ✅ Documentação técnica
- ✅ Decisões de arquitetura
- ✅ Trade-offs técnicos
- ✅ UX thinking

---

## 📝 Notas Finais

### Prioridades

**Must Have (MVP):**
- CRUD de jogos
- Interface básica funcional
- SQLite local
- Ollama integration
- Steam import

**Should Have:**
- Epic/GOG scan
- Sincronização opcional
- UI polida estilo MS Store
- Export/Import

**Could Have:**
- Mobile app
- Estatísticas avançadas
- Gráficos de tempo de jogo
- Conquistas
- Social features

**Won't Have (por agora):**
- Multiplayer features
- Loja integrada
- Streaming integration
- VR support

### Extensões Futuras Possíveis

1. **Comunidade**
   - Compartilhar bibliotecas públicas
   - Reviews e recomendações sociais
   - Listas colaborativas

2. **Analytics**
   - Gráficos de tempo de jogo
   - Tendências de gêneros
   - Estatísticas comparativas

3. **Automação**
   - Auto-detect novos jogos instalados
   - Sync automático com lojas
   - Notificações de promoções

4. **Integrações**
   - Discord Rich Presence
   - Twitch integration
   - Xbox Game Pass
   - PlayStation Network

---

## 📄 Licença

**MIT License** (recomendado para portfólio)
- Permite uso comercial
- Permite modificação
- Requer atribuição
- Sem garantias

---

**Versão do Documento:** 1.0  
**Data:** Dezembro 2025  
**Autor:** Alan de Oliveira Gonçalves  
**Status:** Planejamento → Desenvolvimento

---

## 🔗 Links Úteis

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [React Documentation](https://react.dev/)
- [Shadcn/ui Components](https://ui.shadcn.com/)
- [Ollama Documentation](https://ollama.ai/docs)
- [Steam Web API](https://steamcommunity.com/dev)
- [Supabase Docs](https://supabase.com/docs)
- [Rust Book](https://doc.rust-lang.org/book/)
- [React Native Docs](https://reactnative.dev/docs/getting-started)