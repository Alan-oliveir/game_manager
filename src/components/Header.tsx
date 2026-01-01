import { Moon, Plus, Search, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTheme } from "../hooks/useTheme";

interface HeaderProps {
  onAddGame: () => void;
  searchTerm: string;
  onSearchChange: (term: string) => void;
  activeSection: string;
}

export default function Header({
  onAddGame,
  searchTerm,
  onSearchChange,
  activeSection,
}: HeaderProps) {
  const { isDark, toggleTheme } = useTheme();

  const isSearchable = ["libraries", "favorites"].includes(activeSection);

  return (
    <header className="h-16 bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/60 border-b border-border flex items-center justify-between px-4 md:px-6 gap-4 sticky top-0 z-50">
      {/* Search Bar */}
      <div className="flex-1 max-w-xl transition-opacity duration-200">
        <div className="relative group">
          <Search
            className={`absolute left-3 top-1/2 -translate-y-1/2 transition-colors ${
              isSearchable
                ? "text-muted-foreground group-focus-within:text-primary"
                : "text-muted-foreground/40"
            }`}
            size={18}
          />
          <input
            type="text"
            disabled={!isSearchable}
            placeholder={
              isSearchable
                ? "Buscar jogos por nome, gênero ou plataforma..."
                : "Busca indisponível nesta página"
            }
            value={isSearchable ? searchTerm : ""}
            onChange={(e) => onSearchChange(e.target.value)}
            className={`
              w-full pl-9 pr-4 py-2 h-9 rounded-md border text-sm transition-all
              ${
                isSearchable
                  ? "bg-muted/50 hover:bg-muted focus:bg-background border-transparent focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/20 text-foreground placeholder:text-muted-foreground"
                  : "bg-muted/20 border-transparent text-muted-foreground cursor-not-allowed placeholder:text-muted-foreground/40"
              }
            `}
          />
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2">
        <Button
          onClick={onAddGame}
          size="sm"
          className="px-3 md:px-4 shrink-0"
          title="Adicionar Jogo"
        >
          <Plus size={18} />
          <span className="hidden md:inline ml-1">Adicionar</span>
        </Button>

        <Button
          onClick={toggleTheme}
          variant="ghost"
          size="icon"
          className="text-muted-foreground hover:text-foreground shrink-0"
          title="Alternar tema"
        >
          {isDark ? <Sun size={18} /> : <Moon size={18} />}
        </Button>
      </div>
    </header>
  );
}
