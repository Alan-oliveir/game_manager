import { BookOpen, Globe, type LucideIcon } from 'lucide-react';

import {
  Discord,
  HLTB,
  IGDB,
  NexusMods,
  Reddit,
  Steam,
  Twitch,
  Wikipedia,
  X,
  YouTube,
} from '@/components/icons/logos';

type LinkIcon = LucideIcon | typeof Steam;

interface LinkConfigEntry {
  labelKey: string;
  icon: LinkIcon;
}

export const LINK_CONFIG: Record<string, LinkConfigEntry> = {
  steam: { labelKey: 'links_steam', icon: Steam },
  website: { labelKey: 'links_website', icon: Globe },
  reddit: { labelKey: 'links_reddit', icon: Reddit },
  wiki: { labelKey: 'links_wiki', icon: BookOpen },
  wikipedia: { labelKey: 'links_wikipedia', icon: Wikipedia },
  twitter: { labelKey: 'links_x', icon: X },
  twitch: { labelKey: 'links_twitch', icon: Twitch },
  youtube: { labelKey: 'links_youtube', icon: YouTube },
  discord: { labelKey: 'links_discord', icon: Discord },
  igdb: { labelKey: 'links_igdb', icon: IGDB },
  hltb: { labelKey: 'links_hltb', icon: HLTB },
  nexus: { labelKey: 'links_nexus', icon: NexusMods },
};
