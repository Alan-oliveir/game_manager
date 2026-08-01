import { CheckCircle2, Settings2, Wine } from 'lucide-react';
import type { ComponentType } from 'react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
  Amazon,
  BattleNet,
  Ea,
  Epic,
  Gog,
  Indiegala,
  Itch,
  Legacy,
  Steam,
  Ubisoft,
  Xbox,
} from '@/components/icons/logos';
import { WindowBase } from '@/components/wrappers/WindowBase';
import { useImportProgress } from '@/hooks';
import { cn } from '@/lib/utils';
import {
  AmazonGamesSettings,
  BattleNetSettings,
  EaSettings,
  EpicGamesSettings,
  GogSettings,
  IndiegalaSettings,
  ItchSettings,
  LegacySettings,
  LocalScannerSettings,
  SteamSettings,
  UbisoftSettings,
  WineSettings,
  XboxSettings,
} from '@/windows';

type SourceProvider =
  | 'steam'
  | 'epic'
  | 'ubisoft'
  | 'legacy'
  | 'gog'
  | 'battlenet'
  | 'ea'
  | 'amazon'
  | 'xbox'
  | 'indiegala'
  | 'itch'
  | 'local'
  | 'wine';

type IconComponent = ComponentType<{ size?: number; className?: string }>;

export default function PlatformsConfig({
  isOpen,
  onClose,
  onLibraryUpdate,
}: Readonly<{
  isOpen: boolean;
  onClose: () => void;
  onLibraryUpdate?: () => void;
}>) {
  const { t } = useTranslation('platforms');
  const [activeStore, setActiveStore] = useState<SourceProvider>('steam');
  const { progress } = useImportProgress();

  const stores: {
    id: SourceProvider;
    name: string;
    Icon: IconComponent;
    connected: boolean;
  }[] = [
    {
      id: 'steam',
      name: t('config_store_steam'),
      Icon: Steam,
      connected: true,
    },
    { id: 'epic', name: t('config_store_epic'), Icon: Epic, connected: true },
    {
      id: 'ubisoft',
      name: t('config_store_ubisoft'),
      Icon: Ubisoft,
      connected: true,
    },
    {
      id: 'legacy',
      name: t('config_store_legacy'),
      Icon: Legacy,
      connected: true,
    },
    {
      id: 'gog',
      name: t('config_store_gog'),
      Icon: Gog,
      connected: true,
    },
    {
      id: 'battlenet',
      name: t('config_store_battlenet'),
      Icon: BattleNet,
      connected: true,
    },
    {
      id: 'ea',
      name: t('config_store_ea'),
      Icon: Ea,
      connected: true,
    },
    {
      id: 'amazon',
      name: t('config_store_amazon'),
      Icon: Amazon,
      connected: true,
    },
    {
      id: 'xbox',
      name: t('config_store_xbox'),
      Icon: Xbox,
      connected: true,
    },
    {
      id: 'indiegala',
      name: t('config_store_indiegala'),
      Icon: Indiegala,
      connected: true,
    },
    {
      id: 'itch',
      name: t('config_store_itch'),
      Icon: Itch,
      connected: true,
    },
    {
      id: 'local',
      name: t('config_store_local'),
      Icon: Settings2,
      connected: true,
    },
    { id: 'wine', name: t('config_store_wine'), Icon: Wine, connected: true },
  ];

  const renderActiveStore = () => {
    switch (activeStore) {
      case 'steam':
        return (
          <SteamSettings
            onLibraryUpdate={onLibraryUpdate}
            progress={progress}
          />
        );
      case 'epic':
        return (
          <EpicGamesSettings
            onLibraryUpdate={onLibraryUpdate}
            progress={progress}
          />
        );
      case 'ubisoft':
        return (
          <UbisoftSettings
            onLibraryUpdate={onLibraryUpdate}
            progress={progress}
          />
        );
      case 'legacy':
        return (
          <LegacySettings
            onLibraryUpdate={onLibraryUpdate}
            progress={progress}
          />
        );
      case 'gog':
        return (
          <GogSettings onLibraryUpdate={onLibraryUpdate} progress={progress} />
        );
      case 'battlenet':
        return (
          <BattleNetSettings
            onLibraryUpdate={onLibraryUpdate}
            progress={progress}
          />
        );
      case 'ea':
        return (
          <EaSettings onLibraryUpdate={onLibraryUpdate} progress={progress} />
        );
      case 'amazon':
        return (
          <AmazonGamesSettings
            onLibraryUpdate={onLibraryUpdate}
            progress={progress}
          />
        );
      case 'xbox':
        return (
          <XboxSettings onLibraryUpdate={onLibraryUpdate} progress={progress} />
        );
      case 'indiegala':
        return (
          <IndiegalaSettings
            onLibraryUpdate={onLibraryUpdate}
            progress={progress}
          />
        );
      case 'itch':
        return (
          <ItchSettings onLibraryUpdate={onLibraryUpdate} progress={progress} />
        );
      case 'local':
        return <LocalScannerSettings />;
      case 'wine':
        return <WineSettings />;
      default:
        return null;
    }
  };

  return (
    <WindowBase isOpen={isOpen} onClose={onClose} maxWidth="7xl">
      <div className="flex h-full flex-row">
        {/* Sidebar */}
        <aside className="border-border/40 bg-secondary/10 w-64 border-r p-6">
          <h2 className="text-muted-foreground mb-6 text-xs font-bold tracking-widest uppercase">
            {t('config_sources_title')}
          </h2>
          <nav className="space-y-1">
            {stores.map(store => {
              const StoreIcon = store.Icon;
              const isActive = activeStore === store.id;

              return (
                <button
                  key={store.id}
                  onClick={() => setActiveStore(store.id)}
                  className={cn(
                    'flex w-full items-center justify-between rounded-lg px-3 py-2.5 text-sm transition-colors',
                    isActive
                      ? 'bg-secondary text-foreground font-medium'
                      : 'text-muted-foreground hover:bg-secondary/50 hover:text-foreground'
                  )}
                >
                  <div className="flex items-center gap-3">
                    <StoreIcon
                      size={18}
                      className={isActive ? 'text-primary' : ''}
                    />
                    {store.name}
                  </div>
                  {store.connected && (
                    <CheckCircle2 size={14} className="text-emerald-500/80" />
                  )}
                </button>
              );
            })}
          </nav>
        </aside>

        {/* Conteúdo */}
        <main className="bg-background/95 custom-scrollbar flex-1 overflow-y-auto p-8">
          {renderActiveStore()}
        </main>
      </div>
    </WindowBase>
  );
}
