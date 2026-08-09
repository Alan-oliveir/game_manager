import { open } from '@tauri-apps/plugin-dialog';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
  addGamesFromScan,
  getBestExecutable,
  scanGamesFolder,
} from '@/services/scannerService';
import { ScanResult } from '@/types/scanner';
import { toast } from '@/utils/toast';

/** Hook para gerenciar o estado e lógica de escaneamento de pasta de jogos.
 *
 * @returns Estado e funções para escanear a pasta de jogos
 */
export function useScanner() {
  const { t } = useTranslation('platforms');
  const [scanning, setScanning] = useState(false);
  const [result, setResult] = useState<ScanResult | null>(null);
  const [selectedFolder, setSelectedFolder] = useState<string>('');

  const handleSelectFolder = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Selecione a pasta de jogos',
    });

    if (selected) {
      setSelectedFolder(selected);
    }
  };

  const handleScan = async () => {
    if (!selectedFolder) return;

    setScanning(true);
    // Limpa o resultado anterior para não deixar dados obsoletos na tela caso o novo scan falhe.
    setResult(null);

    try {
      const scanResult = await scanGamesFolder(selectedFolder);
      setResult(scanResult);
    } catch (error) {
      toast.error(t('scanner_scan_failed'));
      console.error('Erro ao escanear:', error);
    } finally {
      setScanning(false);
    }
  };

  const handleAddAll = async () => {
    if (!result?.discoveries.length) return;

    try {
      const games = result.discoveries
        .filter(d => !d.alreadyImported) // Filtra apenas jogos que ainda não foram importados
        .map(d => {
          const best = getBestExecutable(d);

          if (!best) return null;

          return {
            name: d.suggestedName,
            executablePath: best.path,
            basePath: d.basePath,
          };
        })
        .filter(
          (
            game
          ): game is {
            name: string;
            executablePath: string;
            basePath: string;
          } => game !== null
        );

      if (games.length === 0) {
        toast.info(t('scanner_no_new_games'));

        return;
      }

      const message = await addGamesFromScan(selectedFolder, games);
      setResult({ ...result, message });
    } catch (error) {
      toast.error(t('scanner_add_all_failed'));
      console.error('Erro ao adicionar jogos:', error);
    }
  };

  return {
    scanning,
    result,
    selectedFolder,
    handleSelectFolder,
    handleScan,
    handleAddAll,
  };
}
