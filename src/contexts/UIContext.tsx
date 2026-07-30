import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useMemo,
  useState,
} from 'react';

import { ViewFilters } from '@/hooks';
import { Game, Giveaway, RawgGame, UserPreferenceVector } from '@/types';

interface UIContextType {
  activeSection: string;
  setActiveSection: (section: string) => void;

  searchTerm: string;
  setSearchTerm: (term: string) => void;

  isAddModalOpen: boolean;
  setIsAddModalOpen: (open: boolean) => void;
  gameToEdit: Game | null;
  setGameToEdit: (game: Game | null) => void;

  selectedGameId: string | null;
  setSelectedGameId: (id: string | null) => void;

  // Filtros
  hideAdult: boolean;
  toggleAdultFilter: () => void;
  hideDuplicates: boolean;
  toggleDuplicatesFilter: () => void;
  hideNotInstalled: boolean;
  toggleNotInstalledFilter: () => void;
  groupByPlatform: boolean;
  toggleGroupByPlatform: () => void;

  // Agregado para consumo direto pelo dropdown de filtros
  viewFilters: ViewFilters;
  onViewFiltersChange: (next: ViewFilters) => void;

  trendingCache: RawgGame[];
  setTrendingCache: (games: RawgGame[]) => void;
  trendingKey: number;
  setTrendingKey: (key: number | ((prev: number) => number)) => void;
  profileCache: UserPreferenceVector | null;
  setProfileCache: (profile: UserPreferenceVector | null) => void;

  trendingFetchedAt: number | null;
  setTrendingFetchedAt: (value: number | null) => void;
  upcomingCache: RawgGame[];
  setUpcomingCache: (games: RawgGame[]) => void;
  upcomingFetchedAt: number | null;
  setUpcomingFetchedAt: (value: number | null) => void;
  giveawaysCache: Giveaway[];
  setGiveawaysCache: (games: Giveaway[]) => void;
  giveawaysFetchedAt: number | null;
  setGiveawaysFetchedAt: (value: number | null) => void;

  enableUpdaterChecks: boolean;
  setEnableUpdaterChecks: (value: boolean) => void;

  openAddModal: () => void;
  openEditModal: (game: Game) => void;
  closeAddModal: () => void;
}

const UIContext = createContext<UIContextType | undefined>(undefined);

