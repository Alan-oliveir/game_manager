import type { ReactNode } from 'react';

import { Separator } from '@/ui/separator';

interface PlatformHeaderProps {
  title: string;
  description: string;
  rightSlot?: ReactNode;
}

/**
 * Header padrão das abas de configuração de plataformas: título,
 * descrição e um slot opcional à direita (normalmente um StatusBadge, ou o
 * badge "salvo" da aba Wine). Já inclui o Separator que ficava repetido
 * embaixo do header em todos os XSettings.tsx.
 */
export function PlatformHeader({
  title,
  description,
  rightSlot,
}: Readonly<PlatformHeaderProps>) {
  return (
    <>
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">{title}</h2>
          <p className="text-muted-foreground mt-1 text-sm">{description}</p>
        </div>
        {rightSlot}
      </div>
      <Separator className="mt-5" />
    </>
  );
}
