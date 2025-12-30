import {
  Heart,
  Home,
  Library,
  Settings,
  TrendingUp,
  ShoppingCart, PlayIcon,
} from "lucide-react";
import { Game } from "../types";

interface SidebarProps {
  activeSection: string;
  onSectionChange: (section: string) => void;
  games: Game[];
}

const menuItems = [
  { id: "home", label: "Início", icon: Home },
  { id: "libraries", label: "Biblioteca", icon: Library },
  { id: "favorites", label: "Favoritos", icon: Heart },
  { id: "playlist", label: "Playlist", icon: PlayIcon },
  { id: "trending", label: "Em Alta", icon: TrendingUp },
  { id: "wishlist", label: "Lista de Desejos", icon: ShoppingCart },
  { id: "settings", label: "Configurações", icon: Settings },
];

export default function Sidebar({
  activeSection,
  onSectionChange,
  games,
}: SidebarProps) {
  return (
    <aside className="w-64 bg-sidebar border-r border-sidebar-border h-screen flex flex-col">
      {/* Logo/Header */}
      <div className="p-6 border-b border-sidebar-border">
        <h1 className="text-2xl font-bold text-sidebar-foreground">
          Playlite
        </h1>
      </div>

      {/* Menu Items */}
      <nav className="flex-1 p-4 space-y-2">
        {menuItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeSection === item.id;

          return (
            <button
              key={item.id}
              onClick={() => onSectionChange(item.id)}
              className={`
                w-full flex items-center gap-3 px-4 py-3 rounded-lg
                transition-all duration-200
                ${
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground font-semibold"
                    : "text-sidebar-foreground hover:bg-sidebar-accent/50"
                }
              `}
            >
              <Icon size={20} />
              <span>{item.label}</span>
            </button>
          );
        })}
      </nav>

      {/* User Info */}
      <div className="p-4 border-t border-sidebar-border">
        <div className="flex items-center gap-3 px-4 py-3">
          <div className="w-10 h-10 rounded-full bg-linear-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white font-bold">
            U
          </div>
          <div className="flex-1">
            <p className="text-sm font-semibold text-sidebar-foreground">
              Usuário
            </p>
            <p className="text-xs text-muted-foreground">
              Biblioteca: {games.length} {games.length === 1 ? "jogo" : "jogos"}
            </p>
          </div>
        </div>
      </div>
    </aside>
  );
}
