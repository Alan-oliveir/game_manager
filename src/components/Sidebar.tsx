import {
  Gamepad2,
  Heart,
  Home,
  Library,
  Settings,
  ShoppingCart,
  TrendingUp,
  Menu,
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
  { id: "playlist", label: "Playlist", icon: Gamepad2 },
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
    <aside className="w-20 md:w-64 bg-sidebar border-r border-sidebar-border h-screen flex flex-col transition-all duration-300 ease-in-out shrink-0">
      {/* Logo/Header */}
      <div className="h-16 flex items-center justify-center md:justify-start md:px-6 border-b border-sidebar-border">
        {/* Mostra Texto em telas grandes */}
        <h1 className="hidden md:block text-2xl font-bold text-sidebar-foreground truncate">
          Playlite
        </h1>
        {/* Mostra Ícone em telas pequenas */}
        <div className="md:hidden text-sidebar-foreground">
          <Menu size={24} />
        </div>
      </div>

      {/* Menu Items */}
      <nav className="flex-1 p-3 md:p-4 space-y-2 overflow-y-auto custom-scrollbar">
        {menuItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeSection === item.id;

          return (
            <button
              key={item.id}
              onClick={() => onSectionChange(item.id)}
              title={item.label} // Tooltip nativo para quando estiver colapsado
              className={`
                w-full flex items-center justify-center md:justify-start gap-3 px-3 py-3 rounded-lg
                transition-all duration-200
                ${
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground font-semibold"
                    : "text-sidebar-foreground hover:bg-sidebar-accent/50"
                }
              `}
            >
              {/* Ícone fixo */}
              <Icon size={22} className="shrink-0" />
              {/* Texto escondido no mobile, visível no desktop */}
              <span className="hidden md:block truncate text-lg">
                {item.label}
              </span>
            </button>
          );
        })}
      </nav>

      {/* User Info */}
      <div className="p-3 md:p-4 border-t border-sidebar-border">
        <div className="flex items-center justify-center md:justify-start gap-3 px-1 md:px-4 py-2">
          <div className="w-8 h-8 md:w-10 md:h-10 shrink-0 rounded-full bg-linear-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white font-bold text-sm md:text-base">
            U
          </div>

          {/* Info escondida no mobile */}
          <div className="hidden md:block flex-1 overflow-hidden">
            <p className="text-base text-sidebar-foreground truncate">
              Usuário
            </p>
            <p className="text-sm text-muted-foreground truncate">
              {games.length} {games.length === 1 ? "jogo" : "jogos"}
            </p>
          </div>
        </div>
      </div>
    </aside>
  );
}
