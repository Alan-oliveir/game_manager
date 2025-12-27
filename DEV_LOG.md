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

---

## Fase 1: MVP (Desktop)

### 📅 24/12/2025 - Início do projeto

**Tempo Investido:** ~10h  
**Objetivo:** Configuração do ambiente, arquitetura híbrida (Rust + React) e persistência de dados local.

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

### 💡 5. Lições Aprendidas

#### Arquitetura

- Separar lógica em `lib.rs` desde o início economiza refatoração futura
- `Mutex<Connection>` é essencial para thread-safety com SQLite
- Componentização React facilita manutenção e escalabilidade

#### Ferramentas

- Tailwind v4 é mais rápido mas tem documentação limitada (ainda novo)
- Shadcn/UI acelera desenvolvimento de UI mas requer configuração cuidadosa

#### Desenvolvimento

- Mock data é útil para testar UI antes do backend estar pronto
- Documentar problemas economiza tempo em problemas similares
- TypeScript evita muitos bugs em runtime

#### Performance

- SQLite embarcado elimina necessidade de servidor externo
- Tauri gera binários ~10x menores que Electron
- React 19 tem melhorias significativas de performance

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

### 📅 25/12/2025 - Implementação do CRUD Completo e Refinamento de UI

**Tempo investido:** ~4h  
**Objetivo:** Implementar funcionalidades de escrita no banco (Adicionar, Editar, Excluir), corrigir persistência de imagens e polir a responsividade do Grid.

#### ✨ Implementações

- **CRUD Completo:**
  - Implementado comandos Rust `update_game` e `delete_game`.
  - Criado fluxo de exclusão com confirmação.
  - Implementado fluxo de edição reaproveitando o `AddGameModal` com preenchimento automático de dados.
- **Interface (UI):**
  - **Grid Responsivo:** Ajuste fino no CSS para variar de 1 coluna (Mobile) até 5 colunas (Full HD), melhorando a legibilidade.
  - **Menu de Contexto:** Adicionado componente `DropdownMenu` (Shadcn) no Card para ações de Editar/Excluir.
  - **Botão Header:** Correção de contraste (agora sempre azul) e responsividade (esconde texto em telas pequenas).
- **Backend:**
  - Ajuste na tabela SQLite para suportar coluna `cover_url`.

#### 🐛 Problemas Encontrados

**1. Imagem não salvando no banco**

- **Causa:** O Tauri converte variáveis automaticamente de `camelCase` (JS) para `snake_case` (Rust). Eu estava enviando `cover_url` no frontend, mas o binding esperava `coverUrl` para mapear corretamente para o argumento do Rust.
- **Solução:** Alterei a chamada do `invoke` para usar `coverUrl: ...`.
- **Aprendizado:** Atenção redobrada na nomenclatura de variáveis na fronteira entre JS e Rust (Serde).

**2. App reiniciando ao salvar dados**

- **Causa:** O comando `npm run tauri dev` observa mudanças em todos os arquivos. Como o SQLite (`library.db`) mudava ao salvar um jogo, o Tauri achava que era código e recarregava o app.
- **Solução:** Adicionei `library.db` na lista de `ignored` no `tauri.conf.json`.

**3. Coluna inexistente no banco**

- **Causa:** O arquivo `.db` foi criado nas primeiras execuções sem a coluna `cover_url`. O comando `CREATE TABLE IF NOT EXISTS` não atualiza tabelas antigas.
- **Solução:** Deletei o arquivo `.db` manualmente para forçar a recriação da tabela com o schema novo.

#### 💡 Decisões Técnicas

- **Reutilização de Modal:** Decidi usar o mesmo componente `AddGameModal` para criação e edição. Isso evitou duplicar código de formulário. O controle é feito passando a prop opcional `gameToEdit`.
- **Grid Manual vs Auto-fit:** Optei por definir colunas explicitamente (`grid-cols-1` até `grid-cols-5`) em vez de usar `minmax` automático do CSS, para ter controle total sobre quantos cards aparecem em cada resolução específica.

#### ⏭️ Próxima Sessão

- [ ] Implementar funcionalidade da barra de Busca (Filtro em tempo real).
- [ ] Adicionar inputs de "Avaliação" (Estrelas) e "Tempo de Jogo" no Modal.
- [ ] Criar lógica da página "Favoritos" (Sidebar).

---

### 📅 26/12/2025 - Finalização da Fase 1 (Busca e Navegação)

**Tempo investido:** ~2h
**Objetivo:** Implementar sistema de busca em tempo real e lógica de navegação entre Biblioteca e Favoritos.

