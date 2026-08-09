import { invoke } from '@tauri-apps/api/core';

import {
  ExecutableCandidate,
  GameDiscovery,
  ScanResult,
  ScanSourceInfo,
} from '@/types';

/**
 * Escaneia a pasta de jogos fornecida para descobrir jogos instalados.
 *
 * @param folderPath
 */
export async function scanGamesFolder(folderPath: string): Promise<ScanResult> {
  return await invoke<ScanResult>('scan_games_folder', { folderPath });
}

/**
 * Adiciona um único jogo descoberto pelo scan ao banco de dados.
 * Usado pelo fluxo de seleção manual de executável (ExecutableSelection).
 *
 * @param rootFolderPath - Caminho da pasta raiz onde os jogos foram encontrados
 * @param discovery - Descoberta de jogo de origem
 * @param executable - Executável escolhido dentre os candidatos da descoberta
 */
export async function addGameFromScan(
  rootFolderPath: string,
  discovery: GameDiscovery,
  executable: ExecutableCandidate
): Promise<string> {
  return await invoke<string>('add_game_from_scan', {
    rootFolderPath,
    name: discovery.suggestedName,
    executablePath: executable.path,
    basePath: discovery.basePath,
  });
}

/**
 * Adiciona múltiplos jogos descobertos pelo scan ao banco de dados.
 *
 * @param games - Lista de jogos a adicionar
 * @param rootFolderPath - Caminho da pasta raiz onde os jogos foram encontrados
 */
export async function addGamesFromScan(
  rootFolderPath: string,
  games: { name: string; executablePath: string; basePath: string }[]
): Promise<string> {
  return await invoke<string>('add_games_from_scan', { rootFolderPath, games });
}

/** Retorna o executável com a maior pontuação de ranking dentre os candidatos.
 *
 * @param discovery - Descoberta de jogo com candidatos a executáveis
 * @returns Melhor candidato a executável ou null se nenhum disponível
 */
export function getBestExecutable(
  discovery: GameDiscovery
): ExecutableCandidate | null {
  if (discovery.executables.length === 0) return null;

  return discovery.executables.reduce((best, current) =>
    current.rankScore > best.rankScore ? current : best
  );
}

/** Formata tamanho de arquivo em MB ou GB conforme apropriado.
 *
 * @param sizeMb - Tamanho do arquivo em megabytes
 * @returns Tamanho formatado como string
 */
export function formatFileSize(sizeMb: number): string {
  if (sizeMb < 1024) {
    return `${sizeMb} MB`;
  }

  return `${(sizeMb / 1024).toFixed(2)} GB`;
}

// Gerenciamento de fontes salvas
export const listScanSources = async (): Promise<ScanSourceInfo[]> => {
  return await invoke<ScanSourceInfo[]>('list_scan_sources');
};

export const renameScanSource = async (
  id: string,
  newLabel: string
): Promise<void> => {
  await invoke('rename_scan_source', { id, newLabel });
};

export const deleteScanSource = async (
  id: string,
  removeGames: boolean
): Promise<void> => {
  await invoke('delete_scan_source', { id, removeGames });
};
