import { useLibraryImportTrigger, useLibraryStatus } from '@/hooks';
import { storesService } from '@/services/storesService.ts';

/**
 * Hook para gerenciar a importação de jogos instalados via Battle.net.
 * Detecção 100% automática (lê `product.db` do Battle.net Agent);
 * não há OAuth nem caminho manual configurável — Windows apenas.
 */
export function useBattleNetConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = useLibraryStatus();

  const { isImporting: isImportingBattleNet, run: importBattleNetGames } =
    useLibraryImportTrigger(() => storesService.importBattleNetGames(), {
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
