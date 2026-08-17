/**
 * Utilitários para gerenciar informações de biblioteca de jogos.
 * Centraliza lógica compartilhada entre componentes de cards.
 */

/**
 * Retorna o label limpo da biblioteca de jogos.
 */
export function getStoreLabel(store: string): string {
  if (store.includes('Epic')) return 'Epic Games';

  if (store.includes('Steam')) return 'Steam';

  if (store.includes('GOG')) return 'GOG';

  if (store.includes('Prime')) return 'Amazon Prime';

  if (store.includes('Amazon')) return 'Amazon Games';

  if (store.includes('Ubisoft')) return 'Ubisoft';

  if (store.includes('Legacy')) return 'Legacy Games';

  if (store.includes('Heroic')) return 'Heroic';

  if (store.includes('Battle.net')) return 'Battle.net';

  if (store === 'EA') return 'EA App';

  if (store.includes('Xbox')) return 'Xbox / Microsoft Store';

  if (store === 'Indiegala') return 'IndieGala';

  if (store === 'Indie') return 'Indie';

  return store.replace('PC, ', '');
}

/**
 * Retorna as classes Tailwind para cor da badge da biblioteca de jogos.
 */
export function getStoreColor(store: string): string {
  const s = store.toLowerCase();

  if (s.includes('epic')) return 'bg-[#2a2a2a] text-white'; // cinza-escuro Epic

  if (s.includes('steam')) return 'bg-[#1b2838] text-white'; // azul petróleo Steam

  if (s.includes('prime') || s.includes('amazon'))
    return 'bg-[#ff9900] text-black'; // laranja Amazon

  if (s.includes('legacy')) return 'bg-orange-600 text-white';

  if (s.includes('ubisoft')) return 'bg-[#0d1b2a] text-white'; // azul-marinho Ubisoft

  if (s.includes('heroic')) return 'bg-yellow-600 text-white';

  if (s.includes('gog')) return 'bg-violet-700 text-white';

  if (s.includes('battlenet') || s.includes('battle.net'))
    return 'bg-sky-700 text-white';

  if (s === 'ea') return 'bg-[#ff4747] text-white'; // evita falso positivo em "sea", "beat", etc.

  if (s.includes('xbox')) return 'bg-[#107c10] text-white'; // verde Xbox oficial

  if (s.includes('itch')) return 'bg-[#fa5c5c] text-white';

  if (s.includes('indiegala')) return 'bg-purple-700 text-white'; // antes do 'indie' genérico

  if (s.includes('indie')) return 'bg-pink-600 text-white';

  return 'bg-purple-600 text-white';
}
