import {
  Gamepad2,
  Heart,
  Home,
  Library,
  LucideIcon,
  Settings,
  ShoppingCart,
  TrendingUp,
} from 'lucide-react';

/**
 * Item de menu da navegação lateral
 */
export interface MenuItem {
  id: string;
  label: string;
  icon: LucideIcon;
}

/**
 * Configuração dos itens do menu de navegação
 * Usado no componente Sidebar
 */
export const MENU_ITEMS: MenuItem[] = [
  { id: 'home', label: 'Início', icon: Home },
  { id: 'libraries', label: 'Biblioteca', icon: Library },
  { id: 'favorites', label: 'Favoritos', icon: Heart },
  { id: 'playlist', label: 'Playlist', icon: Gamepad2 },
  { id: 'trending', label: 'Em Alta', icon: TrendingUp },
  { id: 'wishlist', label: 'Lista de Desejos', icon: ShoppingCart },
  { id: 'settings', label: 'Configurações', icon: Settings },
];

/**
 * IDs das seções que possuem funcionalidade de busca
 */
export const SEARCHABLE_SECTIONS = ['libraries', 'favorites', 'wishlist'];

/**
 * IDs das seções que possuem funcionalidade de filtro (adulto, duplicatas, não-instalados) e agrupamento
 */
export const FILTERABLE_SECTIONS = new Set(['libraries', 'favorites']);
