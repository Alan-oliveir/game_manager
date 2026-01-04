### 📅 02/01/2026 - 03/01/2026 - Pós-Lançamento: Infraestrutura, UX e Refatoração

**Tempo investido:** ~10h  
**Objetivo:** Polimento da versão v1.0.0 (MVP), implementação de infraestrutura de diagnóstico (logs), melhoria robusta de tratamento de erros e refatoração de componentes repetitivos de UI.

#### ✨ Implementações

- **Infraestrutura de Logs (`logger.rs`):**
    - Configuração da crate `tracing` com `tracing-appender`.
    - Logs rotativos diários salvos em arquivo local para debug em produção.
    - Filtros configurados para silenciar bibliotecas externas e focar no `game_manager_lib`.
- **UX & Feedback:**
    - **Tratamento de Erros (Trending):** Implementação de lógica robusta para diferenciar erros de Conexão (Offline), Configuração (Sem API Key) e Servidor (API Error).
    - **Loading Animado:** Remoção da Splashscreen nativa (que causava flash branco em SSDs rápidos) e substituição por um *Loading State* elegante no React.
    - **Empty States:** Telas amigáveis quando a busca ou listas (Wishlist/Trending) estão vazias.
- **Features:**
    - **Wishlist Manual:** Modal para buscar e adicionar jogos na Lista de Desejos pelo nome (usando busca da Steam) quando o jogo não aparece em "Em Alta".
- **Documentação:**
    - Criação do `CHANGELOG.md` seguindo o padrão "Keep a Changelog".
- **Refatoração (Clean Code):**
    - Criação do componente `ActionButton.tsx`.
    - Padronização de todos os botões redondos (Home, Biblioteca, Favoritos, Wishlist) para usar esse componente único, reduzindo drásticamente a duplicação de classes Tailwind.

#### 🐛 Problemas Encontrados

**Problema 1: Crash na página "Em Alta" (Rendered fewer hooks)**
- **Causa:** O `useEffect` de busca estava posicionado *após* um retorno condicional (`if (!isOnline) return...`). O React exige que hooks sejam chamados na mesma ordem sempre.
- **Solução:** Movido todos os hooks para o topo do componente, antes de qualquer `return`.
- **Aprendizado:** Regra de ouro do React: Hooks sempre no topo, nunca dentro de condicionais.

**Problema 2: Erro SQL na Wishlist ("no such column")**
- **Causa:** A tabela `wishlist` foi atualizada no código Rust (novos campos `steam_app_id`), mas o SQLite local manteve a estrutura antiga do MVP. O comando `CREATE TABLE IF NOT EXISTS` não atualiza esquemas existentes.
- **Solução:** Implementado um reset manual (apagar `library.db`) para desenvolvimento. Para produção futura, será necessário um sistema de Migrations.

**Problema 3: Botão de Menu (Dropdown) não abria com Componente Customizado**
- **Causa:** O componente `ActionButton` não repassava a referência (`ref`) do DOM, impedindo o `DropdownMenuTrigger` do Shadcn de se ancorar.
- **Solução:** Uso de `forwardRef` no componente `ActionButton`.

#### 💡 Decisões Técnicas

- **Decisão:** Remoção da Splashscreen Nativa do Tauri.
    - **Justificativa:** O app carrega rápido demais (~2s). A splashscreen nativa criava uma "corrida visual" com a janela principal. O loading via React oferece uma transição mais suave e controlada.
- **Decisão:** Logs apenas em arquivo na Release.
    - **Justificativa:** Usar `#[cfg(debug_assertions)]` para imprimir no terminal apenas em DEV. Em produção, logs vão apenas para arquivo para não impactar performance ou expor dados se o terminal for aberto.
- **Decisão:** Manter busca da Steam para Wishlist Manual.
    - **Justificativa:** Garante que temos o `steam_app_id` correto para monitoramento de preços, evitando erros de digitação do usuário.

#### 📚 Recursos Úteis

- [Rust Tracing Crate](https://docs.rs/tracing/latest/tracing/)
- [React ForwardRef Docs](https://react.dev/reference/react/forwardRef)

#### ⏭️ Próxima Sessão (Rumo à v2.0)

- [ ] Pesquisar fluxo OAuth2 para integração com IGDB/Twitch.
- [ ] Estudar leitura de manifestos locais da Epic Games Store.
- [ ] Refatorar serviços de API para usar Traits (`MetadataProvider`).
