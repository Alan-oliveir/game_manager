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
  - Quebra do `App.tsx` em rotas manuais e criação da estrutura de pastas `/pages` (`Home`, `Libraries`, `Favorites`, `Settings`).
  - Centralização das ações (`gameActions`) para limpar a passagem de props.
- **Segurança (Security Hardening):**
  - Substituição do `localStorage` pelo `tauri-plugin-store` para armazenamento seguro/criptografado da API Key e Steam ID.
- **Dashboard (Home):**
  - Criação da tela inicial com KPIs (Tempo Total, Total de Jogos), lista de "Mais Jogados" e componente de "Sugestão Aleatória".
- **Infraestrutura:**
  - Configuração do banco SQLite para ser criado no diretório `app_data_dir` (AppData/Libraries), corrigindo conflitos de watcher do Tauri.

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

### 📅 27/12/2025 - Estabilização, Crawler de Gêneros e Página Em Alta

**Tempo investido:** ~5h  
**Objetivo:** Corrigir bugs críticos de persistência, implementar segurança de chaves, enriquecer dados com gêneros reais (Crawler) e criar a página de tendências (Trending) com API externa.

#### ✨ Implementações

- **Estabilização e Segurança:**
  - **Refatoração de Segurança:** Migração do `localStorage` para `tauri-plugin-store` (armazenamento criptografado).
  - **Correção de Persistência:** Ajuste no ciclo de vida do SQLite (movendo `PRAGMA journal_mode` para o backend) para evitar erros silenciosos que impediam o carregamento da lista.
- **Enriquecimento de Dados (Crawler Steam):**
  - Implementado sistema que busca metadados detalhados (Gêneros, Datas) na **Steam Store API** para jogos listados como "Desconhecido".
  - Lógica de **Rate Limiting** (pausa de 1.5s entre requisições) para evitar bloqueios de IP pela Valve.
- **Página "Em Alta" (Integração RAWG):**
  - Integração com a **RAWG API** para buscar jogos populares e lançamentos.
  - **Filtro Inteligente:** A lista exclui automaticamente jogos que o usuário já possui na biblioteca local.
  - **Otimização (Cache):** Implementado padrão de *State Lifting* no `App.tsx` para cachear os resultados da RAWG, eliminando loadings desnecessários ao trocar de abas.
- **UX de Configurações:**
  - Unificação do salvamento de chaves em um único botão "Salvar Todas as Configurações".

#### 🐛 Problemas Encontrados

**1. Falsa "Perda de Dados" ao Reiniciar**

- **Causa:** O comando SQL `PRAGMA journal_mode=WAL` retornava dados inesperados para o frontend, quebrando a promessa de inicialização.
- **Solução:** Configuração movida para o `setup` do Rust (backend), onde o retorno é tratado corretamente.

**2. Deadlock no Banco de Dados (Crawler)**

- **Causa:** O Crawler travava a interface inteira pois mantinha o banco bloqueado (`mutex lock`) enquanto esperava o tempo do Rate Limit (`sleep`).
- **Solução:** Uso de escopo `{}` no Rust para liberar o Mutex imediatamente após a escrita, permitindo que a UI respire enquanto o Crawler "dorme".

**3. Re-fetching Excessivo na Página Em Alta**

- **Causa:** O componente `Trending` era desmontado ao trocar de aba, perdendo os dados e forçando nova chamada de API (lenta) ao voltar.
- **Solução:** Elevação do estado (`trendingCache`) para o `App.tsx`, persistindo os dados na memória durante a sessão.

#### 💡 Decisões Técnicas

- **Store API vs User API:** Optou-se por usar a API da Loja Steam (mais lenta e restrita) para o enriquecimento, pois a API de Usuário não fornece Gêneros/Tags, essenciais para o futuro sistema de recomendação.
- **Persistência em AppData:** Mantida a decisão de usar diretórios padrão do SO (`AppData`), sacrificando a portabilidade em pen-drives em favor de maior compatibilidade com permissões do Windows.

#### 📚 Recursos Úteis