#### ✨ Implementações
- **Busca Reativa:**
  - Transformado o input do Header em componente controlado.
  - Criada lógica centralizada `getDisplayedGames` que filtra por Nome, Gênero ou Plataforma instantaneamente.
- **Navegação (Sidebar):**
  - Implementada lógica para a aba "Favoritos", exibindo apenas jogos marcados.
  - A busca agora funciona globalmente (filtra dentro da biblioteca ou dentro dos favoritos).
- **Refatoração:**
  - Removido sistema de "Mock Data" (dados falsos). Agora o Grid lida com estados vazios ("Nenhum jogo encontrado").
  - Limpeza de código morto no `App.tsx`.

#### 🐛 Problemas Encontrados
**1. Edição de Gênero não salvando**
- **Causa:** O comando SQL `update_game` no Rust estava desatualizado, atualizando apenas `name` e `cover_url`, ignorando os novos campos.
- **Solução:** Atualizei a query SQL para incluir `genre`, `platform`, `rating` e `playtime`.

**2. Busca exibindo dados falsos**
- **Causa:** O componente `GameGrid` tinha uma regra antiga para mostrar dados de exemplo se a lista estivesse vazia. Ao buscar um termo sem resultados, a lista ficava vazia e os dados falsos apareciam.
- **Solução:** Removi a lógica de mock. Agora exibe um componente "Empty State" informativo.

#### 💡 Decisões Técnicas
- **Filtragem no Client-Side:** Como a biblioteca local dificilmente passará de alguns milhares de jogos, optei por filtrar os arrays no Javascript (`.filter`) em vez de fazer queries SQL complexas (`LIKE %...%`) a cada tecla digitada. Isso garante UI instantânea (Zero Latência).

#### ⏭️ Próxima Fase (Fase 2)
- [ ] Iniciar integração com Steam API (Backend Rust).
- [ ] Criar sistema de importação automática de jogos.

---

## Fase 2: Integração com Lojas Digitais

### 📅 26/12/2025 - Integração Steam, Refatoração e Hardening de Segurança

**Tempo investido:** ~5h
**Objetivo:** Conectar a aplicação à API da Steam para importação automática, refatorar a arquitetura do frontend para suportar múltiplas páginas e corrigir vulnerabilidades de segurança.

#### ✨ Implementações
- **Integração com Steam API:**
  - Criado módulo Rust (`steam_service`) usando `reqwest` para buscar jogos do usuário.
  - Implementada lógica de "Upsert" (Inserir ou Ignorar) para não duplicar jogos existentes no banco.
- **Refatoração Arquitetural (Frontend):**
  - Quebra do `App.tsx` em rotas manuais e criação da estrutura de pastas `/pages` (`Home`, `Library`, `Favorites`, `Settings`).
  - Centralização das ações (`gameActions`) para limpar a passagem de props.
- **Segurança (Security Hardening):**
  - Substituição do `localStorage` pelo `tauri-plugin-store` para armazenamento seguro/criptografado da API Key e Steam ID.
- **Dashboard (Home):**
  - Criação da tela inicial com KPIs (Tempo Total, Total de Jogos), lista de "Mais Jogados" e componente de "Sugestão Aleatória".
- **Infraestrutura:**
  - Configuração do banco SQLite para ser criado no diretório `app_data_dir` (AppData/Library), corrigindo conflitos de watcher do Tauri.

#### 🐛 Problemas Encontrados
**1. Loop de Reinício Infinito**
- **Causa:** O arquivo `library.db` estava sendo criado dentro da pasta `src-tauri`. Como o Tauri monitora essa pasta para "Hot Reload", cada alteração no banco disparava uma recompilação, que alterava o banco novamente, criando um loop.
- **Solução:** Alteração no `lib.rs` para usar a API `app.path().app_data_dir()`, salvando o banco na pasta de dados do usuário do Sistema Operacional.

**2. API Key Exposta**
- **Causa:** Inicialmente salvamos a API Key da Steam no `localStorage` do navegador.
- **Solução:** Auditoria de código apontou risco de segurança. Migramos para o plugin nativo `tauri-plugin-store` que persiste dados no disco com maior segurança e isolamento da WebView.

**3. Capas de Jogos Quebradas**
- **Causa:** A API da Steam retorna URLs de imagem baseadas no ID, mas nem todos os jogos antigos possuem a imagem vertical no servidor da CDN.
- **Solução:** Adicionado tratamento de erro `onError` no componente `GameCard` para ativar o fallback visual (card cinza com nome) automaticamente.

**4. Duplicação de Chamada na Importação**
- **Causa:** Erro de "Copy & Paste" no `Settings.tsx` gerou dois blocos de código idênticos para importar jogos.
- **Solução:** Remoção do código duplicado na função `handleImport`.

