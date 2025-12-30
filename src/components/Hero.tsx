import { ReactNode } from "react";
import { ChevronLeft, ChevronRight, Star } from "lucide-react";

interface HeroProps {
    // Dados Básicos
    title: string;
    backgroundUrl?: string | null;
    coverUrl?: string | null;
    genres?: string[];
    rating?: number;

    // Slots de Conteúdo (Composição)
    badges?: ReactNode; // Pílulas acima do título (Em Alta, Sugestão, etc)
    actions?: ReactNode; // Botões principais (Jogar, Wishlist, Detalhes)

    // Navegação (Opcional)
    onNext?: () => void;
    onPrev?: () => void;
    showNavigation?: boolean;
}

export default function Hero({
                                 title,
                                 backgroundUrl,
                                 coverUrl,
                                 genres = [],
                                 rating,
                                 badges,
                                 actions,
                                 onNext,
                                 onPrev,
                                 showNavigation = false,
                             }: HeroProps) {
    return (
        <div className="relative h-125 bg-background group/hero">
            {/* 1. BACKGROUND (Blur) */}
            <div
                className="absolute inset-0 bg-cover bg-center transition-all duration-700"
                style={{
                    backgroundImage: `url(${backgroundUrl || coverUrl})`,
                    filter: "blur(20px) brightness(0.25)",
                }}
            />
            <div className="absolute inset-0 bg-linear-to-t from-background via-transparent to-transparent" />

            {/* 2. NAVEGAÇÃO (Setas) */}
            {showNavigation && onPrev && onNext && (
                <>
                    <button
                        onClick={onPrev}
                        className="absolute left-4 top-1/2 -translate-y-1/2 p-2 bg-black/40 hover:bg-black/60 rounded-full text-white backdrop-blur-sm transition z-20"
                    >
                        <ChevronLeft size={24} />
                    </button>
                    <button
                        onClick={onNext}
                        className="absolute right-4 top-1/2 -translate-y-1/2 p-2 bg-black/40 hover:bg-black/60 rounded-full text-white backdrop-blur-sm transition z-20"
                    >
                        <ChevronRight size={24} />
                    </button>
                </>
            )}

            {/* 3. CONTEÚDO PRINCIPAL */}
            <div className="relative h-full flex items-center px-8 max-w-7xl mx-auto z-10">
                <div
                    className="flex flex-col md:flex-row items-center gap-8 w-full animate-in fade-in duration-500"
                    key={title}
                >
                    {/* Capa */}
                    <img
                        src={coverUrl || ""}
                        alt={title}
                        className="w-64 md:w-80 aspect-3/4 object-cover rounded-lg shadow-2xl border border-white/10"
                    />

                    {/* Coluna de Informações */}
                    <div className="flex-1 space-y-4 text-center md:text-left">

                        {/* Slot de Badges */}
                        {badges && <div className="mb-2">{badges}</div>}

                        <h1 className="text-4xl md:text-5xl font-bold text-white leading-tight">
                            {title}
                        </h1>

                        {/* Lista de Gêneros */}
                        {genres.length > 0 && (
                            <div className="flex flex-wrap gap-2 justify-center md:justify-start">
                                {genres.map((g) => (
                                    <span
                                        key={g}
                                        className="px-3 py-1 bg-white/10 rounded-full text-xs text-white"
                                    >
                    {g}
                  </span>
                                ))}
                            </div>
                        )}

                        {/* Avaliação (Opcional) */}
                        {rating && (
                            <div className="flex items-center gap-6 justify-center md:justify-start pt-2">
                                <div className="flex items-center gap-2">
                                    <Star className="text-yellow-400 fill-yellow-400" size={24} />
                                    <div>
                    <span className="text-2xl font-bold text-white">
                      {rating}
                    </span>
                                        <span className="text-white/50 text-sm ml-1">/ 5.0</span>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Slot de Ações (Botões) */}
                        {actions && (
                            <div className="flex gap-3 justify-center md:justify-start mt-6">
                                {actions}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}