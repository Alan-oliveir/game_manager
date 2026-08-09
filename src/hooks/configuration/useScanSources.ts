import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
  deleteScanSource,
  listScanSources,
  renameScanSource,
} from '@/services/scannerService';
import { ScanSourceInfo } from '@/types/scanner';
import { toast } from '@/utils/toast';

/**
 * Hook para gerenciar as fontes de scan salvas (pastas locais rastreadas),
 * permitindo listar, renomear e apagar.
 */
export function useScanSources() {
  const { t } = useTranslation('platforms');
  const [sources, setSources] = useState<ScanSourceInfo[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);

    try {
      const data = await listScanSources();
      setSources(data);
    } catch (error) {
      toast.error(t('scanner_sources_load_failed'));
      console.error('Erro ao carregar fontes de scan:', error);
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const rename = async (id: string, newLabel: string) => {
    const trimmed = newLabel.trim();

    if (!trimmed) return;

    try {
      await renameScanSource(id, trimmed);
      toast.success(t('scanner_source_renamed'));
      await refresh();
    } catch (error) {
      toast.error(
        typeof error === 'string' ? error : t('scanner_source_rename_failed')
      );
      console.error('Erro ao renomear fonte:', error);
    }
  };

  const remove = async (id: string, removeGames: boolean) => {
    try {
      await deleteScanSource(id, removeGames);
      toast.success(t('scanner_source_deleted'));
      await refresh();
    } catch (error) {
      toast.error(
        typeof error === 'string' ? error : t('scanner_source_delete_failed')
      );
      console.error('Erro ao apagar fonte:', error);
    }
  };

  return { sources, loading, refresh, rename, remove };
}
