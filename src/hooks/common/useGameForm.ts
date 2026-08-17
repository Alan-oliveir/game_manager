import { useEffect, useState } from 'react';

import { Game } from '@/types';

export interface GameFormData {
  name: string;
  coverUrl: string;
  library: string;
  libraryGameId: string;
  installed: boolean;
  importConfidence: string;
  status: string;
  playtime: string;
  rating: number;
  installPath: string;
  executablePath: string;
  launchArgs: string;
}

const INITIAL_STATE: GameFormData = {
  name: '',
  coverUrl: '',
  library: 'Manual',
  libraryGameId: '',
  installed: false,
  importConfidence: '',
  status: 'backlog',
  playtime: '0',
  rating: 0,
  installPath: '',
  executablePath: '',
  launchArgs: '',
};

/**
 * Hook para gerenciar o formulário de adicionar/editar jogo.
 *
 * Responsabilidades:
 * - Gerencia estado do formulário
 * - Inicializa dados baseado em jogo existente (edição) ou vazio (novo)
 * - Valida e transforma dados para salvamento
 * - Reseta formulário quando modal abre/fecha
 *
 * @param isOpen - Se o modal está aberto
 * @param gameToEdit - Jogo a ser editado (opcional)
 * @returns Estado e ações do formulário
 */
export function useGameForm(isOpen: boolean, gameToEdit?: Game | null) {
  const [formData, setFormData] = useState<GameFormData>(INITIAL_STATE);

  // Inicializa/reseta formulário quando modal abre
  useEffect(() => {
    if (isOpen) {
      if (gameToEdit) {
        // Modo edição: preenche com dados do jogo
        setFormData({
          name: gameToEdit.name,
          coverUrl: gameToEdit.coverUrl || '',
          library: gameToEdit.library || 'Manual',
          libraryGameId: gameToEdit.libraryGameId || '',
          installed: gameToEdit.installed || false,
          importConfidence: gameToEdit.importConfidence || '',
          status: gameToEdit.status || 'backlog',
          playtime: gameToEdit.playtime?.toString() || '0',
          rating: gameToEdit.userRating || 0,
          installPath: gameToEdit.installPath || '',
          executablePath: gameToEdit.executablePath || '',
          launchArgs: gameToEdit.launchArgs || '',
        });
      } else {
        // Modo criação: reseta para estado inicial
        setFormData(INITIAL_STATE);
      }
    }
  }, [isOpen, gameToEdit]);

  // Atualiza um campo do formulário
  const handleChange = <K extends keyof GameFormData>(
    field: K,
    value: GameFormData[K]
  ) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  // Valida se o formulário está pronto para salvar
  const isValid = () => {
    return formData.name.trim().length > 0;
  };

  // Transforma dados do formulário para o formato do Game
  const buildPayload = () => {
    return {
      // Mantém ID se estiver editando
      id: gameToEdit?.id,
      name: formData.name,
      coverUrl: formData.coverUrl || undefined,
      library: formData.library,
      libraryGameId: formData.libraryGameId || '',
      installed: formData.installed,
      importConfidence: formData.importConfidence || undefined,
      status: formData.status,
      playtime: Number.parseInt(formData.playtime, 10) || 0,
      userRating: formData.rating > 0 ? formData.rating : undefined,
      installPath: formData.installPath || undefined,
      executablePath: formData.executablePath || undefined,
      launchArgs: formData.launchArgs || undefined,
    };
  };

  // Reseta formulário para estado inicial
  const reset = () => {
    setFormData(INITIAL_STATE);
  };

  return {
    formData,
    handleChange,
    isValid,
    buildPayload,
    reset,
  };
}
