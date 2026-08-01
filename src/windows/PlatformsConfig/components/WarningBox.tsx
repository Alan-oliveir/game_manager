import type { LucideIcon } from 'lucide-react';
import { TriangleAlert } from 'lucide-react';
import type { ReactNode } from 'react';

interface WarningBoxProps {
  title: ReactNode;
  children: ReactNode;
  icon?: LucideIcon;
}

/**
 * Caixa de aviso âmbar, usada por Heroic (aviso de duplicatas) e Wine
 * (aviso "só Linux"). `icon` é configurável porque cada uso original tinha
 * um ícone diferente (TriangleAlert no Heroic, Info no Wine).
 */
export function WarningBox({
  title,
  children,
  icon: Icon = TriangleAlert,
}: Readonly<WarningBoxProps>) {
  return (
    <div className="flex items-start gap-3 rounded-lg border border-amber-500/25 bg-amber-500/8 p-4">
      <Icon size={16} className="mt-0.5 shrink-0 text-amber-400" />
      <div className="space-y-1">
        <p className="text-sm font-medium text-amber-400">{title}</p>
        <div className="text-muted-foreground text-xs leading-relaxed">
          {children}
        </div>
      </div>
    </div>
  );
}
