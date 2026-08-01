import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { usePlatformImportTrigger, usePlatformStatus } from '@/hooks';
import { platformsService } from '@/services/plataformsService';

/**
 * Hook para gerenciar a importação de jogos da Epic Games Store.
 * A detecção é automática (via manifests), não há credenciais a configurar.
 */
export function useEpicConfig(onLibraryUpdate?: () => void) {
  const { t } = useTranslation('platforms');
  const { status, setStatus } = usePlatformStatus();
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isCheckingAuth, setIsCheckingAuth] = useState(true);
  const [isLoggingIn, setIsLoggingIn] = useState(false);

  const checkAuth = async () => {
    setIsCheckingAuth(true);

    try {
      const authenticated = await platformsService.epicIsAuthenticated();
      setIsAuthenticated(authenticated);
    } finally {
      setIsCheckingAuth(false);
    }
  };

  useEffect(() => {
    checkAuth();
  }, []);

  const login = async () => {
    setIsLoggingIn(true);

    try {
      await platformsService.epicLogin();
      await checkAuth();
      setStatus({ type: 'success', message: t('epic_login_success') });
    } catch (err) {
      setStatus({ type: 'error', message: t('epic_login_error') });
    } finally {
      setIsLoggingIn(false);
    }
  };

  const logout = async () => {
    await platformsService.epicLogout();
    setIsAuthenticated(false);
  };

  const { isImporting: isImportingEpic, run: importEpicGames } =
    usePlatformImportTrigger(() => platformsService.importEpicGames(), {
      platformLabel: 'Epic',
      setStatus,
      onLibraryUpdate,
    });

  return {
    loading: {
      importingEpic: isImportingEpic,
      checkingAuth: isCheckingAuth,
      loggingIn: isLoggingIn,
    },
    isAuthenticated,
    status,
    actions: { importEpicGames, login, logout },
  };
}
