import {
  Clock,
  ExternalLink,
  Filter,
  Flame,
  Gamepad2,
  Gift,
  Heart,
  Loader2,
  Sparkles,
  TrendingUp,
  WifiOff,
} from 'lucide-react';
import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import {
  ActionButton,
  Affinity,
  ErrorState,
  FreeGameCard,
  Hero,
  Similarity,
  StandardGameCard,
} from '@/components';
import {
  calculateGameAffinity,
  GiveawayWithAffinity,
  useGiveaways,
  useHeroCarousel,
  useNetworkStatus,
  useProfileSimilar,
  useRecommendation,
  useSortedByAffinity,
  useTrending,
  useUpcoming,
  useWishlist,
} from '@/hooks';
import { wishlistService } from '@/services/wishlistService';
import { Game, Giveaway, RawgGame, SimilarGame } from '@/types';
import { Button } from '@/ui/button';
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/ui/dropdown-menu';
import { Separator } from '@/ui/separator';
import { Skeleton } from '@/ui/skeleton';
import { openExternalLink, toast } from '@/utils';

interface TrendingProps {
  userGames: Game[];
  onChangeTab: (tab: string) => void;
  cachedGames: RawgGame[];
  setCachedGames: (games: RawgGame[]) => void;
  cachedFetchedAt: number | null;
  setCachedFetchedAt: (value: number | null) => void;
  upcomingCache: RawgGame[];
  setUpcomingCache: (games: RawgGame[]) => void;
  upcomingFetchedAt: number | null;
  setUpcomingFetchedAt: (value: number | null) => void;
  giveawaysCache: Giveaway[];
  setGiveawaysCache: (games: Giveaway[]) => void;
  giveawaysFetchedAt: number | null;
  setGiveawaysFetchedAt: (value: number | null) => void;
}

const PLATFORM_OPTIONS = [
  { id: 'Steam', label: 'Steam' },
  { id: 'Epic Store', label: 'Epic Games' },
  { id: 'Itch.io', label: 'Itch.io' },
  { id: 'GOG', label: 'GOG' },
  { id: 'Origin', label: 'EA' },
  { id: 'Ubisoft Connect', label: 'Ubisoft Connect' },
  { id: 'Battle.net', label: 'Battle.net' },
  { id: 'Amazon', label: 'Amazon' },
  { id: 'IndieGala', label: 'IndieGala' },
  { id: 'DRM-Free', label: 'DRM-Free' },
];

