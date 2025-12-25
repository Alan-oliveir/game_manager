import { Moon, Plus, Search, Sun } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";

interface HeaderProps {
  onAddGame: () => void;
}

export default function Header({ onAddGame }: HeaderProps) {
  const [isDark, setIsDark] = useState(true); // Começa true para bater com seu print dark

  const toggleTheme = () => {
    setIsDark(!isDark);
    document.documentElement.classList.toggle("dark");
  };

  return (
    <header className="h-16 bg-background border-b border-border flex items-center justify-between px-6 gap-4">
      {/* SEARCH BAR
               max-w-xl: Limita a largura em telas grandes
               flex-1: Ocupa o espaço disponível, mas respeita o gap
            */}
      <div className="flex-1 max-w-xl">
        <div className="relative group">
          <Search
            className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground group-focus-within:text-primary transition-colors"
            size={20}
          />
          <input
            type="text"
            placeholder="Buscar jogos..."
            className="w-full pl-10 pr-4 py-2 bg-muted/50 hover:bg-muted focus:bg-background rounded-lg border border-transparent focus:border-primary
                     focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all
                     text-foreground placeholder:text-muted-foreground"
          />
        </div>
      </div>

      {/* ACTIONS AREA */}
      <div className="flex items-center gap-2 md:gap-3 shrink-0">
        {/* BOTÃO ADAPTATIVO
                    - Telas pequenas: Só ícone (quadrado)
                    - Telas médias (md): Ícone + Texto (retangular)
                */}
        <Button
          onClick={onAddGame}
          size="lg"
          className="px-3 md:px-4"
          title="Adicionar Jogo"
        >
          <Plus size={20} />
          {/* O texto só aparece de 'md' (768px) para cima */}
          <span className="hidden md:inline">Adicionar Jogo</span>
        </Button>

        {/* THEME TOGGLE */}
        <Button
          onClick={toggleTheme}
          variant="ghost"
          size="icon-lg"
          className="bg-muted hover:bg-accent text-muted-foreground hover:text-foreground"
          title="Alternar tema"
        >
          {isDark ? <Sun size={20} /> : <Moon size={20} />}
        </Button>
      </div>
    </header>
  );
}
