/**
 * Hooks organizados por domínio/feature
 *
 * Estrutura:
 * - library/        - Gerenciamento de biblioteca
 * - recommendation/ - Sistema de recomendação
 * - trending/       - Jogos em alta e lançamentos
 * - wishlist/       - Lista de desejos
 * - ui/             - Componentes de interface
 * - user/           - Perfil e detalhes
 * - common/         - Utilitários genéricos
 * - update/         - Verificação e aplicação de atualizações
 * - configuration/  - Configurações e integrações
 * - game_detail/    - Hook para detalhes e dados extras para jogos
 * - libraries/     - Hook para gerenciar importações de plataformas (Steam, Epic, etc.)
 * - useHome.ts      - Hook específico para a tela inicial (na raizs)
 *
 * Cada pasta pode conter múltiplos hooks relacionados àquela funcionalidade.
 */

// Library hooks
export * from './library';

// Recommendation hooks
export * from './recommendation';

// Trending hooks
export * from './trending';

// Wishlist hooks
export * from './wishlist';

// UI hooks
export * from './ui';

// User hooks
export * from './user';

// Common hooks
export * from './common';

// Update hooks
export * from './update';

// Configuration hooks
export * from './configuration';

// Game hooks
export * from './game_detail';

// Plataforms hooks
export * from './stores';

// Achievements hooks
export * from './achievements';

// Special hooks (na raiz)
export * from './useEnrichmentNotifications';
export * from './useEnrichmentProgressNotifications';
export * from './useHome';