export default function Trending(props: Readonly<TrendingProps>) {
  // Hooks globais
  const { t } = useTranslation('trending');
  const isOnline = useNetworkStatus();
  const { profile } = useRecommendation();
  const { games: wishlistGames } = useWishlist();

  // Hooks de dados
  const {
    games,
    allGenres,
    loading: gamesLoading,
    error,
    selectedGenre,
    setSelectedGenre,
    addToWishlist,
  } = useTrending(props);

  const {
    games: profileSimilar,
    loading: profileLoading,
    hasAnchors,
  } = useProfileSimilar(props.userGames);

  const { upcomingGames } = useUpcoming({
    cachedGames: props.upcomingCache,
    setCachedGames: props.setUpcomingCache,
    cachedFetchedAt: props.upcomingFetchedAt,
    setCachedFetchedAt: props.setUpcomingFetchedAt,
  });

  const {
    filteredGiveaways,
    loading: giveawaysLoading,
    selectedPlatforms,
    togglePlatform,
    shouldShowGiveawaysSection,
  } = useGiveaways(profile, {
    cachedGiveaways: props.giveawaysCache,
    setCachedGiveaways: props.setGiveawaysCache,
    cachedFetchedAt: props.giveawaysFetchedAt,
    setCachedFetchedAt: props.setGiveawaysFetchedAt,
  });

  // Dados derivados
  const heroGames = games.slice(0, 5);

  // Prioriza jogos com maior afinidade ao perfil do usuário
  const gridGames = useSortedByAffinity(games.slice(5, 15), profile);

  // Set em vez de busca linear repetida em cada card (troca O(n) por O(1))
  const wishlistIds = useMemo(
    () => new Set(wishlistGames.map(w => w.id)),
    [wishlistGames]
  );

  // Afinidade calculada uma vez por jogo/render, não recalculada dentro do .map() a cada re-render do componente (hover, filtro, etc.)
  const gridGamesWithAffinity = useMemo(
    () =>
      gridGames.map(game => ({
        game,
        ...calculateGameAffinity(game, profile),
      })),
    [gridGames, profile]
  );

  const upcomingGamesWithAffinity = useMemo(
    () =>
      upcomingGames.map(game => ({
        game,
        ...calculateGameAffinity(game, profile),
      })),
    [upcomingGames, profile]
  );

  // Mantém os dados em tela caso exista conteúdo em cache
  const hasData =
    games.length > 0 ||
    upcomingGames.length > 0 ||
    filteredGiveaways.length > 0 ||
    profileSimilar.length > 0;

  // Estado do carrossel
  const {
    currentIndex: heroIndex,
    next: nextHero,
    prev: prevHero,
  } = useHeroCarousel(heroGames.length, {
    autoAdvanceMs: 8000,
  });

  // Informações do jogo atualmente exibido no Hero
  const currentHero = heroGames[heroIndex];

  const isHeroInWishlist = currentHero
    ? wishlistIds.has(currentHero.id.toString())
    : false;

  // Eventos da interface
  const handleRetry = () => {
    window.location.reload();
  };

  const handleWishlistClick = async (game: RawgGame) => {
    try {
      await addToWishlist(game);

      toast.success(t('game_added_to_wishlist_title', { name: game.name }), {
        description: t('game_added_to_wishlist_description'),
      });
    } catch {
      toast.error(t('add_to_wishlist_error_title'), {
        description: t('add_to_wishlist_error_description'),
      });
    }
  };

  const handleSimilarWishlistClick = async (game: SimilarGame) => {
    try {
      await wishlistService.addToWishlist({
        id: game.id,
        name: game.name,
        cover_url: game.coverUrl,
      });

      toast.success(t('game_added_to_wishlist_title', { name: game.name }), {
        description: t('game_added_to_wishlist_description'),
      });
    } catch {
      toast.error(t('add_to_wishlist_error_title'), {
        description: t('add_to_wishlist_error_description'),
      });
    }
  };

  // Renderizações condicionais

  // Exibe estado offline apenas quando não há dados disponíveis em cache
  if (!isOnline && !hasData) {
    return (
      <ErrorState
        type="offline"
        onAction={() => props.onChangeTab('libraries')}
      />
    );
  }

  // Exibe erros de configuração ou da API somente quando não há dados para exibir
  if (error && !hasData) {
    const isConfigError = error.includes('401') || error.includes('Key');

    if (isConfigError) {
      return (
        <ErrorState
          type="config"
          onAction={() => props.onChangeTab('settings')}
        />
      );
    }

    return <ErrorState type="api" message={error} onRetry={handleRetry} />;
  }

  // Exibe o loading inicial somente quando ainda não existe conteúdo disponível
  if (gamesLoading && !hasData) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center space-y-4">
        <Loader2 className="text-primary h-10 w-10 animate-spin" />
        <p className="text-muted-foreground">{t('checking_global_trends')}</p>
      </div>
    );
  }

  // Caso nenhum jogo tenha sido carregado para o Hero
  if (!currentHero) {
    return (
      <div className="text-muted-foreground flex flex-1 flex-col items-center justify-center">
        <div className="bg-muted/20 mb-4 rounded-full p-4">
          <Gamepad2 className="h-12 w-12 opacity-50" />
        </div>

        <h3 className="text-lg font-medium">{t('no_games_found')}</h3>

        <Button variant="outline" className="mt-6" onClick={handleRetry}>
          {t('reload_button')}
        </Button>
      </div>
    );
  }

  // Renderização principal
  return (
    <div className="custom-scrollbar bg-background flex-1 overflow-y-auto">
      {/* Banner de Modo Offline */}
      {!isOnline && hasData && (
        <div className="flex items-center justify-center gap-2 border-b border-yellow-500/20 bg-yellow-500/10 px-6 py-2 text-xs font-medium text-yellow-600 dark:text-yellow-400">
          <WifiOff size={14} />
          <span>{t('offline_mode_banner')}</span>
        </div>
      )}

      {/* 1. Hero */}
      <Hero
        gameId={currentHero.id.toString()}
        title={currentHero.name}
        backgroundUrl={currentHero.backgroundImage}
        coverUrl={currentHero.backgroundImage}
        genres={currentHero.genres.map(g => g.name)}
        rating={currentHero.rating}
        showNavigation={heroGames.length > 1}
        onNext={nextHero}
        onPrev={prevHero}
        badges={
          <div className="bg-gold-500/20 inline-flex items-center gap-2 rounded-full border border-orange-500/20 bg-orange-500/20 px-3 py-1 text-sm font-medium text-orange-400">
            <Flame size={16} /> {t('trending_badge')}
          </div>
        }
        actions={
          <>
            <Button
              variant="secondary"
              className="gap-2"
              onClick={() => handleWishlistClick(currentHero)}
            >
              <Heart
                size={18}
                className={`text-red-500 ${isHeroInWishlist ? 'fill-current' : ''}`}
              />{' '}
              {t('wishlist_button')}
            </Button>
            <Button
              variant="outline"
              className="gap-2 border-white/20 bg-transparent text-white hover:bg-white/10"
              onClick={() =>
                openExternalLink(`https://rawg.io/games/${currentHero.id}`)
              }
            >
              <ExternalLink size={18} /> {t('view_details_button')}
            </Button>
          </>
        }
      />

      {/* 2. Barra de Filtros de Gênero (Sticky) */}
      <div className="bg-background/80 border-border sticky top-0 z-20 border-b p-4 shadow-sm backdrop-blur-md">
        <div className="mx-auto flex max-w-7xl flex-wrap items-center gap-4 px-6">
          <div className="text-muted-foreground flex items-center gap-2">
            <Filter size={18} />
            <span className="text-sm font-medium">{t('filter_label')}:</span>
          </div>

          <select
            value={selectedGenre}
            onChange={e => setSelectedGenre(e.target.value)}
            className="bg-secondary text-secondary-foreground focus:ring-primary cursor-pointer rounded-md border-none px-3 py-1.5 text-sm font-medium outline-none focus:ring-1"
          >
            <option value="all">{t('all_genres_option')}</option>
            {allGenres.map(g => (
              <option key={g} value={g}>
                {g}
              </option>
            ))}
          </select>

          <div className="text-muted-foreground ml-auto hidden text-sm sm:block">
            {t('suggestions_available')}
          </div>
        </div>
      </div>

      {/* 3. Container Principal de Conteúdo */}
      <div className="mx-auto max-w-7xl px-6 py-8">
        {shouldShowGiveawaysSection && (
          <>
            {/* Header da Seção + Filtro de Loja */}
            <div className="mb-6 flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-purple-500/10 bg-linear-to-br to-purple-600 p-2.5 text-white shadow-lg">
                  <Gift size={20} />
                </div>
                <div>
                  <h2 className="text-2xl font-bold tracking-tight">
                    {t('free_games_section')}
                  </h2>
                  <p className="text-muted-foreground text-sm">
                    {t('free_games_description')}
                  </p>
                </div>
              </div>

              {/* Botão de Filtro */}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="secondary"
                    size="sm"
                    className="w-40 gap-2 font-medium"
                  >
                    <Filter size={16} />
                    {t('filter_stores_button')}
                    {selectedPlatforms.length < PLATFORM_OPTIONS.length && (
                      <span className="bg-primary text-primary-foreground ml-1 flex h-5 w-5 items-center justify-center rounded-full text-[10px] font-bold shadow-sm">
                        {selectedPlatforms.length}
                      </span>
                    )}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-40">
                  <DropdownMenuLabel>{t('platforms_label')}</DropdownMenuLabel>
                  <DropdownMenuSeparator />
                  {PLATFORM_OPTIONS.map(platform => (
                    <DropdownMenuCheckboxItem
                      key={platform.id}
                      checked={selectedPlatforms.includes(platform.id)}
                      onCheckedChange={() => togglePlatform(platform.id)}
                    >
                      {platform.label}
                    </DropdownMenuCheckboxItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
            </div>

            {/* Grid de Jogos Grátis */}
            {filteredGiveaways.length === 0 && !giveawaysLoading ? (
              <div className="text-muted-foreground rounded-lg border border-dashed py-12 text-center">
                <Gift className="mx-auto mb-3 h-12 w-12 opacity-20" />
                <p className="text-base font-medium">
                  {t('no_games_with_filters')}
                </p>
                <p className="mt-1 text-sm">{t('try_other_platforms')}</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
                {giveawaysLoading
                  ? Array.from({ length: 3 }).map((_, i) => (
                      <div key={i} className="space-y-3">
                        <Skeleton className="aspect-video w-full rounded-lg" />
                        <Skeleton className="h-5 w-3/4" />
                        <Skeleton className="h-4 w-1/2" />
                      </div>
                    ))
                  : filteredGiveaways.map((game: GiveawayWithAffinity) => (
                      <FreeGameCard
                        key={game.id}
                        {...game}
                        badge={game.affinityData.badge}
                      />
                    ))}
              </div>
            )}
            <Separator className="mt-8" />
          </>
        )}

        {/* 4. Similares ao Meu Perfil */}
        {hasAnchors && (profileSimilar.length > 0 || profileLoading) && (
          <>
            <div className="mb-6 flex items-center gap-2 pt-8">
              <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
                <Sparkles size={20} />
              </div>
              <div>
                <h2 className="text-2xl font-bold">
                  {t('profile_similar_section')}
                </h2>
                <p className="text-muted-foreground text-sm">
                  {t('profile_similar_description')}
                </p>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-6 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
              {profileLoading
                ? Array.from({ length: 5 }).map((_, i) => (
                    <div key={i} className="space-y-3">
                      <Skeleton className="aspect-3/4 w-full rounded-lg" />
                      <Skeleton className="h-4 w-3/4" />
                      <Skeleton className="h-3 w-1/2" />
                    </div>
                  ))
                : profileSimilar.map((game: SimilarGame) => {
                    const isInWishlist = wishlistIds.has(game.id);

                    return (
                      <StandardGameCard
                        key={game.id}
                        id={game.id}
                        title={game.name}
                        coverUrl={game.coverUrl ?? undefined}
                        rating={game.rating ?? undefined}
                        subtitle={game.genre ?? undefined}
                        badge={
                          <Similarity becauseOf={game.becauseOf}>
                            <span className="rounded-full px-2 py-0.5 text-[10px] font-medium text-white">
                              {game.becauseOf}
                            </span>
                          </Similarity>
                        }
                        actions={
                          <>
                            <ActionButton
                              icon={Heart}
                              variant={
                                isInWishlist ? 'glass-destructive' : 'glass'
                              }
                              onClick={() => handleSimilarWishlistClick(game)}
                              tooltip={t('wishlist_button')}
                            />
                            {game.link ? (
                              <ActionButton
                                icon={ExternalLink}
                                variant="secondary"
                                size={16}
                                onClick={() => openExternalLink(game.link!)}
                                tooltip={t('view_details_button')}
                              />
                            ) : null}
                          </>
                        }
                      />
                    );
                  })}
            </div>

            <Separator className="mt-8" />
          </>
        )}

        {/* 5. Mais Sugestões (Trending Grid) */}
        <div className="mb-6 flex items-center gap-2 pt-8">
          <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
            <TrendingUp size={20} />
          </div>
          <h2 className="text-2xl font-bold">
            {t('more_suggestions_section')}
          </h2>
        </div>

        <div className="grid grid-cols-2 gap-6 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
          {gridGamesWithAffinity.map(({ game, genres, badge }) => {
            const isInWishlist = wishlistIds.has(game.id.toString());

            return (
              <StandardGameCard
                id={game.id.toString()}
                key={game.id}
                title={game.name}
                coverUrl={game.backgroundImage}
                rating={game.rating}
                subtitle={genres.slice(0, 2).join(', ')}
                badge={
                  badge ? (
                    <Affinity
                      badge={
                        badge as 'SÉRIE FAVORITA' | 'TOP PICK' | 'PARA VOCÊ'
                      }
                    >
                      <span>{badge}</span>
                    </Affinity>
                  ) : null
                }
                actions={
                  <>
                    <ActionButton
                      icon={Heart}
                      variant={isInWishlist ? 'glass-destructive' : 'glass'}
                      onClick={() => handleWishlistClick(game)}
                      tooltip={t('wishlist_button')}
                    />
                    <ActionButton
                      icon={ExternalLink}
                      variant="secondary"
                      size={16}
                      onClick={() =>
                        openExternalLink(`https://rawg.io/games/${game.id}`)
                      }
                      tooltip={t('view_details_button')}
                    />
                  </>
                }
              />
            );
          })}
        </div>

        {/* 6. Lançamentos Aguardados */}
        {upcomingGames.length > 0 && (
          <>
            <div className="mb-6 flex items-center gap-2 pt-12">
              <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
                <Clock size={20} />
              </div>
              <h2 className="text-2xl font-bold">
                {t('upcoming_releases_section')}
              </h2>
            </div>

            <div className="grid grid-cols-2 gap-6 md:grid-cols-3 lg:grid-cols-5">
              {upcomingGamesWithAffinity.map(({ game, badge }) => {
                const isInWishlist = wishlistIds.has(game.id.toString());

                return (
                  <StandardGameCard
                    id={game.id.toString()}
                    key={game.id}
                    title={game.name}
                    coverUrl={game.backgroundImage}
                    subtitle={
                      game.released
                        ? t('release_label') +
                          ': ' +
                          new Date(game.released).toLocaleDateString()
                        : t('coming_soon')
                    }
                    badge={
                      badge ? (
                        <Affinity
                          badge={
                            badge as 'SÉRIE FAVORITA' | 'TOP PICK' | 'PARA VOCÊ'
                          }
                        >
                          <span>{badge}</span>
                        </Affinity>
                      ) : null
                    }
                    actions={
                      <>
                        <ActionButton
                          icon={Heart}
                          variant={isInWishlist ? 'glass-destructive' : 'glass'}
                          onClick={() => handleWishlistClick(game)}
                          tooltip={t('wishlist_button')}
                        />
                        <ActionButton
                          icon={ExternalLink}
                          variant="secondary"
                          size={16}
                          onClick={() =>
                            openExternalLink(`https://rawg.io/games/${game.id}`)
                          }
                          tooltip={t('view_details_button')}
                        />
                      </>
                    }
                  />
                );
              })}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
