import { Heart } from 'lucide-react';
import { memo, useCallback, useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import { ActionButton, GameActionsMenu } from '@/components/common';
import { useLaunchGame } from '@/hooks/library';
import { Game, LibraryDisplayNames } from '@/types';

import StandardGameCard from './StandardGameCard';

interface LibraryGameCardProps {
  game: Game;
  onGameClick: (game: Game) => void;
  onToggleFavorite: (id: string) => void;
  onAddToPlaylist: (id: string) => void;
  onEditGame: (game: Game) => void;
  onDeleteGame: (id: string) => void;
  isInPlaylist: (id: string) => boolean;
}

/**
 * Card de jogo usado nas grades de Biblioteca e Favoritos (que compartilham
 * praticamente o mesmo card e as mesmas ações). Extraído para consolidar a
 * duplicação entre `Libraries.tsx`/`Favorites.tsx` e, principalmente, para
 * ser o único lugar responsável por estabilizar os callbacks — sem isso, o
 * `React.memo` do `StandardGameCard` não evita re-renders de verdade.
 *
 * IMPORTANTE: as props de função (`onGameClick`, `onToggleFavorite`,
 * `onAddToPlaylist`, `onEditGame`, `onDeleteGame`, `isInPlaylist`) precisam
 * ser estáveis entre renders no componente pai (useCallback) para que a
 * memoização funcione — se o pai passar uma função nova a cada render,
 * o `useCallback`/`useMemo` aqui dentro recalcula mesmo assim.
 *
 * A referência de `game` só muda para o item que de fato foi alterado
 * (ex: favoritar um jogo específico atualiza só aquele objeto no array),
 * então os demais cards da grade não re-renderizam nessas ações.
 */
export const LibraryGameCard = memo(function LibraryGameCard({
  game,
  onGameClick,
  onToggleFavorite,
  onAddToPlaylist,
  onEditGame,
  onDeleteGame,
  isInPlaylist,
}: Readonly<LibraryGameCardProps>) {
  const { t } = useTranslation('library');
  const { launchGame } = useLaunchGame();

  const subtitle =
    [game.genres?.split(',')[0]?.trim(), game.developer]
      .filter(Boolean)
      .join(' • ') || t('no_data_fallback');

  const handleClick = useCallback(() => onGameClick(game), [onGameClick, game]);

  const handlePlay = useCallback(() => launchGame(game), [launchGame, game]);

  const handleToggleFavorite = useCallback(
    () => onToggleFavorite(game.id),
    [onToggleFavorite, game.id]
  );

  // O JSX de `actions` também precisa de referência estável — React.memo
  // faz comparação rasa, e um JSX "igual" recriado a cada render ainda
  // conta como uma prop diferente.
  const actionsNode = useMemo(
    () => (
      <>
        <ActionButton
          icon={Heart}
          variant={game.favorite ? 'glass-destructive' : 'glass'}
          tooltip={
            game.favorite
              ? t('remove_from_favorites_tooltip')
              : t('add_to_favorites_tooltip')
          }
          onClick={handleToggleFavorite}
        />
        <GameActionsMenu
          game={game}
          inPlaylist={isInPlaylist(game.id)}
          onAddToPlaylist={onAddToPlaylist}
          onEdit={onEditGame}
          onDelete={onDeleteGame}
        />
      </>
    ),
    [
      game,
      handleToggleFavorite,
      isInPlaylist,
      onAddToPlaylist,
      onEditGame,
      onDeleteGame,
      t,
    ]
  );

  return (
    <div className="group relative">
      <StandardGameCard
        id={game.id.toString()}
        title={game.name}
        coverUrl={game.coverUrl}
        platform={LibraryDisplayNames[game.platform]}
        sourceLabel={game.sourceLabel}
        subtitle={subtitle}
        rating={game.userRating || undefined}
        onClick={handleClick}
        onPlay={handlePlay}
        actions={actionsNode}
      />
    </div>
  );
});
