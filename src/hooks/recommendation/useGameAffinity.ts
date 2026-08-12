import { useMemo } from 'react';

import tagMetadata from '@/data/tag_metadata.json';
import { Giveaway, UserPreferenceVector } from '@/types';
import { calculateAffinity } from '@/utils/recommendation';

const TAG_MATCHERS = tagMetadata
  .filter(tag => tag.visible)
  .map(tag => {
    const patterns = new Set<string>();

    if (tag.slug) patterns.add(tag.slug);

    if (tag.name) patterns.add(tag.name);

    const regexes = Array.from(patterns)
      .map(value => buildTagRegex(value))
      .filter((regex): regex is RegExp => !!regex);

    return { slug: tag.slug, category: tag.category, regexes };
  });

function buildTagRegex(value: string): RegExp | null {
  const tokens = value
    .toLowerCase()
    .split(/[^a-z0-9]+/g)
    .filter(Boolean);

  if (tokens.length === 0) return null;

  const pattern = tokens.join(String.raw`(?:\s|-)`);

  return new RegExp(String.raw`\b${pattern}\b`, 'gi');
}

export function calculateGiveawayAffinity(
  giveaway: Giveaway,
  profile: UserPreferenceVector | null
) {
  if (!profile)
    return { affinity: 0, badge: undefined } as {
      affinity: number;
      badge?: 'TOP PICK' | 'PARA VOCÊ';
    };

  const textToScan = `${giveaway.title} ${giveaway.description}`.toLowerCase();
  let score = 0;

  for (const [seriesName, weight] of Object.entries(profile.series)) {
    if (giveaway.title.toLowerCase().includes(seriesName.toLowerCase())) {
      score += weight;
    }
  }

  const matchedTags = TAG_MATCHERS.flatMap(tag => {
    const matches = tag.regexes.some(regex => regex.test(textToScan));

    return matches ? [{ slug: tag.slug, category: tag.category }] : [];
  });

  score += calculateAffinity(profile, [], matchedTags, null);

  let badge: 'TOP PICK' | 'PARA VOCÊ' | undefined;

  if (score > 150) badge = 'TOP PICK';
  else if (score > 100) badge = 'PARA VOCÊ';

  return { affinity: score, badge };
}

/** Forma mínima que qualquer fonte de jogo precisa ter pra entrar no cálculo de afinidade. */
interface AffinitySource {
  genres?: string[];
  tags?: { slug: string }[];
  series?: string | null;
}

function scoreGame(
  game: AffinitySource,
  profile: UserPreferenceVector | null
): number {
  return calculateAffinity(
    profile,
    game.genres ?? [],
    game.tags ?? [],
    game.series ?? null
  );
}

export function calculateGameAffinity<T extends AffinitySource>(
  game: T,
  profile: UserPreferenceVector | null
): {
  genres: string[];
  tags: { slug: string }[];
  affinity: number;
  badge?: 'TOP PICK' | 'PARA VOCÊ';
} {
  const genres = game.genres ?? [];
  const tags = game.tags ?? [];
  const affinity = calculateAffinity(
    profile,
    genres,
    tags,
    game.series ?? null
  );

  let badge: 'TOP PICK' | 'PARA VOCÊ' | undefined;

  if (affinity > 150) badge = 'TOP PICK';
  else if (affinity > 100) badge = 'PARA VOCÊ';

  return { genres, tags, affinity, badge };
}

export function useSortedByAffinity<T extends AffinitySource>(
  games: T[],
  profile: UserPreferenceVector | null
) {
  return useMemo(() => {
    if (!profile) return games;

    return [...games].sort(
      (a, b) => scoreGame(b, profile) - scoreGame(a, profile)
    );
  }, [games, profile]);
}
