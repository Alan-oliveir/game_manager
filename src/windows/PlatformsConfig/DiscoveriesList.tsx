import { GameDiscovery } from '@/types/scanner';

import { GameDiscoveryCard } from './components/GameDiscoveryCard';

interface DiscoveriesListProps {
  discoveries: GameDiscovery[];
  rootFolderPath: string;
}

/**
 * Lista os jogos descobertos pelo scanner local, renderizando um
 * GameDiscoveryCard para cada item.
 */
export function DiscoveriesList({
  discoveries,
  rootFolderPath,
}: Readonly<DiscoveriesListProps>) {
  return (
    <div className="flex flex-col gap-3">
      {discoveries.map(discovery => (
        <GameDiscoveryCard
          key={discovery.id}
          discovery={discovery}
          rootFolderPath={rootFolderPath}
        />
      ))}
    </div>
  );
}