- [Rusqlite Documentation (Pragmas)](https://docs.rs/rusqlite/latest/rusqlite/)
- [RAWG API Documentation](https://rawg.io/apidocs)
- [React State Lifting Pattern](https://react.dev/learn/sharing-state-between-components)

#### ⏭️ Próxima Sessão (Fase 3 & 4)

- [ ] **Lista de Desejos:** Criar tabela no banco e integrar com API de preços (CheapShark).
- [ ] **Sistema de Recomendação V1:** Algoritmo *Content-Based* usando os gêneros capturados pelo Crawler.
- [ ] **Playlists Inteligentes:** Sugestão de "Backlog" baseada no tempo de jogo e afinidade.

---

### 📅 28/12/2025 - Modularização, Lista de Desejos e Integração de Preços

**Tempo investido:** ~8h
**Objetivo:** Refatorar o código backend para facilitar manutenção e implementar o sistema completo de Lista de Desejos com monitoramento automático de preços via API.

#### ✨ Implementações
- **Refatoração do Backend (Rust):**
  - Divisão do arquivo monolítico `lib.rs` em módulos organizados: `database/`, `commands/`, `services/`, `models/` e `constants/`.
- **Feature: Lista de Desejos (Wishlist):**
  - Criação da tabela `wishlist` no SQLite.
  - Implementação de comandos CRUD (`add`, `remove`, `get`) no Rust.
  - Criação da página `Wishlist.tsx` no Frontend e atualização da Sidebar.
  - Integração visual: Botões de "Adicionar à Lista" na página "Em Alta" (Trending).
- **Integração de Preços (CheapShark API):**
  - Novo serviço Rust (`cheapshark.rs`) para buscar ofertas em diversas lojas.
  - Comando `refresh_prices` que atualiza valores e links de compra em lote.
  - Exibição de preços (USD) e indicação visual de "OFERTA!" quando há descontos.
- **UX/Navegação:**
  - Implementação de botões funcionais para abrir links externos ("Ver na Loja", "Ver Detalhes") usando o navegador padrão do sistema.

#### 🐛 Problemas Encontrados
**1. Performance na Criptografia de Chaves**
- **Problema:** A implementação de criptografia com algoritmo Argon2 aumentou o tempo de inicialização do app para ~3 segundos.
- **Causa:** O custo computacional do Argon2 é intencionalmente alto para evitar força-bruta, o que impacta a UX em desktops.
- **Solução:** Revertido temporariamente para armazenamento simples (`.settings.dat`) via `tauri-plugin-store` para manter o app ágil durante o desenvolvimento, com planos de usar OS Keychain no futuro.

**2. Erro de Decodificação JSON (CheapShark)**
- **Problema:** O comando de atualizar preços falhava com `error decoding response body`.
- **Causa:** A struct Rust esperava campos `price` e `retail_price`, mas a API retornava `salePrice` e `normalPrice`.
- **Solução:** Uso do atributo `#[serde(rename = "...")]` nas structs para mapear corretamente os campos JSON da API para os campos do Rust.

#### 💡 Decisões Técnicas
- **Separação de Módulos Rust:** Decidi quebrar o `lib.rs` pois o arquivo estava ficando muito extenso e difícil de navegar. A nova estrutura separa claramente *Comandos* (API p/ Frontend) de *Serviços* (Lógica de Negócios/HTTP) e *Database* (SQL).

#### 📚 Recursos Úteis
- [CheapShark API Documentation](https://apidocs.cheapshark.com/)
- [Serde JSON Field Renaming](https://serde.rs/field-attrs.html)
- [Rust Reqwest Crate](https://docs.rs/reqwest/latest/reqwest/)

#### ⏭️ Próxima Sessão
- [ ] Implementar Algoritmo de Recomendação V1 (Pontuação baseada em Gêneros).
- [ ] Criar Playlist Sugerida na Home baseada nesses scores.
- [ ] Conversão de moedas (USD -> BRL) para exibição de preços.

---

### 📅 29/12/2025 - Motor de Recomendação V1 (Content-Based)

**Tempo investido:** ~2h
**Objetivo:** Implementar a primeira versão do algoritmo de recomendação, capaz de aprender o perfil do utilizador (afinidade por géneros) e personalizar a interface.

#### ✨ Implementações
- **Backend (Rust):**
  - Criação do serviço `recommendation.rs`: Lógica matemática que calcula pesos para cada género baseado no tempo de jogo e favoritos.
  - Novo comando `get_user_profile`: Expõe o perfil calculado para o frontend.
  - Desacoplamento total: O motor de recomendação não sabe onde os dados serão usados, apenas processa números.
- **Frontend (React):**
  - Hook `useRecommendation`: Encapsula a lógica de calcular a afinidade de um jogo específico com o perfil do utilizador.
  - **Reordenação Inteligente:** A página "Em Alta" agora ordena as sugestões baseada na afinidade (Score do Utilizador) em vez da ordem padrão da API.
  - **Feedback Visual:** Adição da badge "TOP PICK" para jogos com alta compatibilidade.

#### 💡 Decisões Técnicas
- **Algoritmo Deterministico:** Optei por não usar ML complexo ou vetores multidimensionais agora. Um sistema de pesos simples (`Playtime * 1.0 + Favorites * 50.0`) é mais fácil de debugar, extremamente rápido e funciona offline.
- **Cálculo no Frontend:** O Backend entrega o perfil (`{"RPG": 1000, "FPS": 0}`), mas é o Frontend que calcula a nota de cada jogo da lista "Em Alta". Isso poupa o Backend de ter que processar listas que vêm de APIs externas (RAWG).

---

### Padronização de UI

**Tempo investido:** ~5h
**Objetivo:** Otimizar o carregamento da Home e padronizar os cards de jogos em todas as telas.

#### ✨ Implementações
- **Recomendação:**
  - Integração: Badges "TOP PICK" e "PARA VOCÊ" visíveis na Home e Trending.
- **Home Dashboard 2.0:**
  - **State Lifting:** Cache de `trending` e `profile` movido para o `App.tsx`, eliminando carregamentos desnecessários ao navegar entre abas.
  - **Lógica de Backlog:** Seção "O Melhor do seu Backlog" sugerindo jogos não jogados com alta afinidade.
- **Nova Seção "Trending":**
  - Implementação de busca de lançamentos de novos jogos para a nova seção "Lançamentos Aguardados" (Upcoming).
  - Filtro automático de lançamentos baseado no gosto do usuário.
- **Refatoração de UI (DRY):**
  - Criação do componente `StandardGameCard.tsx`.
  - Migração das páginas **Home**, **Trending** e **Wishlist** para usar este componente único.
- **Feature Launcher:**
  - Utilitário centralizado `launcher.ts` para iniciar jogos via protocolo `steam://`.

#### 🐛 Problemas Encontrados
**1. Reloading constante da Home**
- **Problema:** Ao sair e voltar para a Home, a API da RAWG era chamada novamente.
- **Solução:** Implementação de cache no componente pai (`App.tsx`) injetando os dados e funções de *set* via props para a Home.

**2. Redundância de Código nos Cards**
- **Problema:** Cada página (Wishlist, Trending, Home) tinha sua própria implementação HTML/CSS dos cards.
- **Solução:** Abstração completa para `StandardGameCard`, recebendo ações (botões) via prop `ReactNode`.

#### ⏭️ Próxima Sessão
- [ ] **Refatoração Final:** Aplicar `StandardGameCard` e o `launcher` nas páginas **Library** e **Favorites**.
- [ ] **Nova Página "Playlist":** Criar uma lista de reprodução manual/sugerida onde o usuário pode reordenar a fila de próximos jogos ("Up Next").
- [ ] **Polimento Visual:** Ajustes finos de espaçamento e consistência de design.

---

### 📅 30/12/2025 - Refatoração Visual, Playlist e Lançamento v1.0 (Playlite)

**Tempo investido:** ~6h
**Objetivo:** Implementar a funcionalidade de Playlist, unificar a identidade visual (Hero/Cards), resolver problemas de layout/scroll e finalizar a estrutura base do aplicativo.

#### ✨ Implementações
- **Feature Playlist:**
  - Criação do hook `usePlaylist` com persistência local (`.settings.dat`).
  - Interface de "Fila" (Queue) com reordenação manual e sugestões laterais inteligentes baseadas no backlog.
- **Componentização (DRY):**
  - Criação do componente `Hero.tsx` reutilizável, limpando o código da `Home` e `Trending`.
  - Refatoração final do `StandardGameCard` para suportar ações customizadas e botão "Play" contextual.
- **UX & UI:**
  - Implementação de **Scrollbar Customizada** via CSS puro (tema dark).
  - Substituição de `alert()` por notificações elegantes usando **Sonner** (Toasts).
  - Responsividade aprimorada no Modal de Detalhes e na página de Playlist (Full width).
- **Branding:**
  - Renomeação do projeto na interface para **Playlite**.

#### 🐛 Problemas Encontrados
**Problema 1:** Layout da Playlist vazio em telas grandes (1080p+)
- **Causa:** O container da lista estava limitado a `max-w-3xl`, deixando muito espaço em branco.
- **Solução:** Removi a limitação de largura da coluna principal e ajustei a sidebar de sugestões para ter tamanho fixo, ocupando 100% da tela disponível.
- **Aprendizado:** Em aplicações desktop (ao contrário da web), devemos aproveitar melhor a largura total da janela ("screen real estate").

**Problema 2:** Erro de Tipagem `isLocalGame` na Home
- **Causa:** Ao misturar jogos locais (`Game`) e da nuvem (`RawgGame`) no Hero, o TypeScript perdeu a referência de quais propriedades existiam.
- **Solução:** Criação de Type Guards e funções helpers explícitas para normalizar os dados antes de passar para o componente.

#### 💡 Decisões Técnicas
- **Decisão:** Uso do padrão de **Composição** nos Cards e Hero.
  - **Justificativa:** Em vez de passar dezenas de props booleanas (`showHeart`, `showPlay`), passamos os próprios componentes (`actions={<Button>...}`) para dentro do container. Isso desacoplou a lógica visual da lógica de negócio de cada página.
- **Decisão:** Manter a ordenação da Playlist sem bibliotecas de Drag-and-Drop (por enquanto).
  - **Justificativa:** Botões de "Subir/Descer" são mais fáceis de implementar de forma robusta e acessível nesta fase inicial, evitando peso extra no bundle.

#### 📚 Recursos Úteis
- [Shadcn UI - Sonner](https://ui.shadcn.com/docs/components/sonner)

#### ⏭️ Próxima Fase (Roadmap v2.0)
- [ ] **IA:** Melhorar recomendação com explicações via LLM local ou API.
- [ ] **Integrações:** Implementar importação de bibliotecas (Epic, GOG, Amazon).
- [ ] **Cloud:** Sincronização de saves e playlist entre dispositivos (Supabase).
- [ ] **Mobile:** Iniciar estudos para portar a interface para React Native.

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
*Última atualização: 30/12/2025*
