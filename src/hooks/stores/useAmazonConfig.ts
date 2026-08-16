import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useLibraryImportTrigger, useLibraryStatus } from '@/hooks';
import { storesService } from '@/services/storesService.ts';

/**
 * Hook para gerenciar a importação de jogos da Amazon Games.
 * Combina biblioteca completa (via conta conectada) com jogos instalados
 * detectados automaticamente pelo Amazon Games App (Windows apenas).
 */
export function useAmazonConfig(onLibraryUpdate?: () => void) {
  const { t } = useTranslation('platforms');
  const { status, setStatus } = useLibraryStatus();
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isCheckingAuth, setIsCheckingAuth] = useState(true);
  const [isLoggingIn, setIsLoggingIn] = useState(false);

  const checkAuth = async () => {
    setIsCheckingAuth(true);

    try {
      const authenticated = await storesService.amazonIsAuthenticated();
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
      await storesService.amazonLogin();
      await checkAuth();
      setStatus({ type: 'success', message: t('amazon_login_success') });
    } catch (err) {
      setStatus({ type: 'error', message: t('amazon_login_error') });
    } finally {
      setIsLoggingIn(false);
    }
  };

  const logout = async () => {
    await storesService.amazonLogout();
    setIsAuthenticated(false);
  };

  const { isImporting: isImportingAmazon, run: importAmazonGames } =
    useLibraryImportTrigger(() => storesService.importAmazonGames(), {
      platformLabel: 'Amazon',
      setStatus,
      onLibraryUpdate,
    });

  return {
    loading: {
      importingAmazon: isImportingAmazon,
      checkingAuth: isCheckingAuth,
      loggingIn: isLoggingIn,
    },
    isAuthenticated,
    status,
    actions: { importAmazonGames, login, logout },
  };
}
