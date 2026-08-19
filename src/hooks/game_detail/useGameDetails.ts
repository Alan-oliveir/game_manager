import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import {
  CloudAvailability,
  Game,
  GameDetails,
  GameLibraryLink,
} from 'src/types';

/**
 * Hook para gerenciar os detalhes de um jogo selecionado, incluindo o carregamento
 * de informações locais, disponibilidade em cloud gaming, e a identificação de
 * versões em outras plataformas.
 *
 * @param selectedGame - Jogo atualmente selecionado
 * @param allGames - Lista completa de jogos para identificar versões relacionadas
 * @returns Objeto contendo detalhes do jogo, estado de carregamento, versões relacionadas,
 *          disponibilidade em cloud gaming e função para recarregar dados
 */
export function useGameDetails(selectedGame: Game | null, allGames: Game[]) {
  const [details, setDetails] = useState<GameDetails | null>(null);
  const [loading, setLoading] = useState(false);
  const [siblings, setSiblings] = useState<GameLibraryLink[]>([]);
  const [cloudAvailability, setCloudAvailability] =
    useState<CloudAvailability | null>(null);

  // Move loadData outside useEffect so it can be returned
  const loadData = async () => {
    if (!selectedGame) {
      setDetails(null);
      setCloudAvailability(null);

      return;
    }

    setLoading(true);

    try {
      const localData = await invoke<GameDetails>('get_library_game_details', {
        gameId: selectedGame.id,
      });
      // Se encontrou, define os detalhes; senão, define como null
      setDetails(localData || null);

      // Cloud gaming é buscado sob demanda, separado do resto — uma falha (ex. catálogo Xbox indisponível) não derruba a tela inteira.
      try {
        const cloud = await invoke<CloudAvailability>(
          'get_cloud_gaming_availability',
          {
            gameName: selectedGame.name,
            library: selectedGame.library,
            libraryGameId: selectedGame.libraryGameId,
            steamAppId: localData?.steamAppId ?? null,
          }
        );
        setCloudAvailability(cloud);
      } catch (err) {
        console.error('Erro ao carregar disponibilidade em cloud gaming:', err);
        setCloudAvailability(null);
      }
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
      setCloudAvailability(null);

      return;
    }

    // 1. Identifica versões em outras plataformas (Siblings)
    const related = allGames
      .filter(
        g =>
          g.name.toLowerCase() === selectedGame.name.toLowerCase() &&
          g.id !== selectedGame.id
      )
      .map(g => ({ id: g.id, library: g.library || 'Outra' }));
    setSiblings(related);
    // 2. Busca detalhes do banco de dados local (e, dentro dela, cloud availability)
    loadData();
  }, [selectedGame, allGames]);

  return { details, loading, siblings, cloudAvailability, refresh: loadData };
}
