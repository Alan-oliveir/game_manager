import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { usePlatformImportTrigger, usePlatformStatus } from '@/hooks';
import { platformsService } from '@/services/plataformsService';

/**
 * Hook para gerenciar a importação de jogos da Amazon Games.
 * Combina biblioteca completa (via conta conectada) com jogos instalados
 * detectados automaticamente pelo Amazon Games App (Windows apenas).
 */
export function useAmazonConfig(onLibraryUpdate?: () => void) {
  const { t } = useTranslation('platforms');
  const { status, setStatus } = usePlatformStatus();
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isCheckingAuth, setIsCheckingAuth] = useState(true);
  const [isLoggingIn, setIsLoggingIn] = useState(false);

  const checkAuth = async () => {
    setIsCheckingAuth(true);

    try {
      const authenticated = await platformsService.amazonIsAuthenticated();
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
      await platformsService.amazonLogin();
      await checkAuth();
      setStatus({ type: 'success', message: t('amazon_login_success') });
    } catch (err) {
      setStatus({ type: 'error', message: t('amazon_login_error') });
    } finally {
      setIsLoggingIn(false);
    }
  };

  const logout = async () => {
    await platformsService.amazonLogout();
    setIsAuthenticated(false);
  };

  const { isImporting: isImportingAmazon, run: importAmazonGames } =
    usePlatformImportTrigger(() => platformsService.importAmazonGames(), {
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
