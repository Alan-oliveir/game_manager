import {
  useLocalStoragePlatformPath,
  usePlatformImportTrigger,
  usePlatformStatus,
} from '@/hooks';
import { platformsService } from '@/services/plataformsService';

/**
 * Hook para gerenciar a importação de jogos instalados via EA App.
 * EA não oferece um jeito viável de autenticar e listar a biblioteca completa.
 * A detecção depende inteiramente da pasta de instalação configurada pelo usuário.
 */
export function useEaConfig(onLibraryUpdate?: () => void) {
  const { status, setStatus } = usePlatformStatus();
  const [installDir, setInstallDir] =
    useLocalStoragePlatformPath('ea_install_dir');

  const { isImporting: isImportingEa, run: importEaGames } =
    usePlatformImportTrigger(() => platformsService.importEaGames(), {
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
