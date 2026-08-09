/**
 * Tipos relacionados para o scanner de jogos na aplicação.
 * Inclui resultados de varredura e descobertas de jogos.
 */

export interface ScanResult {
  success: boolean;
  message: string;
  discoveries: GameDiscovery[];
}

export interface GameDiscovery {
  id: string;
  basePath: string;
  executablePath: string;
  suggestedName: string;
  confidence: number;
  executables: ExecutableCandidate[];
  alreadyImported: boolean;
}

export interface ExecutableCandidate {
  path: string;
  filename: string;
  sizeMb: number;
  rankScore: number;
  executableType: 'WindowsExe' | 'LinuxElf' | 'Script' | 'Unknown';
}

// Espelha ScanSourceInfo do Rust
export interface ScanSourceInfo {
  id: string;
  folderPath: string;
  label: string;
  createdAt: string;
  lastScannedAt: string | null;
  gameCount: number;
}
