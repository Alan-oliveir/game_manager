import { ReactNode } from "react";
import { Play } from "lucide-react";
import { cn } from "@/lib/utils";

interface StandardGameCardProps {
  title: string;
  coverUrl?: string | null;
  subtitle?: string;
  badge?: string;
  rating?: number;

  // Ações
  onClick?: () => void;
  actions?: ReactNode;
  className?: string;
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
}: StandardGameCardProps) {
  return (
    <div
      className={cn(
        "group relative bg-card rounded-xl overflow-hidden border border-border hover:shadow-lg transition-all hover:-translate-y-1 cursor-pointer flex flex-col h-full",
        className
      )}
      onClick={onClick}
    >
      {/* Imagem (Aspect Ratio fixo) */}
      <div className="aspect-[3/4] bg-muted relative overflow-hidden">
        {coverUrl ? (
          <img
            src={coverUrl}
            alt={title}
            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-muted-foreground text-xs p-2 text-center">
            Sem Imagem
          </div>
        )}

        {/* Overlay de Ações (Hover) */}
        <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2 p-4">
          {actions || (
            <Play className="text-white fill-white opacity-80" size={32} />
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
          <span className="text-xs text-muted-foreground truncate max-w-[120px]">
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
