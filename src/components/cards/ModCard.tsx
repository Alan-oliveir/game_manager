import { ExternalLink, ImageOff, User } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { TrendingMod } from '@/types';
import { openExternalLink } from '@/utils/openLink';

interface ModCardProps {
  mod: TrendingMod;
}

export function ModCard({ mod }: ModCardProps) {
  const { t } = useTranslation('game_detail');
  const [imgError, setImgError] = useState(false);

  return (
    <div className="group border-border bg-muted/10 hover:border-border/80 hover:bg-muted/20 overflow-hidden rounded-lg border transition-all duration-200">
      <div className="bg-muted/30 relative aspect-video w-full overflow-hidden">
        {mod.pictureUrl && !imgError ? (
          <img
            src={mod.pictureUrl}
            alt={mod.name}
            className="absolute inset-0 h-full w-full object-cover"
            onError={() => setImgError(true)}
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center">
            <ImageOff className="h-8 w-8 opacity-20" />
          </div>
        )}

        <button
          onClick={() => openExternalLink(mod.modPageUrl)}
          title={t('addons_mod_link_title')}
          className="bg-background/80 text-foreground hover:text-primary absolute top-2 right-2 flex h-7 w-7 items-center justify-center rounded-md opacity-0 backdrop-blur-sm transition-opacity duration-200 group-hover:opacity-100"
        >
          <ExternalLink className="h-3.5 w-3.5" />
        </button>
      </div>

      <div className="p-3">
        <p
          className="text-foreground truncate text-sm font-medium"
          title={mod.name}
        >
          {mod.name}
        </p>

        <div className="text-muted-foreground mt-1 flex items-center gap-1 text-xs">
          <User className="h-3 w-3 shrink-0" />
          <span className="truncate">{mod.author}</span>
        </div>

        {mod.summary && (
          <p className="text-muted-foreground mt-1.5 line-clamp-2 text-xs">
            {mod.summary}
          </p>
        )}
      </div>
    </div>
  );
}
