import type { LucideIcon } from 'lucide-react';
import { Loader2 } from 'lucide-react';

import { cn } from '@/lib/utils';
import { Button } from '@/ui/button';

interface PlatformActionButtonProps {
  onClick: () => void;
  isLoading: boolean;
  disabled: boolean;
  label: string;
  loadingLabel?: string;
  icon?: LucideIcon;
  variant?: 'default' | 'outline';
  className?: string;
}

/**
 * Botão de ação padrão das abas de plataforma (importar, salvar, salvar e
 * importar): troca o ícone por um spinner enquanto `isLoading`, e
 * opcionalmente troca o texto por `loadingLabel`. `icon` fica de fora
 * quando o botão original só troca o texto (ex: "Salvar credenciais" da
 * Steam, que não tem ícone no estado ocioso).
 */
export function PlatformActionButton({
  onClick,
  isLoading,
  disabled,
  label,
  loadingLabel,
  icon: Icon,
  variant = 'default',
  className,
}: Readonly<PlatformActionButtonProps>) {
  return (
    <Button
      variant={variant}
      onClick={onClick}
      disabled={disabled}
      className={cn(
        'flex items-center gap-2',
        variant === 'default' && 'bg-blue-600 text-white hover:bg-blue-700',
        className
      )}
    >
      {isLoading ? (
        <Loader2 className="h-4 w-4 animate-spin" />
      ) : (
        Icon && <Icon size={16} />
      )}
      {isLoading ? (loadingLabel ?? label) : label}
    </Button>
  );
}
