import type { ReactNode } from 'react';

import { cn } from '@/lib/utils';

interface InfoNoteBoxProps {
  children: ReactNode;
  className?: string;
}

/**
 * Caixa neutra (bg-muted/30) para notas informativas — ex: explicações
 * sobre o uso do Wine em Epic/Legacy, ou o preview de comando na aba Wine.
 * Aceita qualquer conteúdo (parágrafos, listas, blocos de código).
 */
export function InfoNoteBox({
  children,
  className,
}: Readonly<InfoNoteBoxProps>) {
  return (
    <div
      className={cn('bg-muted/30 space-y-2 rounded-md border p-3', className)}
    >
      {children}
    </div>
  );
}
