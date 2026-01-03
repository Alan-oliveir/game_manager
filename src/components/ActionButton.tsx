import { Button } from "@/components/ui/button";
import { LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { MouseEvent } from "react";

interface ActionButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  icon: LucideIcon;
  variant?:
    | "default"
    | "destructive"
    | "outline"
    | "secondary"
    | "ghost"
    | "glass"
    | "glass-destructive";
  size?: number; // Tamanho do ícone
  tooltip?: string;
}

export function ActionButton({
  icon: Icon,
  className,
  variant = "secondary",
  size = 18,
  tooltip,
  onClick,
  ...props
}: ActionButtonProps) {
  // Tratamento especial para variantes customizadas que não existem no Shadcn padrão
  const getVariantClasses = () => {
    switch (variant) {
      case "glass":
        return "bg-black/60 text-white hover:bg-black/80 backdrop-blur-md border-none";
      case "glass-destructive": // Para o coração quando favoritado
        return "bg-black/60 text-red-500 hover:bg-black/80 backdrop-blur-md border-none";
      default:
        return ""; // Outras variantes usam as classes padrão do Shadcn
    }
  };

  const baseVariant =
    variant === "glass" || variant === "glass-destructive" ? "ghost" : variant;

  const handleClick = (e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    if (onClick) onClick(e);
  };

  return (
    <Button
      size="icon"
      variant={baseVariant}
      className={cn(
        "rounded-full h-10 w-10 shadow-lg transition-all", // Base style
        getVariantClasses(),
        className
      )}
      onClick={handleClick}
      title={tooltip}
      {...props}
    >
      <Icon
        size={size}
        className={cn(
          // Se for glass-destructive, força o fill vermelho
          variant === "glass-destructive" ? "fill-red-500" : ""
        )}
      />
    </Button>
  );
}
