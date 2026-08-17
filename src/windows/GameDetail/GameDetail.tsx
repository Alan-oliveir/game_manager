import { useEffect, useState } from 'react';

import { WindowBase } from '@/components/wrappers/WindowBase';
import { GameEditForm } from '@/dialogs';
import { Game, GameDetails, GameLibraryLink } from '@/types/game';
import { GameHeader, GameSidebar } from '@/windows';
import { GameContentTabs } from '@/windows/GameDetail/GameWindowTabs.tsx';

interface GameDetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  game: Game | null;
  details: GameDetails | null;
  loading: boolean;
  siblings: GameLibraryLink[];
  onSwitchGame: (id: string) => void;
  onRefresh?: () => void;
}

export default function GameDetail({
  game,
  details: initialDetails,
  siblings,
  isOpen,
  onClose,
  onSwitchGame,
  onRefresh,
  loading,
}: Readonly<GameDetailsModalProps>) {
  // Estado local para manter os detalhes atualizados se houver tradução
  const [currentDetails, setCurrentDetails] = useState<GameDetails | null>(
    initialDetails
  );
  const [isEditing, setIsEditing] = useState(false);

  // Sincroniza quando os props mudam (novo jogo selecionado)
  useEffect(() => {
    setCurrentDetails(initialDetails);
    setIsEditing(false);
  }, [initialDetails, game]);

  if (!game) return null;

  const handleEditSuccess = () => {
    setIsEditing(false);

    onRefresh?.(); // Recarrega os dados do banco
  };

  const handleClose = () => {
    setIsEditing(false);
    onClose();
  };

  return (
    <WindowBase isOpen={isOpen} onClose={handleClose} maxWidth="7xl">
      {/* Header com Botão de Editar */}
      <GameHeader
        game={game}
        onClose={handleClose}
        isEditing={isEditing}
        onEditToggle={() => setIsEditing(!isEditing)}
      />

      {/* Layout Principal */}
      <div className="bg-background grid min-h-0 flex-1 grid-cols-1 gap-0 overflow-hidden lg:grid-cols-12">
        {/* Coluna 1: Sidebar (Escondida em modo edição para dar espaço ao form) */}
        {!isEditing && (
          <div className="border-border bg-muted/5 custom-scrollbar min-h-0 overflow-y-auto border-r lg:col-span-4">
            <GameSidebar
              game={game}
              details={currentDetails}
              siblings={siblings}
              onSwitchGame={onSwitchGame}
            />
          </div>
        )}

        {/* Coluna 2: Conteúdo Principal (Expandido em modo edição) */}
        <div
          className={`bg-background flex min-h-0 flex-col overflow-hidden ${isEditing ? 'lg:col-span-12' : 'lg:col-span-8'}`}
        >
          {isEditing && currentDetails ? (
            <div className="custom-scrollbar h-full overflow-y-auto p-6 lg:p-10">
              <GameEditForm
                gameId={game.id}
                details={currentDetails}
                onCancel={() => setIsEditing(false)}
                onSuccess={handleEditSuccess}
              />
            </div>
          ) : (
            <GameContentTabs
              game={game}
              details={currentDetails}
              loading={loading}
              isEditing={isEditing}
              onDescriptionUpdate={translated =>
                setCurrentDetails(prev =>
                  prev ? { ...prev, descriptionPtbr: translated } : null
                )
              }
            />
          )}
        </div>
      </div>
    </WindowBase>
  );
}
