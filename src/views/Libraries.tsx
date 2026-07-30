import { Library } from 'lucide-react';
import { useCallback, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { LibraryGameGrid } from '@/components';
import { useLibraryFilter, usePlaylist } from '@/hooks';
import { Game, GameActions } from '@/types';
import { toast } from '@/utils';

interface LibraryProps extends GameActions {
  games: Game[];
  searchTerm: string;
  hideAdult?: boolean;
  hideDuplicates?: boolean;
  hideNotInstalled?: boolean;
  groupByPlatform?: boolean;
}

export default function Libraries({
  games,
  searchTerm,
  hideAdult,
  hideDuplicates,
  hideNotInstalled,
  groupByPlatform = false,
  ...actions
}: Readonly<LibraryProps>) {
  const { t } = useTranslation('library');
  const { addToPlaylist, isInPlaylist } = usePlaylist(games);

  const gamesRef = useRef(games);
  useEffect(() => {
    gamesRef.current = games;
  }, [games]);

  const handleAddToPlaylist = useCallback(
    (gameId: string) => {
      const game = gamesRef.current.find(g => g.id === gameId);
      addToPlaylist(gameId);

      if (game) {
        toast.success(t('game_added_to_playlist', { name: game.name }));
      }
    },
    [addToPlaylist, t]
  );

  const displayedGames = useLibraryFilter({
    games,
    searchTerm,
    hideAdult,
    hideDuplicates,
    hideNotInstalled,
  });

  // Empty state
  if (displayedGames.length === 0) {
    return (
      <div className="text-muted-foreground flex flex-1 flex-col items-center justify-center text-lg">
        <Library className="mb-4 h-16 w-16 opacity-20" />
        {searchTerm ? t('no_games_found_search') : t('no_games_in_library')}
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden p-8">
      {/* Header da Biblioteca — fixo, não faz parte da área com scroll */}
      <div className="mb-6 flex shrink-0 items-center gap-3">
        <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
          <Library size={24} />
        </div>
        <div>
          <h1 className="text-2xl font-bold">{t('my_library_title')}</h1>
          <p className="text-muted-foreground text-sm">
            {displayedGames.length}{' '}
            {displayedGames.length === 1
              ? t('game_singular')
              : t('games_plural')}{' '}
            {displayedGames.length === 1
              ? t('found_singular')
              : t('found_plural')}
          </p>
        </div>
      </div>

      {/* Grade virtualizada — única área com scroll próprio */}
      <div className="min-h-0 flex-1">
        <LibraryGameGrid
          games={displayedGames}
          groupByPlatform={groupByPlatform}
          onGameClick={actions.onGameClick}
          onToggleFavorite={actions.onToggleFavorite}
          onAddToPlaylist={handleAddToPlaylist}
          onEditGame={actions.onEditGame}
          onDeleteGame={actions.onDeleteGame}
          isInPlaylist={isInPlaylist}
        />
      </div>
    </div>
  );
}
