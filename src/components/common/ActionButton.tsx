import { LucideIcon } from 'lucide-react';
import React, { memo, MouseEvent } from 'react';

import { cn } from '@/lib/utils';
import { Button } from '@/ui/button';

interface ActionButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  icon: LucideIcon;
  variant?:
    | 'default'
    | 'destructive'
    | 'outline'
    | 'secondary'
    | 'ghost'
    | 'glass'
    | 'glass-destructive';
  size?: number; // Tamanho do ícone
  tooltip?: string;
}

/**
 * Botão circular de ação rápida usado em overlays de cards e menus.
 * Estende o Button do Shadcn/UI com variantes customizadas de vidro (glass).
 *
 * Usado principalmente em:
 * - Overlay de cards (Jogar, Favoritar, Menu)
 * - GameActionsMenu (trigger do dropdown)
 *
 * Bloqueia propagação de cliques automaticamente para evitar conflitos com cards clicáveis.
 *
 * Memoizado: usado dentro de grades com centenas de cards, então evita
 * re-renderizar quando o pai re-renderiza mas as props deste botão em
 * específico não mudaram (depende de `onClick` estável no componente pai).
 *
 * @param icon - Componente de ícone do Lucide React
 * @param variant - Estilo visual. Variantes customizadas:
 *   - 'glass': fundo preto semi-transparente com blur (hover em cards)
 *   - 'glass-destructive': igual glass mas com ícone vermelho (favorito ativo)
 * @param size - Tamanho do ícone em pixels (padrão: 18)
 * @param tooltip - Texto exibido no hover (atributo title)
 */
export const ActionButton = memo(function ActionButton({
  icon: Icon,
  className,
  variant = 'secondary',
  size = 18,
  tooltip,
  onClick,
  ...props
}: Readonly<ActionButtonProps>) {
  // Tratamento especial para variantes customizadas que não existem no Shadcn padrão
  const getVariantClasses = () => {
    switch (variant) {
      case 'glass':
        return 'bg-black/60 text-white hover:bg-black/80 backdrop-blur-md border-none';
      case 'glass-destructive': // Para o coração quando favoritado
        return 'bg-black/60 text-red-500 hover:bg-black/80 backdrop-blur-md border-none';
      default:
        return ''; // Outras variantes usam as classes padrão do Shadcn
    }
  };

  const baseVariant =
    variant === 'glass' || variant === 'glass-destructive' ? 'ghost' : variant;

  const handleClick = (e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();

    if (onClick) onClick(e);
  };

  return (
    <Button
      size="icon"
      variant={baseVariant}
      className={cn(
        'h-10 w-10 rounded-full shadow-lg transition-all', // Base style
        getVariantClasses(),
        className
      )}
      onClick={handleClick}
      title={tooltip}
      {...props}
    >
      {
        <Icon
          size={size}
          className={cn(variant === 'glass-destructive' ? 'fill-red-500' : '')}
        />
      }
    </Button>
  );
});
