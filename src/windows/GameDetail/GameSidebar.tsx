import {
  Building2,
  Calendar,
  Clock,
  Gamepad2,
  ListCheck,
  type LucideIcon,
  Star,
  Tag,
  TrendingUp,
  Trophy,
  Users,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Game, GameDetails, GameLibraryLink, GameTag } from '@/types/game';
import { Badge } from '@/ui/badge';
import { Button } from '@/ui/button';
import { formatTime } from '@/utils';
import {
  AgeRatingBadge,
  GameLinks,
  HltbBadge,
  SteamReviewBadge,
} from '@/windows';

interface GameSidebarProps {
  game: Game;
  details: GameDetails | null;
  siblings: GameLibraryLink[];
  onSwitchGame: (id: string) => void;
}

// === TagSection ====

const TAG_ORDER = ['mode', 'narrative', 'theme', 'gameplay', 'meta'];
const TAG_LABEL_KEYS: Record<string, string> = {
  mode: 'tags_mode',
  narrative: 'tags_narrative',
  theme: 'tags_theme',
  gameplay: 'tags_gameplay',
  meta: 'tags_meta',
};

function TagSection({ tags }: Readonly<{ tags: GameTag[] }>) {
  const { t } = useTranslation('game_detail');

  const grouped = tags.reduce(
    (acc, tag) => {
      const cat = tag.category;

      if (!acc[cat]) acc[cat] = [];

      acc[cat].push(tag);

      return acc;
    },
    {} as Record<string, GameTag[]>
  );

  return (
    <div className="space-y-3">
      {TAG_ORDER.map(cat => {
        const catTags = grouped[cat];

        if (!catTags?.length) return null;

        return (
          <div key={cat} className="space-y-1.5">
            <span className="text-muted-foreground/70 pl-1 text-[10px] font-bold tracking-widest uppercase">
              {t(TAG_LABEL_KEYS[cat] || cat)}
            </span>
            <div className="flex flex-wrap gap-1.5">
              {catTags.map(tag => (
                <Badge
                  key={tag.slug}
                  variant="secondary"
                  className="bg-secondary/40 hover:bg-secondary hover:border-border/50 border border-transparent px-2 py-0.5 text-xs font-normal transition-all"
                >
                  {tag.name}
                </Badge>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}

// === Componente Principal (Sidebar) ===

export function GameSidebar({
  game,
  details,
  siblings,
  onSwitchGame,
}: Readonly<GameSidebarProps>) {
  const { t } = useTranslation('game_detail');

  // Lógica para extrair APENAS os modos de jogo
  const gameModes =
    details?.tags && Array.isArray(details.tags)
      ? details.tags
          .filter(tag => tag.category === 'mode') // Filtra só categoria 'mode'
          .map(tag => tag.name) // Pega o nome (Singleplayer, Co-op...)
          .join(', ') || null // Junta com vírgula
      : null;

  // Verifica se existe pelo menos um dado de HLTB maior que 0
  const hasHltbData = !!(
    (details?.hltbMainStory && details.hltbMainStory > 0) ||
    (details?.hltbMainExtra && details.hltbMainExtra > 0) ||
    (details?.hltbCompletionist && details.hltbCompletionist > 0) ||
    (details?.hltbCoopTime && details.hltbCoopTime > 0)
  );

  return (
    <div className="space-y-6 p-6 lg:p-8">
      {/* AVISO DE CONTEÚDO ADULTO */}
      {(details?.isAdult || details?.adultTags) && (
        <AgeRatingBadge
          isAdult={details?.isAdult}
          adultTags={details?.adultTags}
        />
      )}

      {/* 1. DADOS DO USUÁRIO */}
      <div className="space-y-4">
        <h3 className="text-muted-foreground flex items-center gap-2 text-sm font-bold tracking-wider uppercase">
          <Trophy size={18} className="text-primary" /> {t('sidebar_your_data')}
        </h3>
        <div className="grid grid-cols-2 gap-4">
          {/* Card 1: Tempo Real Jogado */}
          <div className="bg-card rounded-lg border px-4 py-2 shadow-sm">
            <span className="text-muted-foreground mb-1 block text-[11px] font-semibold uppercase">
              {t('sidebar_played')}
            </span>
            <div className="flex items-center gap-2 font-mono text-base font-bold">
              <Clock size={18} className="text-muted-foreground/70" />
              {formatTime(game.playtime)}
            </div>
          </div>

          {/* Card 2: Status */}
          <div className="bg-card rounded-lg border px-4 py-2 shadow-sm">
            <span className="text-muted-foreground mb-1 block text-[11px] font-semibold uppercase">
              {t('sidebar_status')}
            </span>
            <div className="flex items-center gap-2 text-base font-medium">
              <TrendingUp size={18} className="text-muted-foreground/70" />
              {game.playtime === 0
                ? t('sidebar_backlog')
                : t('sidebar_playing')}
            </div>
          </div>
        </div>
      </div>

      {/* 2. REVIEWS */}
      {(details?.steamReviewLabel || details?.criticScore) && (
        <div className="space-y-4">
          <h3 className="text-muted-foreground flex items-center gap-2 text-sm font-bold tracking-wider uppercase">
            <Star size={18} /> {t('sidebar_reviews_heading')}
          </h3>

          <SteamReviewBadge
            label={details?.steamReviewLabel}
            count={details?.steamReviewCount}
            score={details?.steamReviewScore}
          />
        </div>
      )}

      {/* 3. TEMPO DE JOGO (HOW LONG TO BEAT) */}
      {hasHltbData && (
        <div className="space-y-3">
          <h3 className="text-muted-foreground flex items-center gap-2 text-sm font-bold tracking-wider uppercase">
            <Clock size={18} /> {t('sidebar_playtime_heading')}
          </h3>

          <HltbBadge
            mainStory={details?.hltbMainStory}
            mainExtra={details?.hltbMainExtra}
            completionist={details?.hltbCompletionist}
            coopTime={details?.hltbCoopTime}
          />
        </div>
      )}

      {/* 4. DETALHES TÉCNICOS */}
      <div className="space-y-1.5">
        <h3 className="text-muted-foreground flex items-center gap-2 pb-2 text-sm font-bold tracking-wider uppercase">
          <ListCheck size={18} /> {t('sidebar_details_heading')}
        </h3>

        <DetailRow
          icon={Building2}
          label={t('sidebar_dev_pub')}
          value={`${details?.developer}`}
        />

        <DetailRow
          icon={Building2}
          label={t('sidebar_dev_pub')}
          value={`${details?.publisher}`}
        />

        {details?.releaseDate && (
          <DetailRow
            icon={Calendar}
            label={t('sidebar_release')}
            value={new Date(details.releaseDate).toLocaleDateString('pt-BR')}
          />
        )}

        <DetailRow
          icon={Gamepad2}
          label={t('sidebar_genre')}
          value={game.genres}
        />

        <DetailRow
          icon={TrendingUp}
          label={t('sidebar_series')}
          value={details?.series}
        />

        <DetailRow
          icon={Star}
          label={t('sidebar_metacritic')}
          value={
            details?.criticScore ? details.criticScore.toString() : undefined
          }
        />

        <DetailRow
          icon={Users}
          label={t('sidebar_mode')}
          value={gameModes ?? undefined}
        />
      </div>

      {/* 5. LINKS */}
      <GameLinks links={details?.externalLinks} />

      {/* 6. TAGS (Com Categorização) */}
      {details?.tags &&
        Array.isArray(details.tags) &&
        details.tags.length > 0 && (
          <div className="space-y-3">
            <h3 className="text-muted-foreground flex items-center gap-1 text-sm font-bold tracking-wider uppercase">
              <Tag size={18} /> {t('sidebar_features')}
            </h3>
            <TagSection tags={details.tags} />
          </div>
        )}

      {/* 7. OUTRAS PLATAFORMAS */}
      {siblings.length > 0 && (
        <div className="border-border/40 space-y-2 border-t pt-4">
          <span className="text-muted-foreground text-sm font-medium">
            {t('sidebar_other_versions')}
          </span>
          <div className="flex flex-wrap gap-2">
            {siblings.map(sib => (
              <Button
                key={sib.id}
                variant="ghost"
                size="sm"
                onClick={() => onSwitchGame(sib.id)}
                className="border-border/50 h-7 border text-xs"
              >
                {sib.library}
              </Button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// === DetailRow ====

function DetailRow({
  icon: Icon,
  label,
  value,
}: Readonly<{
  icon: LucideIcon;
  label: string;
  value?: string;
}>) {
  if (!value) return null;

  return (
    <div className="border-border/40 flex justify-between rounded border-b px-2 py-2 transition-colors hover:bg-white/5">
      <span className="text-muted-foreground flex items-center gap-2 text-sm">
        <Icon size={16} /> {label}
      </span>
      <span className="text-foreground/90 max-w-[60%] truncate text-right text-sm font-medium">
        {value}
      </span>
    </div>
  );
}
