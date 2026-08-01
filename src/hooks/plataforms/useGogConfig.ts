import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { parsePlatformError } from '@/errors/errorMessages';
import {
  useLocalStoragePlatformPath,
  usePlatformImportTrigger,
  usePlatformStatus,
} from '@/hooks';
import { platformsService } from '@/services/plataformsService';
import { toast } from '@/utils/toast';

/**
 * Hook para gerenciar a conexão OAuth e importação da biblioteca GOG.
 * Diferente das demais fontes, exige um login prévio (via WebviewWindow)
 * antes que a importação da biblioteca esteja disponível.
 */
export function useGogConfig(onLibraryUpdate?: () => void) {
  const { t } = useTranslation('platforms');
  const { status, setStatus } = usePlatformStatus();
  const [gogGamesDir, setGogGamesDir] =
    useLocalStoragePlatformPath('gog_games_dir');

  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [checkingAuth, setCheckingAuth] = useState(true);

  const refreshAuthStatus = useCallback(async () => {
    try {
      const authenticated = await platformsService.gogIsAuthenticated();
      setIsAuthenticated(authenticated);
    } catch {
      setIsAuthenticated(false);
    }
  }, []);

  useEffect(() => {
    refreshAuthStatus().finally(() => setCheckingAuth(false));
  }, [refreshAuthStatus]);

  const [isLoggingIn, setIsLoggingIn] = useState(false);

  const runLogin = useCallback(async () => {
    setIsLoggingIn(true);

    try {
      await platformsService.gogLogin();
    } catch {
      // erro tratado no catch do login() abaixo, via parsePlatformError
    } finally {
      setIsLoggingIn(false);
    }
  }, []);

  // `run` engole erros internamente (só atualiza `status` e mostra toast),
  // então a Promise sempre resolve ao final do fluxo — usa isso apenas
  // como sinal de "tentativa terminou" e reconsulta o estado real depois,
  // já que `run` não devolve sucesso/falha diretamente.
  const login = useCallback(async () => {
    await runLogin();
    await refreshAuthStatus();
  }, [runLogin, refreshAuthStatus]);

  const logout = useCallback(async () => {
    try {
      await platformsService.gogLogout();
      setIsAuthenticated(false);
      setStatus({ type: null, message: '' });
      toast.success(t('gog_disconnected_success'));
    } catch (e) {
      const errorMsg = parsePlatformError(e);
      setStatus({ type: 'error', message: errorMsg });
      toast.error(errorMsg);
    }
  }, [setStatus, t]);

  const { isImporting: isImportingGog, run: importGogGames } =
    usePlatformImportTrigger(() => platformsService.importGogGames(), {
      platformLabel: 'GOG',
      setStatus,
      onLibraryUpdate,
    });

  return {
    isAuthenticated,
    gogGamesDir,
    setGogGamesDir,
    loading: {
      checkingAuth,
      loggingIn: isLoggingIn,
      importingGog: isImportingGog,
    },
    status,
    actions: { login, logout, importGogGames },
  };
}
