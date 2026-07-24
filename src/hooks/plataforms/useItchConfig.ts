import { useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { usePlatformImportAction, usePlatformStatus } from '@/hooks';
import { platformsService } from '@/services/plataformsService';

export type ItchImportMode = 'installed' | 'full';

const MODE_STORAGE_KEY = 'itch_import_mode';

function readStoredMode(): ItchImportMode {
  if (typeof localStorage === 'undefined') return 'installed';

  return localStorage.getItem(MODE_STORAGE_KEY) === 'full'
    ? 'full'
    : 'installed';
}

/**
 * Hook para gerenciar a importação de jogos da Itch.io.
 * Detecção 100% automática baseada no butler.db.
 */
export function useItchConfig(onLibraryUpdate?: () => void) {
  const { t } = useTranslation('platforms');
  const { status, setStatus } = usePlatformStatus();
  const [mode, setModeState] = useState<ItchImportMode>(readStoredMode);

  const setMode = useCallback((next: ItchImportMode) => {
    setModeState(next);

    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(MODE_STORAGE_KEY, next);
    }
  }, []);

  const { isImporting: isImportingItch, run: importItchGames } =
    usePlatformImportAction(
      () => platformsService.importItchGames(mode === 'full'),
      {
        setStatus,
        onLibraryUpdate,
        loadingMessage:
          mode === 'full'
            ? t('itch_importing_full_status')
            : t('itch_importing_status'),
      }
    );

  return {
    mode,
    setMode,
    loading: { importingItch: isImportingItch },
    status,
    actions: { importItchGames },
  };
}
