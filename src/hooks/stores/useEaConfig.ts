import {
  useLibraryImportTrigger,
  useLibraryStatus,
  useLocalStorageLibraryPath,
} from '@/hooks';
import { storesService } from '@/services/storesService.ts';

/**
 * Hook para gerenciar a importação de jogos instalados via EA App.
 * EA não oferece um jeito viável de autenticar e listar a biblioteca completa.
 * A detecção depende inteiramente da pasta de instalação configurada pelo usuário.
 */
export function useEaConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = useLibraryStatus();
  const [installDir, setInstallDir] =
    useLocalStorageLibraryPath('ea_install_dir');

  const { isImporting: isImportingEa, run: importEaGames } =
    useLibraryImportTrigger(() => storesService.importEaGames(), {
      platformLabel: 'EA',
      setStatus,
      onLibraryUpdate,
    });

  return {
    installDir,
    setInstallDir,
    loading: {
      importingEa: isImportingEa,
    },
    status,
    actions: { importEaGames },
  };
}
