import { ExternalLink as ExternalLinkIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { LINK_CONFIG } from '@/types';
import { Button } from '@/ui/button';

interface GameExternalLinksProps {
  links?: Record<string, string>;
}

export function GameLinks({ links }: Readonly<GameExternalLinksProps>) {
  const { t } = useTranslation('game_detail');

  if (!links || Object.keys(links).length === 0) return null;

  const validLinks = Object.entries(links).filter(
    ([, url]) => url && url.trim().length > 0
  );

  if (validLinks.length === 0) return null;

  return (
    <div className="border-border/40 border-t py-6">
      <h3 className="text-muted-foreground mb-3 text-xs font-bold tracking-widest uppercase">
        {t('links_heading')}
      </h3>
      <div className="grid grid-cols-4 gap-3 lg:grid-cols-6">
        {validLinks.map(([key, url]) => {
          const config = LINK_CONFIG[key.toLowerCase()] || {
            labelKey: undefined,
            icon: ExternalLinkIcon,
          };
          const Icon = config.icon;

          return (
            <Button
              key={key}
              variant="outline"
              size="sm"
              className="h-8 w-full justify-start gap-2 px-2 text-xs font-medium"
              asChild
            >
              <a href={url} target="_blank" rel="noreferrer">
                <Icon size={16} className="shrink-0 opacity-80" />
                <span className="truncate">
                  {config.labelKey ? t(config.labelKey) : key}
                </span>
              </a>
            </Button>
          );
        })}
      </div>
    </div>
  );
}
