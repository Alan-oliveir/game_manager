import type { ReactNode } from 'react';

interface PlatformActionsFooterProps {
  children: ReactNode;
}

/**
 * Wrapper da linha de botões de ação no rodapé das abas de plataforma.
 */
export function PlatformActionsFooter({
  children,
}: Readonly<PlatformActionsFooterProps>) {
  return (
    <div className="border-border/10 flex justify-end gap-3 border-t pt-8">
      {children}
    </div>
  );
}
