import { invoke } from '@tauri-apps/api/core';
import {
  ChartBar,
  Clock,
  Dna,
  ExternalLink,
  Gamepad2,
  Heart,
  ImageOff,
  Library,
  Loader2,
  Play,
  Sparkles,
  TrendingUp,
  Trophy,
  Users,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { StandardGameCard, StatCard } from '@/components/cards';
import { ActionButton, Hero, RecentAchievements } from '@/components/common';
import {
  EAPlaySection,
  GamePassSection,
  HumbleChoiceSection,
  OfflineAwareSection,
  PrimeGamingSection,
  UbisoftPlusSection,
} from '@/components/subscriptions';
import { Recommendation } from '@/components/tooltips';
import { useHeroCarousel, useHome, useLaunchGame } from '@/hooks';
import { Game, TrendingGame, UserPreferenceVector } from '@/types';
import { Button } from '@/ui/button';
import { Separator } from '@/ui/separator';
import { formatTime, openExternalLink } from '@/utils';

interface HomeProps {
  onChangeTab: (tab: string) => void;
  games: Game[];
  trendingCache: TrendingGame[];
  setTrendingCache: (games: TrendingGame[]) => void;
  onGameClick: (game: Game) => void;
  profileCache: UserPreferenceVector | null;
  setProfileCache: (profile: UserPreferenceVector) => void;
  trendingFetchedAt: number | null;
  setTrendingFetchedAt: (value: number | null) => void;
}

export default function Home(props: Readonly<HomeProps>) {
  const { t } = useTranslation('common');

  const { launchGame } = useLaunchGame();

  const {
    stats,
    continuePlaying,
    backlogRecommendations,
    collaborativeRecs,
    mostPlayed,
    topGenres,
    profileLoading,
    trending,
    loadingRecommendations,
  } = useHome(props);

  // Estado dos serviços habilitados
  const [enabledServices, setEnabledServices] = useState<string[]>([]);

  useEffect(() => {
    invoke<string[]>('get_subscription_settings')
      .then(setEnabledServices)
      .catch(() => setEnabledServices([]));
  }, []);

  // Lógica do Hero usando useHeroCarousel.
  const heroSlides = useMemo(
    () =>
      [
        mostPlayed[0],
        ...(trending || []).slice(0, 2),
        backlogRecommendations[0],
      ].filter(Boolean),
    [mostPlayed, trending, backlogRecommendations]
  );

  const { currentIndex, next, prev } = useHeroCarousel(heroSlides.length, {
    autoAdvanceMs: 8000,
  });
  const currentHero = heroSlides[currentIndex] || mostPlayed[0];

  // Helper obter lista de gêneros
  const getGenresList = (game: Game | TrendingGame): string[] =>
    game.genres ?? [];

  // Helper obter nome de exibição — prioriza o nome canônico do IGDB (display_name) quando disponível;
  // cai para o nome importado da loja (Game.name) ou o nome nativo do TrendingGame quando não há.
  const getDisplayName = (game: Game | TrendingGame): string =>
    ('displayName' in game && game.displayName) || game.name;

  // Helper isLocalGame
  const isLocalGame = (game: Game | TrendingGame): game is Game =>
    'playtime' in game;

  const recommendationsContent = (() => {
    if (loadingRecommendations) {
      return (
        <div className="flex h-40 items-center justify-center rounded-xl border border-dashed border-white/10">
          <Loader2 className="text-muted-foreground animate-spin" />
        </div>
      );
    }

    if (backlogRecommendations.length > 0) {
      return (
        <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-5">
          {backlogRecommendations.map(game => (
            <StandardGameCard
              id={game.id.toString()}
              key={game.id}
              title={getDisplayName(game)}
              coverUrl={game.coverUrl}
              subtitle={game.genres?.[0]}
              onClick={() => props.onGameClick(game)}
              badge={
                <Recommendation reason={game.reason}>
                  <span>{t('recommended_badge')}</span>
                </Recommendation>
              }
              actions={
                <ActionButton
                  icon={Play}
                  variant="secondary"
                  onClick={() => launchGame(game)}
                  tooltip={t('play_now_tooltip')}
                />
              }
            />
          ))}
        </div>
      );
    }

    return (
      <div className="border-border text-muted-foreground rounded-xl border border-dashed p-8 text-center">
        <p>{t('nothing_to_play_message')}</p>
      </div>
    );
  })();

  if (profileLoading) {
    return (
      <div className="animate-in fade-in flex h-full flex-1 flex-col items-center justify-center gap-4 duration-700">
        <Loader2 className="text-primary h-10 w-10 animate-spin" />
      </div>
    );
  }

  return (
    <div className="custom-scrollbar bg-background flex-1 overflow-y-auto pb-10">
      {/* Seção: Hero component */}
      {currentHero && (
        <Hero
          gameId={currentHero.id.toString()}
          title={getDisplayName(currentHero)}
          coverUrl={currentHero.coverUrl}
          genres={getGenresList(currentHero)}
          rating={isLocalGame(currentHero) ? currentHero.userRating : undefined}
          showNavigation={heroSlides.length > 1}
          onNext={next}
          onPrev={prev}
          badges={
            <div className="inline-flex items-center gap-2 rounded-full border border-orange-500/20 bg-orange-500/20 px-3 py-1 text-sm font-medium text-orange-400">
              {backlogRecommendations.some(g => g.id === currentHero.id) && (
                <>
                  <Sparkles size={14} /> {t('hero_suggestion_badge')}
                </>
              )}
              {trending?.some(g => g.id === currentHero.id) && (
                <>
                  <TrendingUp size={14} /> {t('hero_trending_badge')}
                </>
              )}
              {mostPlayed.some(g => g.id === currentHero.id) && (
                <>
                  <Trophy size={14} /> {t('hero_champion_badge')}
                </>
              )}
            </div>
          }
          actions={
            isLocalGame(currentHero) ? (
              <Button onClick={() => launchGame(currentHero)}>
                <Play size={18} />{' '}
                <p className="font-bold">{t('play_now_button')}</p>
              </Button>
            ) : (
              <Button
                variant="outline"
                className="gap-2 border-white/20 bg-transparent text-white hover:bg-white/10"
                onClick={() =>
                  openExternalLink(
                    `https://www.igdb.com/games/${currentHero.slug}`
                  )
                }
              >
                <ExternalLink size={18} /> {t('view_details_button')}
              </Button>
            )
          }
        />
      )}

      <Separator className={'mb-3'} />

      <div className="relative z-20 mx-auto max-w-7xl space-y-10 p-8">
        {/* Stats Cards */}
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {/* StatCards */}
          <StatCard
            icon={<Library size={24} />}
            label={t('total_games_stat')}
            value={stats.totalGames}
            color="text-blue-500"
            bg="bg-blue-500/10"
          />
          <StatCard
            icon={<Clock size={24} />}
            label={t('playtime_stat')}
            value={formatTime(stats.totalPlaytime)}
            color="text-purple-500"
            bg="bg-purple-500/10"
          />
          <StatCard
            icon={<Heart size={24} />}
            label={t('favorites_stat')}
            value={stats.totalFavorites}
            color="text-pink-500"
            bg="bg-pink-500/10"
          />
          <StatCard
            icon={<Trophy size={24} />}
            label={t('favorite_genre_stat')}
            value={topGenres[0]?.[0] || '-'}
            color="text-yellow-500"
            bg="bg-yellow-500/10"
          />
        </div>

        {/* Achievements Component */}
        <RecentAchievements
          onViewAll={() => props.onChangeTab('achievements')}
        />

        {/* Seção: Prime Gaming — renderiza somente se habilitado e online */}
        <OfflineAwareSection enabled={enabledServices.includes('prime_gaming')}>
          <PrimeGamingSection />
        </OfflineAwareSection>

        {/* Seção: Game Pass — renderiza somente se habilitado e online */}
        <OfflineAwareSection enabled={enabledServices.includes('game_pass_pc')}>
          <GamePassSection
            excludeEaPlay={enabledServices.includes('ea_play')}
          />
        </OfflineAwareSection>

        {/* Seção: EA Play — renderiza quando habilitado e online. Quando ambos (Game Pass + EA Play)
            estiverem ativos, o GamePassSection exclui os títulos EA Play para evitar duplicatas,
            portanto aqui mostramos sempre a seção do EA Play quando o usuário a habilitar. */}
        <OfflineAwareSection enabled={enabledServices.includes('ea_play')}>
          <EAPlaySection />
        </OfflineAwareSection>

        {/* Seção: Ubisoft+ — renderiza somente se habilitado e online */}
        <OfflineAwareSection enabled={enabledServices.includes('ubisoft_plus')}>
          <UbisoftPlusSection />
        </OfflineAwareSection>

        {/* Seção: Humble Choice — renderiza somente se habilitado e online */}
        <OfflineAwareSection
          enabled={enabledServices.includes('humble_choice')}
        >
          <HumbleChoiceSection />
        </OfflineAwareSection>

        {/* Seção: Continue Jogando */}
        {continuePlaying.length > 0 && (
          <section>
            {/* Header e Grid de Continue Jogando */}
            <div className="mb-6 flex items-center gap-2">
              <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
                <Clock size={24} />
              </div>
              <h2 className="text-2xl font-bold">
                {t('continue_playing_section')}
              </h2>
            </div>
            <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-5">
              {continuePlaying.map(game => (
                <StandardGameCard
                  id={game.id.toString()}
                  key={game.id}
                  title={getDisplayName(game)}
                  coverUrl={game.coverUrl}
                  subtitle={`${formatTime(game.playtime)} ${t('played_suffix')}`}
                  onClick={() => props.onGameClick(game)}
                  actions={
                    <ActionButton
                      icon={Play}
                      variant="secondary"
                      onClick={() => launchGame(game)}
                      tooltip={t('play_now_tooltip')}
                    />
                  }
                />
              ))}
            </div>
          </section>
        )}

        {/* Seção: content-based */}
        <section className="mb-12">
          <div className="mb-6 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
                <Dna size={24} />
              </div>
              <div>
                <h2 className="text-2xl font-bold">
                  {t('recommended_for_you_section')}
                </h2>
              </div>
            </div>
          </div>

          {recommendationsContent}
        </section>

        {/* Seção: Collaborative Filtering */}
        {collaborativeRecs.length > 0 && (
          <section className="animate-in fade-in slide-in-from-bottom-4 mb-12 duration-700">
            <div className="mb-6 flex items-center gap-2">
              <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
                <Users size={24} />
              </div>
              <div>
                <h2 className="text-2xl font-bold">
                  {t('users_like_you_section')}
                </h2>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-5">
              {collaborativeRecs.map(game => (
                <StandardGameCard
                  id={game.id.toString()}
                  key={`cf-${game.id}`}
                  title={getDisplayName(game)}
                  coverUrl={game.coverUrl}
                  subtitle={game.genres?.[0]}
                  onClick={() => props.onGameClick(game)}
                  badge={
                    <Recommendation reason={game.reason}>
                      <span>{t('community_badge')}</span>
                    </Recommendation>
                  }
                  actions={
                    <ActionButton
                      icon={Play}
                      variant="secondary"
                      onClick={() => launchGame(game)}
                      tooltip={t('play_now_tooltip')}
                    />
                  }
                />
              ))}
            </div>
          </section>
        )}

        {/* Estatísticas e Gráficos */}
        <div className="mb-6 flex items-center gap-2">
          <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
            <ChartBar size={24} />
          </div>
          <h2 className="text-2xl font-bold">{t('statistics_section')}</h2>
        </div>
        <div className="grid grid-cols-2 gap-4 lg:grid-cols-3 lg:gap-8">
          {/* Mais Jogados Card */}
          <div className="bg-card border-border rounded-xl border p-6 lg:col-span-2">
            <div className="mb-3 flex items-center gap-2">
              <TrendingUp size={20} className="text-primary" />
              <h2 className="text-lg font-semibold">
                {t('most_played_section')}
              </h2>
            </div>
            <Separator className={'mb-3'} />
            <div className="grid grid-cols-1 gap-4">
              {mostPlayed.map((game, index) => (
                <button
                  type="button"
                  key={game.id}
                  className="hover:bg-muted/50 group flex w-full cursor-pointer items-center gap-3 rounded-lg p-2 text-left transition-colors"
                  onClick={() => launchGame(game)}
                >
                  <div className="text-muted-foreground w-6 text-center font-bold">
                    {index + 1}
                  </div>
                  {game.coverUrl ? (
                    <img
                      src={game.coverUrl}
                      alt=""
                      className="bg-muted h-16 w-12 rounded object-cover"
                    />
                  ) : (
                    <div className="bg-muted flex h-16 w-12 items-center justify-center rounded text-[10px]">
                      <ImageOff />
                    </div>
                  )}
                  <div className="min-w-0 flex-1">
                    <h4 className="group-hover:text-primary truncate text-sm font-medium transition-colors">
                      {getDisplayName(game)}
                    </h4>
                    <div className="mt-1 flex items-center justify-between">
                      <span className="text-muted-foreground max-w-25 truncate text-xs">
                        {game.genres?.[0]}
                      </span>
                      <span className="bg-secondary rounded px-1.5 py-0.5 font-mono text-xs">
                        {formatTime(game.playtime)}
                      </span>
                    </div>
                  </div>
                </button>
              ))}
            </div>
          </div>
          {/* Gêneros Card */}
          <div className="bg-card border-border col-span-1 h-full rounded-xl border p-6">
            <div className="mb-3 flex items-center gap-2">
              <Gamepad2 size={20} className="text-primary" />
              <h2 className="text-lg font-semibold">
                {t('genres_and_played_section')}
              </h2>
            </div>
            <Separator className={'mb-3'} />
            <div className="space-y-4">
              {topGenres.map(([genre, count]) => {
                const percent = Math.round((count / stats.totalGames) * 100);

                return (
                  <div key={genre}>
                    <div className="mb-1 flex justify-between text-xs">
                      <span className="font-medium">{genre}</span>
                      <span className="text-muted-foreground">
                        {count} {t('games_suffix')}
                      </span>
                    </div>
                    <div className="bg-secondary h-2 overflow-hidden rounded-full">
                      <div
                        className="bg-primary/80 h-full transition-all duration-1000 ease-out"
                        style={{ width: `${percent}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
