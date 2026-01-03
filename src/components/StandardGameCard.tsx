import { ReactNode, useState } from "react";
import { Play, ImageOff } from "lucide-react";
import { cn } from "@/lib/utils";
import { ActionButton } from "./ActionButton";

interface StandardGameCardProps {
  title: string;
  coverUrl?: string | null;
  subtitle?: string;
  badge?: string;
  rating?: number;
  onClick?: () => void;
  actions?: ReactNode;
  className?: string;
  onPlay?: () => void;
}

export default function StandardGameCard({
  title,
  coverUrl,
  subtitle,
  badge,
  rating,
  onClick,
  actions,
  className,
  onPlay,
}: StandardGameCardProps) {
  // Estado para controlar erro de carregamento da imagem
  const [imageError, setImageError] = useState(false);

  return (
    <div
      className={cn(
        "group relative bg-card rounded-xl overflow-hidden border border-border hover:shadow-lg transition-all hover:-translate-y-1 cursor-pointer flex flex-col h-full",
        className
      )}
      onClick={onClick}
    >
      {/* Container da Imagem (Aspect Ratio fixo 3/4 para capas verticais) */}
      <div className="aspect-3/4 bg-muted relative overflow-hidden">
        {/* Lógica de Imagem com Fallback */}
        {coverUrl && !imageError ? (
          <img
            src={coverUrl}
            alt={title}
            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
            onError={() => setImageError(true)} // Ativa o fallback se falhar
          />
        ) : (
          /* Fallback Visual (Gradiente + Ícone + Nome) */
          <div className="w-full h-full bg-linear-to-br from-secondary/50 via-muted to-background flex flex-col items-center justify-center p-4 text-center">
            <ImageOff className="w-10 h-10 opacity-20 mb-3" />
            <span className="text-[10px] text-muted-foreground font-semibold uppercase tracking-widest line-clamp-2">
              {title}
            </span>
          </div>
        )}

        {/* Overlay de Ações (Hover) */}
        <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2 p-4 z-20">
          {onPlay && (
            <ActionButton
              icon={Play}
              variant="glass"
              onClick={onPlay}
              tooltip="Jogar Agora"
            />
          )}

          {/* Ações secundárias (Favoritar, Menu, etc) */}
          {actions && (
            <div className="flex items-center justify-center gap-2">
              {actions}
            </div>
          )}
        </div>

        {/* Badge (Canto Superior Esquerdo) */}
        {badge && (
          <div className="absolute top-2 left-2 bg-purple-600 text-white text-[10px] font-bold px-2 py-0.5 rounded-full shadow-lg z-10 border border-purple-400/30 backdrop-blur-md">
            {badge}
          </div>
        )}
      </div>

      {/* Conteúdo (Título e Subtítulo) */}
      <div className="p-3 flex flex-col flex-1 gap-1">
        <h3 className="font-semibold text-sm line-clamp-1" title={title}>
          {title}
        </h3>

        <div className="flex justify-between items-center mt-auto">
          <span className="text-xs text-muted-foreground truncate max-w-30">
            {subtitle}
          </span>

          {rating && (
            <span className="text-[10px] font-bold text-yellow-500 bg-yellow-500/10 px-1.5 py-0.5 rounded flex items-center gap-1">
              ★ {rating}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