export function UIProvider({ children }: Readonly<{ children: ReactNode }>) {
  const [activeSection, setActiveSection] = useState('home');
  const [searchTerm, setSearchTerm] = useState('');
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [gameToEdit, setGameToEdit] = useState<Game | null>(null);
  const [selectedGameId, setSelectedGameId] = useState<string | null>(null);

  const [hideAdult, setHideAdult] = useState(
    () => localStorage.getItem('playlite_hide_adult') === 'true'
  );
  const [hideDuplicates, setHideDuplicates] = useState(
    () => localStorage.getItem('playlite_hide_duplicates') === 'true'
  );
  const [hideNotInstalled, setHideNotInstalled] = useState(
    () => localStorage.getItem('playlite_hide_not_installed') === 'true'
  );
  const [groupByPlatform, setGroupByPlatform] = useState(
    () => localStorage.getItem('playlite_group_by_platform') === 'true'
  );

  const [trendingCache, setTrendingCache] = useState<RawgGame[]>([]);
  const [trendingKey, setTrendingKey] = useState(0);
  const [profileCache, setProfileCache] = useState<UserPreferenceVector | null>(
    null
  );

  const [trendingFetchedAt, setTrendingFetchedAt] = useState<number | null>(
    null
  );
  const [upcomingCache, setUpcomingCache] = useState<RawgGame[]>([]);
  const [upcomingFetchedAt, setUpcomingFetchedAt] = useState<number | null>(
    null
  );
  const [giveawaysCache, setGiveawaysCache] = useState<Giveaway[]>([]);
  const [giveawaysFetchedAt, setGiveawaysFetchedAt] = useState<number | null>(
    null
  );

  const [enableUpdaterChecks, setEnableUpdaterChecks] = useState(true);

  const toggleAdultFilter = useCallback(() => {
    setHideAdult(prev => {
      const newValue = !prev;
      localStorage.setItem('playlite_hide_adult', String(newValue));

      return newValue;
    });
  }, []);

  const toggleDuplicatesFilter = useCallback(() => {
    setHideDuplicates(prev => {
      const newValue = !prev;
      localStorage.setItem('playlite_hide_duplicates', String(newValue));

      return newValue;
    });
  }, []);

  const toggleNotInstalledFilter = useCallback(() => {
    setHideNotInstalled(prev => {
      const newValue = !prev;
      localStorage.setItem('playlite_hide_not_installed', String(newValue));

      return newValue;
    });
  }, []);

  const toggleGroupByPlatform = useCallback(() => {
    setGroupByPlatform(prev => {
      const newValue = !prev;
      localStorage.setItem('playlite_group_by_platform', String(newValue));

      return newValue;
    });
  }, []);

  const viewFilters: ViewFilters = useMemo(
    () => ({ hideAdult, hideDuplicates, hideNotInstalled }),
    [hideAdult, hideDuplicates, hideNotInstalled]
  );

  const onViewFiltersChange = useCallback(
    (next: ViewFilters) => {
      if (next.hideAdult !== hideAdult) toggleAdultFilter();

      if (next.hideDuplicates !== hideDuplicates) toggleDuplicatesFilter();

      if (next.hideNotInstalled !== hideNotInstalled)
        toggleNotInstalledFilter();
    },
    [
      hideAdult,
      hideDuplicates,
      hideNotInstalled,
      toggleAdultFilter,
      toggleDuplicatesFilter,
      toggleNotInstalledFilter,
    ]
  );

  const openAddModal = useCallback(() => {
    setGameToEdit(null);
    setIsAddModalOpen(true);
  }, []);

  const openEditModal = useCallback((game: Game) => {
    setGameToEdit(game);
    setIsAddModalOpen(true);
  }, []);

  const closeAddModal = useCallback(() => {
    setIsAddModalOpen(false);
    setGameToEdit(null);
  }, []);

  const value = useMemo(
    () => ({
      activeSection,
      setActiveSection,
      searchTerm,
      setSearchTerm,
      isAddModalOpen,
      setIsAddModalOpen,
      gameToEdit,
      setGameToEdit,
      selectedGameId,
      setSelectedGameId,
      hideAdult,
      toggleAdultFilter,
      hideDuplicates,
      toggleDuplicatesFilter,
      hideNotInstalled,
      toggleNotInstalledFilter,
      groupByPlatform,
      toggleGroupByPlatform,
      viewFilters,
      onViewFiltersChange,
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
      enableUpdaterChecks,
      setEnableUpdaterChecks,
      openAddModal,
      openEditModal,
      closeAddModal,
    }),
    [
      activeSection,
      searchTerm,
      isAddModalOpen,
      gameToEdit,
      selectedGameId,
      hideAdult,
      toggleAdultFilter,
      hideDuplicates,
      toggleDuplicatesFilter,
      hideNotInstalled,
      toggleNotInstalledFilter,
      groupByPlatform,
      toggleGroupByPlatform,
      viewFilters,
      onViewFiltersChange,
      trendingCache,
      trendingKey,
      profileCache,
      trendingFetchedAt,
      upcomingCache,
      upcomingFetchedAt,
      giveawaysCache,
      giveawaysFetchedAt,
      enableUpdaterChecks,
      openAddModal,
      openEditModal,
      closeAddModal,
    ]
  );

  return <UIContext.Provider value={value}>{children}</UIContext.Provider>;
}

// eslint-disable-next-line react-refresh/only-export-components
export function useUI() {
  const context = useContext(UIContext);

  if (!context) {
    throw new Error('useUI must be used within UIProvider');
  }

  return context;
}
