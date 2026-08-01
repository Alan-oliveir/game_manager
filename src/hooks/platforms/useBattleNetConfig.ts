import { usePlatformImportTrigger, usePlatformStatus } from '@/hooks';
import { platformsService } from '@/services/plataformsService';

/**
 * Hook para gerenciar a importação de jogos instalados via Battle.net.
 * Detecção 100% automática (lê `product.db` do Battle.net Agent);
 * não há OAuth nem caminho manual configurável — Windows apenas.
 */
export function useBattleNetConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = usePlatformStatus();

  const { isImporting: isImportingBattleNet, run: importBattleNetGames } =
    usePlatformImportTrigger(() => platformsService.importBattleNetGames(), {
      platformLabel: 'BattleNet',
      setStatus,
      onLibraryUpdate,
    });

  return {
    loading: { importingBattleNet: isImportingBattleNet },
    status,
    actions: { importBattleNetGames },
  };
}
