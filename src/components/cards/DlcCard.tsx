import { CheckCircle2, ExternalLink, ImageOff } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { DlcKind, GameDlc } from '@/types';
import { Badge } from '@/ui/badge';
import { openExternalLink } from '@/utils/openLink';

const DLC_KIND_LABELS: Record<string, string> = {
  expansion: 'addons_dlc_kind_expansion',
  standalone_expansion: 'addons_dlc_kind_standalone_expansion',
  dlc: 'addons_dlc_kind_dlc',
};

interface DlcCardProps {
  dlc: GameDlc;
}

export function DlcCard({ dlc }: DlcCardProps) {
  const { t } = useTranslation('game_detail');
  const [imgError, setImgError] = useState(false);
  const kindLabelKey = DLC_KIND_LABELS[dlc.kind as DlcKind];

  return (
    <div className="border-border bg-muted/10 hover:border-border/80 hover:bg-muted/20 flex items-start gap-3 rounded-lg border p-3 transition-colors">
      <div className="bg-muted/30 h-16 w-12 shrink-0 overflow-hidden rounded">
        {dlc.coverUrl && !imgError ? (
          <img
            src={dlc.coverUrl}
            alt={dlc.name}
            className="h-full w-full object-cover"
            onError={() => setImgError(true)}
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center">
            <ImageOff className="text-muted-foreground/40 h-4 w-4" />
          </div>
        )}
      </div>

      <div className="min-w-0 flex-1">
        <p
          className="text-foreground truncate text-sm font-medium"
          title={dlc.name}
        >
          {dlc.name}
        </p>

        <div className="mt-1.5 flex flex-wrap items-center gap-1.5">
          <Badge variant="secondary" className="text-[10px]">
            {kindLabelKey ? t(kindLabelKey) : dlc.kind}
          </Badge>

          {dlc.owned && (
            <Badge
              variant="outline"
              className="border-green-500/30 text-[10px] text-green-500"
            >
              <CheckCircle2 className="h-2.5 w-2.5" />
              {t('addons_dlc_owned_badge')}
            </Badge>
          )}
        </div>

        {dlc.igdbUrl && (
          <button
            onClick={() => openExternalLink(dlc.igdbUrl!)}
            className="text-muted-foreground hover:text-primary mt-1.5 flex items-center gap-1 text-xs transition-colors"
          >
            <ExternalLink className="h-3 w-3" />
            {t('addons_view_on_igdb')}
          </button>
        )}
      </div>
    </div>
  );
}
