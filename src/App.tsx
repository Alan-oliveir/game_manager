import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useMemo } from 'react';
import { Toaster } from 'sonner';

import { ErrorBoundary } from '@/components';
import AddGame from '@/dialogs/AddGame';
import {
  useDebounce,
  useEnrichmentNotifications,
  useEnrichmentProgressNotifications,
  useGameDetails,
} from '@/hooks';
import { UpdateProvider } from '@/providers/UpdateProvider';
import { Game } from '@/types';
import { initializeToastRouting, toast } from '@/utils/toast';
import Favorites from '@/views/Favorites';
import Home from '@/views/Home';
import Libraries from '@/views/Libraries';
import Playlist from '@/views/Playlist';
import Settings from '@/views/Settings';
import Trending from '@/views/Trending';
import Wishlist from '@/views/Wishlist';
import GameDetail from '@/windows/GameDetail/GameDetail';

import Header from './components/layout/Header';
import Sidebar from './components/layout/Sidebar';
import {
  GameLibraryProvider,
  UIProvider,
  useGameLibrary,
  useUI,
} from './contexts';
import { ConfirmProvider, useConfirm } from './providers/ConfirmProvider';

function AppContent() {
  // Contexts
  const { games, refreshGames, saveGame, removeGame, toggleFavorite } =
    useGameLibrary();
  const {
    activeSection,
    setActiveSection,
    searchTerm,
    setSearchTerm,
    isAddModalOpen,
    gameToEdit,
    selectedGameId,
    setSelectedGameId,
    libraryViewFilters,
    onLibraryViewFiltersChange,
    favoritesViewFilters,
    onFavoritesViewFiltersChange,
    libraryGroupByPlatform,
    libraryToggleGroupByPlatform,
    favoritesGroupByPlatform,
    favoritesToggleGroupByPlatform,
    trendingCache,
    setTrendingCache,
    trendingKey,
    setTrendingKey,
    profileCache,
    setProfileCache,
    trendingFetchedAt,
    setTrendingFetchedAt,
    upcomingCache,
    setUpcomingCache,
    upcomingFetchedAt,
    setUpcomingFetchedAt,
    giveawaysCache,
    setGiveawaysCache,
    giveawaysFetchedAt,
    setGiveawaysFetchedAt,
    openAddModal,
    openEditModal,
    closeAddModal,
  } = useUI();

  const debouncedSearchTerm = useDebounce(searchTerm, 300);
  const { confirm } = useConfirm();

  // Inicia atualização em background de reviews e preços
  useEffect(() => {
    invoke('check_and_refresh_background').catch(err => {
      console.error('Erro ao iniciar atualização em background:', err);
    });
  }, []);

  useEffect(() => {
    void initializeToastRouting();
  }, []);

  useEnrichmentNotifications();
  useEnrichmentProgressNotifications();

  // Updater manual: sob demanda apenas
  const handleCheckUpdates = async () => {
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();

      if (update) {
        toast.info(`Nova versão disponível: ${update.version}`, {
          description: 'Clique em Atualizar para instalar.',
          duration: 10000,
          action: {
            label: 'Atualizar',
            onClick: async () => {
              const toastId = toast.loading('Baixando atualização...', {
                duration: Infinity,
              });

              try {
                await update.downloadAndInstall();
                toast.success('Atualização instalada! Reiniciando...', {
                  id: toastId,
                  duration: 2000,
                });
              } catch {
                toast.error('Falha ao instalar. Tente novamente.', {
                  id: toastId,
                });
              }
            },
          },
        });
      } else {
        toast.success('Você já está na versão mais recente!');
      }
    } catch (error) {
      console.error('Erro ao verificar atualizações:', error);
      toast.error('Não foi possível verificar atualizações');
    }
  };

  // Selected game memo
  const selectedGame = useMemo(
    () => games.find(g => g.id === selectedGameId) || null,
    [games, selectedGameId]
  );

  const { details, loading, siblings, refresh } = useGameDetails(
    selectedGame,
    games
  );

  const currentViewFilters =
    activeSection === 'favorites' ? favoritesViewFilters : libraryViewFilters;
  const currentOnViewFiltersChange =
    activeSection === 'favorites'
      ? onFavoritesViewFiltersChange
      : onLibraryViewFiltersChange;

  const currentGroupByPlatform =
    activeSection === 'favorites'
      ? favoritesGroupByPlatform
      : libraryGroupByPlatform;
  const currentToggleGroupByPlatform =
    activeSection === 'favorites'
      ? favoritesToggleGroupByPlatform
      : libraryToggleGroupByPlatform;

  // Handlers
  const handleSettingsUpdate = () => {
    refreshGames();
    setTrendingCache([]);
    setTrendingKey(k => k + 1);
  };

  const handleSaveGameWrapper = async (data: Partial<Game>) => {
    try {
      await saveGame(data, gameToEdit?.id);
      closeAddModal();
    } catch (e) {
      toast.error('Erro ao salvar: ' + e);
    }
  };

  const handleDeleteWrapper = useCallback(
    async (id: string) => {
      const confirmed = await confirm({
        title: 'Excluir Jogo',
        description:
          'Tem certeza que deseja excluir este jogo? Esta ação não pode ser desfeita.',
        confirmText: 'Excluir',
        cancelText: 'Cancelar',
      });

      if (confirmed) {
        try {
          await removeGame(id);
          toast.success('Jogo excluído com sucesso!');
        } catch {
          toast.error('Erro ao excluir jogo.');
        }
      }
    },
    [confirm, removeGame]
  );

  const handleGameClick = useCallback(
    (game: Game) => setSelectedGameId(game.id),
    [setSelectedGameId]
  );

  const handleSwitchGame = (id: string) => {
    setSelectedGameId(id);
  };

  const closeDetails = () => {
    setSelectedGameId(null);
  };

  // Props comuns
  const commonGameActions = {
    onToggleFavorite: toggleFavorite,
    onGameClick: handleGameClick,
    onDeleteGame: handleDeleteWrapper,
    onEditGame: openEditModal,
  };

  // Roteamento
  const renderContent = () => {
    switch (activeSection) {
      case 'home':
        return (
          <Home
            onChangeTab={setActiveSection}
            games={games}
            trendingCache={trendingCache}
            setTrendingCache={setTrendingCache}
            profileCache={profileCache}
            setProfileCache={setProfileCache}
            trendingFetchedAt={trendingFetchedAt}
            setTrendingFetchedAt={setTrendingFetchedAt}
            onGameClick={handleGameClick}
          />
        );
      case 'libraries':
        return (
          <Libraries
            games={games}
            searchTerm={debouncedSearchTerm}
            hideAdult={libraryViewFilters.hideAdult}
            hideDuplicates={libraryViewFilters.hideDuplicates}
            hideNotInstalled={libraryViewFilters.hideNotInstalled}
            groupByPlatform={libraryGroupByPlatform}
            {...commonGameActions}
          />
        );
      case 'favorites':
        return (
          <Favorites
            games={games}
            searchTerm={debouncedSearchTerm}
            hideAdult={favoritesViewFilters.hideAdult}
            hideDuplicates={favoritesViewFilters.hideDuplicates}
            hideNotInstalled={favoritesViewFilters.hideNotInstalled}
            groupByPlatform={favoritesGroupByPlatform}
            {...commonGameActions}
          />
        );
      case 'playlist':
        return (
          <Playlist
            allGames={games}
            onGameClick={handleGameClick}
            profileCache={profileCache}
          />
        );
      case 'trending':
        return (
          <Trending
            key={trendingKey}
            userGames={games}
            onChangeTab={setActiveSection}
            cachedGames={trendingCache}
            setCachedGames={setTrendingCache}
            cachedFetchedAt={trendingFetchedAt}
            setCachedFetchedAt={setTrendingFetchedAt}
            upcomingCache={upcomingCache}
            setUpcomingCache={setUpcomingCache}
            upcomingFetchedAt={upcomingFetchedAt}
            setUpcomingFetchedAt={setUpcomingFetchedAt}
            giveawaysCache={giveawaysCache}
            setGiveawaysCache={setGiveawaysCache}
            giveawaysFetchedAt={giveawaysFetchedAt}
            setGiveawaysFetchedAt={setGiveawaysFetchedAt}
          />
        );
      case 'wishlist':
        return <Wishlist searchTerm={debouncedSearchTerm} />;
      case 'settings':
        return <Settings onLibraryUpdate={handleSettingsUpdate} />;
      default:
        return <div className="p-10 text-center">Página não encontrada</div>;
    }
  };

  return (
    <div className="bg-background text-foreground flex h-screen overflow-hidden">
      <Sidebar
        activeSection={activeSection}
        onSectionChange={setActiveSection}
        games={games}
      />
      <main className="flex min-w-0 flex-1 flex-col">
        <Header
          onAddGame={openAddModal}
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
          activeSection={activeSection}
          viewFilters={currentViewFilters}
          onViewFiltersChange={currentOnViewFiltersChange}
          groupByPlatform={currentGroupByPlatform}
          onToggleGroupByPlatform={currentToggleGroupByPlatform}
          onCheckUpdates={handleCheckUpdates}
          onLibraryUpdate={refreshGames}
          userGames={games}
        />
        <ErrorBoundary>{renderContent()}</ErrorBoundary>
      </main>
      <ErrorBoundary>
        <AddGame
          isOpen={isAddModalOpen}
          onClose={closeAddModal}
          onSave={handleSaveGameWrapper}
          gameToEdit={gameToEdit}
        />
        <GameDetail
          isOpen={!!selectedGameId}
          onClose={closeDetails}
          game={selectedGame}
          details={details}
          loading={loading}
          siblings={siblings}
          onSwitchGame={handleSwitchGame}
          onRefresh={refresh}
        />
      </ErrorBoundary>
      <Toaster />
    </div>
  );
}

export default function App() {
  return (
    <ConfirmProvider>
      <GameLibraryProvider>
        <UIProvider>
          <UpdateProvider>
            <AppContent />
          </UpdateProvider>
        </UIProvider>
      </GameLibraryProvider>
    </ConfirmProvider>
  );
}
