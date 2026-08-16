/**
 * Utilitários para gerenciar informações de biblioteca de jogos.
 * Centraliza lógica compartilhada entre componentes de cards.
 */

/**
 * Retorna o label limpo da biblioteca de jogos.
 */
export function getStoreLabel(platform: string): string {
  if (platform.includes('Epic')) return 'Epic Games';

  if (platform.includes('Steam')) return 'Steam';

  if (platform.includes('GOG')) return 'GOG';

  if (platform.includes('Prime')) return 'Amazon Prime';

  if (platform.includes('Amazon')) return 'Amazon Games';

  if (platform.includes('Ubisoft')) return 'Ubisoft';

  if (platform.includes('Legacy')) return 'Legacy Games';

  if (platform.includes('Heroic')) return 'Heroic';

  if (platform.includes('Battle.net')) return 'Battle.net';

  if (platform === 'EA') return 'EA App';

  if (platform.includes('Xbox')) return 'Xbox / Microsoft Store';

  if (platform === 'Indiegala') return 'IndieGala';

  if (platform === 'Indie') return 'Indie';

  return platform.replace('PC, ', '');
}

/**
 * Retorna as classes Tailwind para cor da badge da biblioteca de jogos.
 */
export function getStoreColor(platform: string): string {
  const p = platform.toLowerCase();

  if (p.includes('epic')) return 'bg-[#2a2a2a] text-white'; // cinza-escuro Epic

  if (p.includes('steam')) return 'bg-[#1b2838] text-white'; // azul petróleo Steam

  if (p.includes('prime') || p.includes('amazon'))
    return 'bg-[#ff9900] text-black'; // laranja Amazon

  if (p.includes('legacy')) return 'bg-orange-600 text-white';

  if (p.includes('ubisoft')) return 'bg-[#0d1b2a] text-white'; // azul-marinho Ubisoft

  if (p.includes('heroic')) return 'bg-yellow-600 text-white';

  if (p.includes('gog')) return 'bg-violet-700 text-white';

  if (p.includes('battlenet') || p.includes('battle.net'))
    return 'bg-sky-700 text-white';

  if (p === 'ea') return 'bg-[#ff4747] text-white'; // evita falso positivo em "sea", "beat", etc.

  if (p.includes('xbox')) return 'bg-[#107c10] text-white'; // verde Xbox oficial

  if (p.includes('itch')) return 'bg-[#fa5c5c] text-white';

  if (p.includes('indiegala')) return 'bg-purple-700 text-white'; // antes do 'indie' genérico

  if (p.includes('indie')) return 'bg-pink-600 text-white';

  return 'bg-purple-600 text-white';
}
