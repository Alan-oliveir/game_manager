import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
  useLibraryImportTrigger,
  useLibraryStatus,
  useNativePathPicker,
} from '@/hooks';
import { settingsService } from '@/services/settingsService';
import { storesService } from '@/services/storesService.ts';
import { toast } from '@/utils/toast';

const STEAM_ROOT_KEY = 'steam_root';

/**
 * Hook para gerenciar a configuração e importação da biblioteca Steam.
 *
 * Substitui a porção "Steam" do antigo `useStoresConfig`, que carregava
 * credenciais e persistia `steamRoot` mesmo quando outra aba (Epic, Ubisoft,
 * etc.) estava aberta.
 */
export function useSteamConfig(onLibraryUpdate?: () => void) {
  const { t } = useTranslation('platforms');
  const { status, setStatus } = useLibraryStatus();

  const [steamConfig, setSteamConfig] = useState({
    steamId: '',
    steamApiKey: '',
    steamRoot: localStorage.getItem(STEAM_ROOT_KEY) || '',
  });

  const [isLoadingSecrets, setIsLoadingSecrets] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  // Carrega credenciais salvas ao montar
  useEffect(() => {
    settingsService
      .getSecrets()
      .then(data => {
        setSteamConfig(prev => ({
          ...prev,
          steamId: data.steamId || '',
          steamApiKey: data.steamApiKey || '',
        }));
      })
      .catch(e => console.error('Erro ao carregar credenciais Steam', e))
      .finally(() => setIsLoadingSecrets(false));
  }, []);

  // Persiste steamRoot localmente sempre que mudar
  useEffect(() => {
    localStorage.setItem(STEAM_ROOT_KEY, steamConfig.steamRoot);
  }, [steamConfig.steamRoot]);

  /**
   * Salva as credenciais Steam (ID e API Key) no keystore seguro.
   */
  const saveSteamKeys = async () => {
    setIsSaving(true);
    setStatus({ type: null, message: '' });

    try {
      const currentSecrets = await settingsService.getSecrets();

      await settingsService.setSecrets({
        steamId: steamConfig.steamId.trim() || null,
        steamApiKey: steamConfig.steamApiKey.trim() || null,
        rawgApiKey: currentSecrets.rawgApiKey || null,
        geminiApiKey: currentSecrets.geminiApiKey || null,
        gamebrainApiKey: currentSecrets.gamebrainApiKey || null,
        nexusApiKey: currentSecrets.nexusApiKey || null,
      });

      const successMsg = t('steam_credentials_saved');
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

  const { isImporting: isImportingSteam, run: runImportSteamLibrary } =
    useLibraryImportTrigger(
      () =>
        storesService.importSteamLibrary(
          steamConfig.steamId,
          steamConfig.steamApiKey,
          steamConfig.steamRoot
        ),
      { platformLabel: 'Steam', setStatus, onLibraryUpdate }
    );

  /**
   * Importa a biblioteca Steam, validando antes se as credenciais e o
   * diretório de instalação foram configurados.
   */
  const importSteamLibrary = async () => {
    if (!steamConfig.steamId || !steamConfig.steamApiKey) {
      const errorMsg = t('steam_missing_credentials');
      setStatus({ type: 'error', message: errorMsg });
      toast.error(errorMsg);

      return;
    }

    if (!steamConfig.steamRoot) {
      const errorMsg = t('steam_missing_directory');
      setStatus({ type: 'error', message: errorMsg });
      toast.error(errorMsg);

      return;
    }

    await runImportSteamLibrary();
  };

  const { pick: pickSteamDirectory } = useNativePathPicker({
    directory: true,
    title: t('steam_select_directory_title'),
    successMessage: t('steam_directory_selected'),
  });

  /**
   * Abre diálogo para selecionar o diretório raiz do Steam.
   */
  const chooseSteamDirectory = async () => {
    const selected = await pickSteamDirectory();

    if (selected) {
      setSteamConfig(prev => ({ ...prev, steamRoot: selected }));
    }
  };

  /**
   * Salva credenciais e importa a biblioteca numa única ação.
   */
  const saveAndImport = async () => {
    await saveSteamKeys();
    setTimeout(() => {
      void importSteamLibrary();
    }, 500);
  };

  return {
    steamConfig,
    setSteamConfig,
    isLoadingSecrets,
    loading: {
      saving: isSaving,
      importingSteam: isImportingSteam,
    },
    status,
    actions: {
      saveSteamKeys,
      importSteamLibrary,
      chooseSteamDirectory,
      saveAndImport,
    },
  };
}
