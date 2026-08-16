import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import { Game, GameDetails, GameStoreLink } from 'src/types';

/**
 * Hook para gerenciar os detalhes de um jogo selecionado, incluindo o carregamento
 * de informações locais e a identificação de versões em outras plataformas.
 *
 * @param selectedGame - Jogo atualmente selecionado
 * @param allGames - Lista completa de jogos para identificar versões relacionadas
 * @returns Objeto contendo detalhes do jogo, estado de carregamento, versões relacionadas e função para recarregar dados
 */
export function useGameDetails(selectedGame: Game | null, allGames: Game[]) {
  const [details, setDetails] = useState<GameDetails | null>(null);
  const [loading, setLoading] = useState(false);
  const [siblings, setSiblings] = useState<GameStoreLink[]>([]);

  // Move loadData outside useEffect so it can be returned
  const loadData = async () => {
    if (!selectedGame) {
      setDetails(null);

      return;
    }

    setLoading(true);

    try {
      const localData = await invoke<GameDetails>('get_library_game_details', {
        gameId: selectedGame.id,
      });
      // Se encontrou, define os detalhes; senão, define como null
      setDetails(localData || null);
    } catch (err) {
      console.error('Erro ao carregar detalhes locais:', err);
      setDetails(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    // Se nenhum jogo foi selecionado, limpa o estado
    if (!selectedGame) {
      setDetails(null);

      return;
    }

    // 1. Identifica versões em outras plataformas (Siblings)
    const related = allGames
      .filter(
        g =>
          g.name.toLowerCase() === selectedGame.name.toLowerCase() &&
          g.id !== selectedGame.id
      )
      .map(g => ({ id: g.id, platform: g.platform || 'Outra' }));
    setSiblings(related);
    // 2. Busca detalhes do banco de dados local
    loadData();
  }, [selectedGame, allGames]);

  return { details, loading, siblings, refresh: loadData };
}
