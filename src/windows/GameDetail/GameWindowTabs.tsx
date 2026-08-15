import { Blocks, BookOpen, Compass, Play, Wrench } from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Game, GameDetails } from '@/types/game';
import {
  GameAddons,
  GameDescription,
  GameDiscovery,
  GameExtras,
  GameMedia,
} from '@/windows';

// === TIPOS ===

// Tipo das abas disponíveis
type GameTab = 'description' | 'discovery' | 'addons' | 'media' | 'extras';

interface Tab {
  id: GameTab;
  label: string;
  icon: React.ReactNode;
}

// === COMPONENTE DE ABAS ===

interface GameTabsProps {
  activeTab: GameTab;
  onTabChange: (tab: GameTab) => void;
}

export function GameTabs({ activeTab, onTabChange }: GameTabsProps) {
  const { t } = useTranslation('game_detail');
  const tabs: Tab[] = [
    {
      id: 'description',
      label: t('window_tabs_tab_description'),
      icon: <BookOpen />,
    },
    {
      id: 'discovery',
      label: t('window_tabs_tab_discovery'),
      icon: <Compass />,
    },
    { id: 'addons', label: t('window_tabs_tab_addons'), icon: <Blocks /> },
    { id: 'media', label: t('window_tabs_tab_media'), icon: <Play /> },
    { id: 'extras', label: t('window_tabs_tab_extras'), icon: <Wrench /> },
  ];

  return (
    <div className="border-border flex items-center gap-1 border-b">
      {tabs.map(tab => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className={`flex items-center gap-1 border-b-2 px-4 py-2 text-sm font-medium transition-all duration-150 focus:outline-none ${
            activeTab === tab.id
              ? 'border-primary text-primary bg-primary/5'
              : 'text-muted-foreground hover:text-foreground hover:bg-muted/40 border-transparent'
          } `}
        >
          <span
            className={
              activeTab === tab.id ? 'text-primary' : 'text-muted-foreground'
            }
          >
            {tab.icon}
          </span>
          {tab.label}
        </button>
      ))}
    </div>
  );
}

// === COMPONENTE ORQUESTRADOR ===

interface GameContentTabsProps {
  game: Game;
  details: GameDetails | null;
  loading: boolean;
  isEditing: boolean;
  onDescriptionUpdate: React.ComponentProps<
    typeof GameDescription
  >['onDescriptionUpdate'];
}

export function GameContentTabs({
  game,
  details,
  loading,
  isEditing,
  onDescriptionUpdate,
}: GameContentTabsProps) {
  const [activeTab, setActiveTab] = useState<GameTab>('description');

  // Reset para Descrição quando abrir um novo jogo
  useEffect(() => {
    setActiveTab('description');
  }, [game.id]);

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Só mostra as abas fora do modo edição */}
      {!isEditing && (
        <GameTabs activeTab={activeTab} onTabChange={setActiveTab} />
      )}

      {/* Conteúdo da aba ativa */}
      <div className="custom-scrollbar min-h-0 flex-1 overflow-y-auto p-6 lg:p-10">
        {activeTab === 'description' && (
          <GameDescription
            gameId={game.id}
            details={details}
            loading={loading}
            onDescriptionUpdate={onDescriptionUpdate}
          />
        )}
        {activeTab === 'discovery' && !isEditing && (
          <GameDiscovery game={game} />
        )}
        {activeTab === 'addons' && !isEditing && <GameAddons game={game} />}
        {activeTab === 'media' && !isEditing && <GameMedia game={game} />}
        {activeTab === 'extras' && !isEditing && (
          <GameExtras game={game} details={details} />
        )}
      </div>
    </div>
  );
}
