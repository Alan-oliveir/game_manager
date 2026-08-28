import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useLibraryImportTrigger, useLibraryStatus } from '@/hooks';
import { settingsService } from '@/services/settingsService.ts';
import { storesService } from '@/services/storesService.ts';
import { toast } from '@/utils/toast';

/**
 * Hook para gerenciar a importação de jogos instalados via Xbox App / Microsoft Store
 * (Gaming Services). Detecção totalmente automática (via marcador `.GamingRoot` em
 * cada drive) — sem login e sem pasta a configurar.
 *
 * Também gerencia credenciais de API do Xbox Live para conquistas.
 */
export function useXboxConfig(onLibraryUpdate?: () => void) {
  const { t } = useTranslation('platforms');
  const { status, setStatus } = useLibraryStatus();

  const [xboxConfig, setXboxConfig] = useState({
    xboxLiveClientId: '',
    xboxLiveClientSecret: '',
  });

  const [isLoadingSecrets, setIsLoadingSecrets] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  // Carrega credenciais salvas ao montar
  useEffect(() => {
    settingsService
      .getSecrets()
      .then(data => {
        setXboxConfig(prev => ({
          ...prev,
          xboxLiveClientId: data.xboxLiveClientId || '',
          xboxLiveClientSecret: data.xboxLiveClientSecret || '',
        }));
      })
      .catch(e => console.error('Erro ao carregar credenciais Xbox Live', e))
      .finally(() => setIsLoadingSecrets(false));
  }, []);

  /**
   * Salva as credenciais Xbox Live (Client ID e Secret) no keystore seguro.
   */
  const saveXboxKeys = async () => {
    setIsSaving(true);
    setStatus({ type: null, message: '' });

    try {
      const currentSecrets = await settingsService.getSecrets();

      await settingsService.setSecrets({
        steamId: currentSecrets.steamId || null,
        steamApiKey: currentSecrets.steamApiKey || null,
        steamgriddbApiKey: currentSecrets.steamgriddbApiKey || null,
        rawgApiKey: currentSecrets.rawgApiKey || null,
        geminiApiKey: currentSecrets.geminiApiKey || null,
        gamebrainApiKey: currentSecrets.gamebrainApiKey || null,
        nexusApiKey: currentSecrets.nexusApiKey || null,
        igdbClientId: currentSecrets.igdbClientId || null,
        igdbClientSecret: currentSecrets.igdbClientSecret || null,
        xboxLiveClientId: xboxConfig.xboxLiveClientId.trim() || null,
        xboxLiveClientSecret: xboxConfig.xboxLiveClientSecret.trim() || null,
        itadApiKey: currentSecrets.itadApiKey || null,
      });

      const successMsg = t('xbox_live_credentials_saved');
      setStatus({ type: 'success', message: successMsg });
      toast.success(successMsg);
    } catch (error) {
      const errorMsg = `${t('common_save_error_prefix')} ${error}`;
      setStatus({ type: 'error', message: errorMsg });
      toast.error(errorMsg);
    } finally {
      setIsSaving(false);
    }
  };

  const { isImporting: isImportingXbox, run: importXboxGames } =
    useLibraryImportTrigger(() => storesService.importXboxGames(), {
      platformLabel: 'Xbox',
      setStatus,
      onLibraryUpdate,
    });

  return {
    xboxConfig,
    setXboxConfig,
    isLoadingSecrets,
    loading: {
      importingXbox: isImportingXbox,
      saving: isSaving,
    },
    status,
    actions: { importXboxGames, saveXboxKeys },
  };
}
