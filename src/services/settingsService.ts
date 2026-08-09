import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

import { ERROR_MESSAGES, parseBackupError } from '@/errors/errorMessages';
import { KeysBatch } from '@/types';

const getErrorMessage = (error: unknown) =>
  error instanceof Error ? error.message : String(error);

export const settingsService = {
  /**
   * Obtém as chaves de API salvas para usar em serviços externos.
   */
  getSecrets: async (): Promise<KeysBatch> => {
    return await invoke<KeysBatch>('get_secrets');
  },

  /**
   * Configura e salva chaves de API usadas na aplicação para acessar serviços externos.
   *
   * @param keys
   */
  setSecrets: async (keys: {
    steamId: string | null;
    steamApiKey: string | null;
    rawgApiKey: string | null;
    geminiApiKey: string | null;
    gamebrainApiKey: string | null;
    nexusApiKey: string | null;
    igdbClientId: string | null;
    igdbClientSecret: string | null;
    xboxLiveClientId: string | null;
    xboxLiveClientSecret: string | null;
  }): Promise<void> => {
    await invoke('set_secrets', keys);
  },

  /**
   * Busca capas faltantes para jogos na biblioteca.
   * Pode demorar dependendo do número de capas faltantes.
   * Utiliza RAWG como fonte.
   */
  fetchMissingCovers: async (): Promise<void> => {
    await invoke('fetch_missing_covers');
  },

  /**
   * Preenche campos de metadados vazios para todos os jogos da biblioteca.
   *
   * Ignora o cache da RAWG para garantir dados atualizados, mas nunca
   * sobrescreve campos que já possuem valor — apenas preenche lacunas.
   *
   * Útil após importações parciais (ex: Legacy Games) ou quando a RAWG
   * atualizou o catálogo depois da última sincronização.
   */
  fillMissingMetadata: async (): Promise<void> => {
    await invoke('fill_missing_metadata');
  },

  /**
   * Exporta toda a biblioteca para JSON.
   * Abre diálogo nativo para escolher local de salvamento.
   *
   * @throws Se o usuário cancelar ou não tiver permissões de escrita
   */
  exportDatabase: async (): Promise<string> => {
    try {
      const filePath = await save({
        defaultPath: `playlite-backup-${new Date().toISOString().split('T')[0]}.json`,
        filters: [
          {
            name: 'JSON',
            extensions: ['json'],
          },
        ],
      });

      if (!filePath) {
        throw new Error(ERROR_MESSAGES.CANCELLED);
      }

      await invoke('export_database', { filePath });

      return 'Backup exportado com sucesso!';
    } catch (error: unknown) {
      const errorMessage = getErrorMessage(error);

      if (errorMessage === ERROR_MESSAGES.CANCELLED) {
        throw new Error(ERROR_MESSAGES.CANCELLED);
      }

      const friendlyError = parseBackupError(error);

      if (friendlyError === errorMessage) {
        throw new Error(ERROR_MESSAGES.BACKUP_EXPORT_FAILED);
      }

      throw new Error(friendlyError);
    }
  },

  /**
   * Importa biblioteca de um backup JSON.
   * SOBRESCREVE todos os dados existentes sem confirmação.
   * Valida formato antes de importar.
   *
   * @throws Se o arquivo for inválido ou corrompido
   */
  importDatabase: async (): Promise<string> => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'JSON',
            extensions: ['json'],
          },
        ],
      });

      if (!selected) {
        throw new Error(ERROR_MESSAGES.CANCELLED);
      }

      const filePath = selected;

      return await invoke<string>('import_database', { filePath });
    } catch (error: unknown) {
      const errorMessage = getErrorMessage(error);

      if (errorMessage === ERROR_MESSAGES.CANCELLED) {
        throw new Error(ERROR_MESSAGES.CANCELLED);
      }

      const friendlyError = parseBackupError(error);

      if (friendlyError === errorMessage) {
        throw new Error(ERROR_MESSAGES.BACKUP_IMPORT_FAILED);
      }

      throw new Error(friendlyError);
    }
  },

  /**
   * Remove apenas entradas expiradas do cache.
   * Mantém dados ainda válidos baseado nos TTLs configurados.
   */
  cleanupCache: async (): Promise<string> => {
    return await invoke<string>('cleanup_cache');
  },

  /**
   * Remove completamente o cache de metadados.
   * Use com cuidado - força nova busca em APIs na próxima atualização.
   */
  clearAllCache: async (): Promise<string> => {
    return await invoke<string>('clear_all_cache');
  },
};
