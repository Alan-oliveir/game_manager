import { useCallback } from 'react';

import { toast } from '@/utils/toast';

interface DialogFilter {
  name: string;
  extensions: string[];
}

interface UseNativePathPickerOptions {
  /** Se true (padrão), abre seletor de diretório. Se false, seletor de arquivo. */
  directory?: boolean;
  title: string;
  filters?: DialogFilter[];
  successMessage?: string;
  errorMessage?: string;
}

/**
 * Abre o diálogo nativo do Tauri (`@tauri-apps/plugin-dialog`) para
 * selecionar um diretório ou arquivo.
 *
 * Extraído porque a mesma lógica (import dinâmico do plugin, chamada de
 * `open`, tratamento de erro) estava duplicada em `chooseSteamDirectory`
 * (useStoresConfig), `handleChooseDir` (HeroicSettings), `handleChooseFile`
 * (LegacySettings) e `chooseWinePrefix` (useWineConfig).
 *
 * Retorna apenas o caminho selecionado (ou `null`); cada plataforma decide
 * o que fazer com ele (ex: salvar em state local, persistir em localStorage).
 */
export function useNativePathPicker({
  directory = true,
  title,
  filters,
  successMessage,
  errorMessage = 'Erro ao selecionar diretório',
}: UseNativePathPickerOptions) {
  const pick = useCallback(async (): Promise<string | null> => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        directory,
        multiple: false,
        title,
        ...(filters ? { filters } : {}),
      });

      if (selected && typeof selected === 'string') {
        if (successMessage) toast.success(successMessage);

        return selected;
      }

      return null;
    } catch (error) {
      console.error('Erro ao selecionar caminho:', error);
      toast.error(errorMessage);

      return null;
    }
  }, [directory, errorMessage, filters, successMessage, title]);

  return { pick };
}
