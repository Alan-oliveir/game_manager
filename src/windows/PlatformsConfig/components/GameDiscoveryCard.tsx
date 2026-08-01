import { FileText, FolderOpen, Gamepad2, Star } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { getBestExecutable } from '@/services/scannerService';
import { GameDiscovery } from '@/types/scanner';
import { Badge } from '@/ui/badge';
import { Button } from '@/ui/button';
import { Card, CardContent } from '@/ui/card';
import { ExecutableSelection } from '@/windows';

interface GameDiscoveryCardProps {
  discovery: GameDiscovery;
}

/**
 * Card individual de um jogo descoberto pelo scanner local. Exibe nome
 * sugerido, confiança da detecção, caminho base, quantidade de
 * executáveis candidatos e o melhor candidato sugerido, além do botão
 * que abre o modal de seleção de executável (ExecutableSelection).
 */
export function GameDiscoveryCard({
  discovery,
}: Readonly<GameDiscoveryCardProps>) {
  const { t } = useTranslation('platforms');
  const [showModal, setShowModal] = useState(false);
  const bestExe = getBestExecutable(discovery);

  return (
    <>
      <Card className="bg-card/40 border-border/40 hover:bg-card/60 transition-colors">
        <CardContent className="px-4 py-2">
          <div className="flex items-center justify-between gap-3">
            <div className="min-w-0 flex-1 space-y-1">
              <div className="flex items-center gap-2">
                <Gamepad2 className="text-muted-foreground h-4 w-4" />
                <h5 className="truncate text-sm font-semibold">
                  {discovery.suggestedName}
                </h5>
                <Badge variant="secondary" className="text-xs">
                  {discovery.confidence}/10
                </Badge>
              </div>

              <div className="text-muted-foreground flex flex-wrap gap-x-3 gap-y-0.5 text-xs">
                <div className="flex max-w-xs items-center gap-1 truncate">
                  <FolderOpen size={12} /> {discovery.basePath}
                </div>
                <div className="flex items-center gap-1">
                  <FileText size={12} /> {discovery.executables.length}{' '}
                  {t('discoveries_executables')}
                </div>
                {bestExe && (
                  <div className="flex items-center gap-1 text-yellow-500/80">
                    <Star size={12} className="fill-yellow-500/10" />
                    {t('discoveries_suggestion')}: {bestExe.filename}
                  </div>
                )}
              </div>
            </div>

            <Button
              size="sm"
              onClick={() => setShowModal(true)}
              className="shrink-0"
            >
              {t('discoveries_select')}
            </Button>
          </div>
        </CardContent>
      </Card>

      <ExecutableSelection
        open={showModal}
        discovery={discovery}
        onClose={() => setShowModal(false)}
      />
    </>
  );
}
