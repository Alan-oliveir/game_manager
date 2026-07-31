import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';

import { ERROR_MESSAGES } from '@/errors/errorMessages';
import { settingsService } from '@/services/settingsService';
import { toast } from '@/utils/toast';

/**
 * Hook para gerenciar as configurações do aplicativo, incluindo chaves de API,
 * enriquecimento de metadados, backup e restauração, autenticação com serviços
 * externos e gerenciamento de cache.
 *
 * @param onLibraryUpdate - Função callback chamada quando a biblioteca é atualizada
 * @returns Objeto contendo estados, chaves e ações relacionadas às configurações
 */
export function useSettings(onLibraryUpdate: () => void) {
  const [keys, setKeys] = useState({
    rawgApiKey: '',
    geminiApiKey: '',
    gamebrainApiKey: '',
    nexusApiKey: '',
  });

  const [loading, setLoading] = useState({
    initial: true,
    saving: false,
    enriching: false,
    fetchingCovers: false,
    fillingMissing: false,
    exporting: false,
    importingBackup: false,
    authenticating: false,
    loadingCacheStats: false,
    cleaningCache: false,
    clearingAllCache: false,
    refreshingReviews: false,
    refreshingWishlistPrices: false,
  });

  const [status, setStatus] = useState<{
    type: 'success' | 'error' | null;
    message: string;
  }>({ type: null, message: '' });

  const [progress, setProgress] = useState<{
    current: number;
    total: number;
    game: string;
  } | null>(null);

  const [saveLocally, setSaveLocally] = useState(
    localStorage.getItem('config_save_covers') === 'true'
  );

  const getErrorMessage = (error: unknown, fallback: string) => {
    if (error instanceof Error && error.message) {
      return error.message;
    }

    return fallback;
  };

  // Carrega secrets ao iniciar
  useEffect(() => {
    settingsService
      .getSecrets()
      .then(data => {
        setKeys({
          rawgApiKey: data.rawgApiKey || '',
          geminiApiKey: data.geminiApiKey || '',
          gamebrainApiKey: data.gamebrainApiKey || '',
          nexusApiKey: data.nexusApiKey || '',
        });
      })
      .catch(e => console.error('Erro ao carregar settings', e))
      .finally(() => setLoading(prev => ({ ...prev, initial: false })));
  }, []);

  const saveKeys = async () => {
    setLoading(prev => ({ ...prev, saving: true }));
    setStatus({ type: null, message: '' });

    try {
      // Carrega as credenciais Steam existentes para não sobrescrever
      const currentSecrets = await settingsService.getSecrets();

      await settingsService.setSecrets({
        steamId: currentSecrets.steamId || null,
        steamApiKey: currentSecrets.steamApiKey || null,
        rawgApiKey: keys.rawgApiKey.trim() || null,
        geminiApiKey: keys.geminiApiKey.trim() || null,
        gamebrainApiKey: keys.gamebrainApiKey.trim() || null,
        nexusApiKey: keys.nexusApiKey.trim() || null,
      });
      setStatus({
        type: 'success',
        message: 'Credenciais salvas com segurança!',
      });
      toast.success('Credenciais API salvas!');
    } catch (error) {
      const errorMsg = `Erro ao salvar: ${error}`;
      setStatus({ type: 'error', message: errorMsg });
      toast.error(errorMsg);
    } finally {
      setLoading(prev => ({ ...prev, saving: false }));
    }
  };

  const enrichLibrary = async () => {
    setLoading(prev => ({ ...prev, enriching: true }));
    setStatus({
      type: null,
      message: 'Iniciando atualização de metadados...',
    });

    try {
      await settingsService.enrichLibrary();
    } catch (error) {
      setStatus({ type: 'error', message: String(error) });
      setLoading(prev => ({ ...prev, enriching: false }));
    }
  };

  const fetchMissingCovers = async () => {
    setLoading(prev => ({ ...prev, fetchingCovers: true }));
    setStatus({ type: null, message: 'Buscando capas faltantes...' });

    try {
      await settingsService.fetchMissingCovers();
    } catch (error) {
      setStatus({ type: 'error', message: String(error) });
      setLoading(prev => ({ ...prev, fetchingCovers: false }));
    }
  };

  const fillMissingMetadata = async () => {
    setLoading(prev => ({ ...prev, fillingMissing: true }));
    setStatus({ type: null, message: 'Buscando campos faltantes na RAWG...' });

    try {
      await settingsService.fillMissingMetadata();
    } catch (error) {
      setStatus({ type: 'error', message: String(error) });
      setLoading(prev => ({ ...prev, fillingMissing: false }));
    }
  };

  const exportDatabase = async () => {
    setLoading(prev => ({ ...prev, exporting: true }));
    setStatus({ type: null, message: 'Exportando backup...' });

    try {
      const msg = await settingsService.exportDatabase();
      setStatus({ type: 'success', message: msg });
    } catch (error: unknown) {
      const errorMessage = getErrorMessage(error, 'Erro ao exportar');

      if (errorMessage === ERROR_MESSAGES.CANCELLED) {
        setStatus({ type: null, message: '' });
      } else {
        setStatus({
          type: 'error',
          message: errorMessage,
        });
      }
    } finally {
      setLoading(prev => ({ ...prev, exporting: false }));
    }
  };

  const importDatabase = async () => {
    setLoading(prev => ({ ...prev, importingBackup: true }));
    setStatus({ type: null, message: 'Importando backup...' });

    try {
      const msg = await settingsService.importDatabase();
      setStatus({ type: 'success', message: msg });
      onLibraryUpdate();
    } catch (error: unknown) {
      const errorMessage = getErrorMessage(error, 'Erro ao importar');

      if (errorMessage === ERROR_MESSAGES.CANCELLED) {
        setStatus({ type: null, message: '' });
      } else {
        setStatus({
          type: 'error',
          message: errorMessage,
        });
      }
    } finally {
      setLoading(prev => ({ ...prev, importingBackup: false }));
    }
  };

  const cleanupCache = async () => {
    setLoading(prev => ({ ...prev, cleaningCache: true }));
    setStatus({ type: null, message: 'Limpando cache expirado...' });

    try {
      const msg = await settingsService.cleanupCache();
      setStatus({ type: 'success', message: msg });
      toast.success(msg || 'Cache expirado limpo com sucesso!');
    } catch (error) {
      const errorMsg = String(error);
      setStatus({ type: 'error', message: errorMsg });
      toast.error(errorMsg);
    } finally {
      setLoading(prev => ({ ...prev, cleaningCache: false }));
    }
  };

  const clearAllCache = async () => {
    setLoading(prev => ({ ...prev, clearingAllCache: true }));
    setStatus({ type: null, message: 'Limpando todo o cache...' });

    try {
      const msg = await settingsService.clearAllCache();
      setStatus({ type: 'success', message: msg });
      toast.success(msg || 'Todo o cache foi limpo com sucesso!');
    } catch (error) {
      const errorMsg = String(error);
      setStatus({ type: 'error', message: errorMsg });
      toast.error(errorMsg);
    } finally {
      setLoading(prev => ({ ...prev, clearingAllCache: false }));
    }
  };

  const toggleSaveLocally = (checked: boolean) => {
    setSaveLocally(checked);
    localStorage.setItem('config_save_covers', String(checked));
    toast.success(`Modo offline ${checked ? 'ativado' : 'desativado'}`);
  };

  const handleClearCache = async () => {
    try {
      await invoke('clear_cover_cache');
      toast.success('Espaço liberado! Imagens locais removidas.');
    } catch {
      toast.error('Erro ao limpar cache.');
    }
  };

  const updateLoadingForEnrichProgress = (isCoverTask: boolean) => {
    setLoading(prev => ({
      ...prev,
      enriching: !isCoverTask,
      fetchingCovers: isCoverTask,
    }));
  };

  const finishEnrichment = () => {
    setLoading(prev => ({
      ...prev,
      enriching: false,
      fetchingCovers: false,
      fillingMissing: false,
    }));
  };

  const updateLoadingForRefreshProgress = (
    refreshType: 'reviews' | 'prices'
  ) => {
    setLoading(prev => ({
      ...prev,
      refreshingReviews: refreshType === 'reviews',
      refreshingWishlistPrices: refreshType === 'prices',
    }));
  };

  const finishReviewsRefresh = () => {
    setLoading(prev => ({ ...prev, refreshingReviews: false }));
  };

  const finishWishlistRefresh = () => {
    setLoading(prev => ({ ...prev, refreshingWishlistPrices: false }));
  };

  // Listeners para eventos de enriquecimento
  useEffect(() => {
    const handleEnrichProgress = (event: {
      payload: { current: number; total_found: number; last_game: string };
    }) => {
      const p = event.payload;
      setProgress({
        current: p.current,
        total: p.total_found,
        game: p.last_game,
      });

      updateLoadingForEnrichProgress(p.last_game.startsWith('Capa:'));
    };

    const handleEnrichComplete = () => {
      finishEnrichment();
      setProgress(null);
      setStatus({
        type: 'success',
        message: 'Processo concluído com sucesso!',
      });
      onLibraryUpdate();
    };

    const handleRefreshProgress = (event: {
      payload: {
        current: number;
        total: number;
        item_name: string;
        refresh_type: 'reviews' | 'prices';
      };
    }) => {
      const p = event.payload;
      setProgress({
        current: p.current,
        total: p.total,
        game: p.item_name,
      });

      updateLoadingForRefreshProgress(p.refresh_type);
    };

    const handleReviewsRefreshComplete = (event: { payload: string }) => {
      finishReviewsRefresh();
      setProgress(null);
      toast.info(String(event.payload), { duration: 4000 });
    };

    const handleWishlistRefreshComplete = (event: { payload: string }) => {
      finishWishlistRefresh();
      setProgress(null);
      toast.info(String(event.payload), { duration: 4000 });
    };

    let cleanup = () => {};
    let isActive = true;

    const setupListeners = async () => {
      const [
        unlistenProgress,
        unlistenComplete,
        unlistenRefreshProgress,
        unlistenReviewsComplete,
        unlistenWishlistComplete,
      ] = await Promise.all([
        listen('enrich_progress', handleEnrichProgress),
        listen('enrich_complete', handleEnrichComplete),
        listen('refresh_progress', handleRefreshProgress),
        listen('reviews_refresh_complete', handleReviewsRefreshComplete),
        listen('wishlist_refresh_complete', handleWishlistRefreshComplete),
      ]);

      const currentCleanup = () => {
        unlistenProgress();
        unlistenComplete();
        unlistenRefreshProgress();
        unlistenReviewsComplete();
        unlistenWishlistComplete();
      };

      if (!isActive) {
        currentCleanup();

        return;
      }

      cleanup = currentCleanup;
    };

    void setupListeners();

    return () => {
      isActive = false;
      cleanup();
    };
  }, [onLibraryUpdate]);

  // Auto-close status messages
  useEffect(() => {
    if (status.type && status.message) {
      const timer = setTimeout(() => {
        setStatus({ type: null, message: '' });
      }, 5000);

      return () => clearTimeout(timer);
    }
  }, [status]);

  return {
    keys,
    setKeys,
    loading,
    status,
    progress,
    saveLocally,
    toggleSaveLocally,
    handleClearCache,
    actions: {
      saveKeys,
      enrichLibrary,
      fetchMissingCovers,
      fillMissingMetadata,
      exportDatabase,
      importDatabase,
      cleanupCache,
      clearAllCache,
    },
  };
}