#### 💡 Decisões Técnicas
- **Pages vs Components:** Decidi separar "Telas" (que têm acesso ao estado global e roteamento) de "Componentes" (que apenas recebem dados puros). Isso facilitou a leitura do `App.tsx`.
- **Persistência Local de Chaves:** Optei por salvar as credenciais da Steam apenas no dispositivo do usuário (client-side) em vez de criar um backend na nuvem, mantendo a filosofia "Local-First" e privacidade do projeto.
- **Pausa no Enriquecimento de Dados:** A API `GetOwnedGames` da Steam não retorna gêneros. Decidi manter os dados como "Desconhecido" temporariamente e focar na estrutura do App, deixando a implementação de um Crawler de metadados para uma sessão futura dedicada.

#### 📚 Recursos Úteis
- [Tauri Plugin Store Documentation](https://v2.tauri.app/plugin/store/)
- [Steam Web API Documentation](https://developer.valvesoftware.com/wiki/Steam_Web_API)
- [Reqwest Crate (Rust)](https://docs.rs/reqwest/latest/reqwest/)

#### ⏭️ Próxima Sessão
- [ ] Estudo aprofundado do código gerado (Rust/Tauri Bridge e Security).
- [ ] Planejamento do "Crawler" para buscar Gêneros e Tags dos jogos (Enriquecimento).
- [ ] Desenvolvimento da página "Em Alta" (Trending).

---

### 📅 27/12/2025 - Estabilização, Debugging de Persistência e Documentação

**Tempo investido:** ~2h
**Objetivo:** Retomar o desenvolvimento, auditar o código com ferramentas de IA, corrigir bugs de inicialização e documentar o progresso público.

#### ✨ Implementações
- **Refatoração de Segurança:** Implementação completa do `tauri-plugin-store` para gerenciamento seguro de chaves de API (substituindo o localStorage vulnerável).
- **Correção de Inicialização (Persistência):** Ajuste no ciclo de vida do banco de dados.
  - Movida a configuração `PRAGMA journal_mode=WAL` do comando `init_db` (invocado pelo frontend) para o `setup` do Tauri (backend), evitando erros de execução que impediam o carregamento da lista de jogos.
  - Atualizado `App.tsx` para garantir que `refreshGames` seja chamado mesmo se a inicialização do banco retornar avisos não críticos.
- **Documentação:** Atualização dos arquivos README e docs públicos do repositório.

#### 🐛 Problemas Encontrados
**1. Falsa "Perda de Dados" ao Reiniciar**
- **Problema:** Ao fechar e abrir o app, a lista de jogos aparecia vazia, embora o arquivo `library.db` tivesse dados. Reimportar da Steam trazia os jogos de volta (0 adicionados).
- **Causa:** O comando SQL `PRAGMA journal_mode=WAL` retorna uma linha de resultado ("wal"). A função `init_db` usava `conn.execute` (que espera 0 linhas de retorno), causando um erro silencioso. Esse erro quebrava a promessa no `useEffect` do React, impedindo a chamada de `refreshGames`.
- **Solução:** Mover a configuração do PRAGMA para o `setup` da aplicação (onde erros podem ser ignorados ou tratados sem afetar o frontend) e remover do `init_db`.

**2. Bug de Duplicação no Settings**
- **Problema:** A função de importação estava duplicada no arquivo `Settings.tsx`, podendo causar condições de corrida.
- **Solução:** Remoção do código redundante identificada na revisão.

#### 💡 Decisões Técnicas
- **Persistência em AppData vs Portátil:** Mantida a decisão de usar `app_data_dir` (AppData no Windows). Embora impeça o app de ser "portátil" (rodar de pen drive com dados), garante compatibilidade com permissões de usuário do Windows e segue padrões de instalação profissional.
- **Uso de Ferramentas de Análise (IA):** Utilização de análise estática via LLM para identificar vulnerabilidades de segurança (API Key) e bugs lógicos que passariam despercebidos em testes manuais simples.

#### 📚 Recursos Úteis
- [Rusqlite Documentation (Pragmas)](https://docs.rs/rusqlite/latest/rusqlite/)
- [Tauri Directories Guide](https://v2.tauri.app/reference/javascript/path/)

#### ⏭️ Próxima Sessão
- [ ] Implementar Crawler/Scraper para buscar Gêneros e Tags reais (substituindo "Desconhecido").
- [ ] Desenvolver a página "Em Alta" com integração de API pública (RAWG/IGDB).
- [ ] Polimento final da UI da Home com dados reais.

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
- Hydra Launcher

---

*Autor: Alan de Oliveira Gonçalves*  
*Última atualização: 26/12/2025*
