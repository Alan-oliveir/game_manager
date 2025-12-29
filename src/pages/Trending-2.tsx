import React, { useState } from 'react';

interface Game {
  id: number;
  name: string;
  cover_url: string;
  rating: number;
  genres: string[];
}

interface Deal {
  id: number;
  name: string;
  cover_url: string;
  normalPrice: number;
  salePrice: number;
  discount: number;
  store: string;
}

interface UpcomingGame {
  id: number;
  name: string;
  cover_url: string;
  releaseDate: string;
  genres: string[];
}

interface MissingGame {
  id: number;
  name: string;
  cover_url: string;
  price: number;
}

interface Collection {
  series: string;
  owned: string[];
  missing: MissingGame[];
}

const MOCK_TRENDING: Game[] = [
  {
    id: 1,
    name: "Black Myth: Wukong",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7yru.jpg",
    rating: 4.5,
    genres: ["Action", "RPG"],
  },
  {
    id: 2,
    name: "Palworld",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7kcq.jpg",
    rating: 4.3,
    genres: ["Indie", "Adventure"],
  },
  {
    id: 3,
    name: "Senua's Saga",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7g8v.jpg",
    rating: 4.6,
    genres: ["Action", "Adventure"],
  },
  {
    id: 4,
    name: "Dragon's Dogma 2",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7kku.jpg",
    rating: 4.2,
    genres: ["Action", "RPG"],
  },
  {
    id: 5,
    name: "The Finals",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7b7g.jpg",
    rating: 4.0,
    genres: ["Shooter", "Action"],
  },
  {
    id: 6,
    name: "Tekken 8",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7ilh.jpg",
    rating: 4.7,
    genres: ["Fighting"],
  },
];

const MOCK_DEALS: Deal[] = [
  {
    id: 201,
    name: "The Witcher 3",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co5vt4.jpg",
    normalPrice: 39.99,
    salePrice: 9.99,
    discount: 75,
    store: "Steam",
  },
  {
    id: 202,
    name: "Cyberpunk 2077",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co2of5.jpg",
    normalPrice: 59.99,
    salePrice: 29.99,
    discount: 50,
    store: "GOG",
  },
  {
    id: 203,
    name: "Red Dead Redemption 2",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co1q1f.jpg",
    normalPrice: 59.99,
    salePrice: 19.99,
    discount: 67,
    store: "Epic",
  },
  {
    id: 204,
    name: "God of War",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co1tmu.jpg",
    normalPrice: 49.99,
    salePrice: 24.99,
    discount: 50,
    store: "Steam",
  },
];

const MOCK_UPCOMING: UpcomingGame[] = [
  {
    id: 301,
    name: "GTA VI",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co87vf.jpg",
    releaseDate: "2025",
    genres: ["Action", "Adventure"],
  },
  {
    id: 302,
    name: "Monster Hunter Wilds",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7zh3.jpg",
    releaseDate: "2025",
    genres: ["Action", "RPG"],
  },
  {
    id: 303,
    name: "Death Stranding 2",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co7zct.jpg",
    releaseDate: "2025",
    genres: ["Action", "Adventure"],
  },
  {
    id: 304,
    name: "Avowed",
    cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co61m9.jpg",
    releaseDate: "Fev 2025",
    genres: ["RPG", "Adventure"],
  },
];

const MOCK_COLLECTION: Collection[] = [
  {
    series: "The Witcher",
    owned: ["The Witcher 3"],
    missing: [
      {
        id: 401,
        name: "The Witcher 1",
        cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co2wgx.jpg",
        price: 9.99,
      },
      {
        id: 402,
        name: "The Witcher 2",
        cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co2wh4.jpg",
        price: 19.99,
      },
    ],
  },
  {
    series: "Dark Souls",
    owned: ["Dark Souls III"],
    missing: [
      {
        id: 403,
        name: "Dark Souls Remastered",
        cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co1vcn.jpg",
        price: 39.99,
      },
      {
        id: 404,
        name: "Dark Souls II",
        cover_url: "https://images.igdb.com/igdb/image/upload/t_cover_big/co1vcg.jpg",
        price: 39.99,
      },
    ],
  },
];

const Trending: React.FC = () => {
  const [heroIndex, setHeroIndex] = useState(0);
  const [selectedGenre, setSelectedGenre] = useState("all");

  const currentHero = MOCK_TRENDING[heroIndex];

  const filteredGames =
    selectedGenre === "all"
      ? MOCK_TRENDING
      : MOCK_TRENDING.filter((g) => g.genres.includes(selectedGenre));

  const allGenres = [...new Set(MOCK_TRENDING.flatMap((g) => g.genres))];

  return (
    <div className="min-h-screen bg-gray-950 text-white overflow-y-auto">
      {/* Hero Section */}
      <div className="relative h-96">
        <div
          className="absolute inset-0 bg-cover bg-center"
          style={{
            backgroundImage: `url(${currentHero.cover_url})`,
            filter: "blur(20px) brightness(0.25)",
          }}
        />
        <div className="absolute inset-0 bg-gradient-to-t from-gray-950 via-transparent to-transparent" />

        {MOCK_TRENDING.length > 1 && (
          <>
            <button
              onClick={() =>
                setHeroIndex((heroIndex - 1 + MOCK_TRENDING.length) % MOCK_TRENDING.length)
              }
              className="absolute left-4 top-1/2 -translate-y-1/2 z-20 p-2 bg-black/40 hover:bg-black/60 rounded-full transition"
            >
              ◀
            </button>
            <button
              onClick={() => setHeroIndex((heroIndex + 1) % MOCK_TRENDING.length)}
              className="absolute right-4 top-1/2 -translate-y-1/2 z-20 p-2 bg-black/40 hover:bg-black/60 rounded-full transition"
            >
              ▶
            </button>
          </>
        )}

        <div className="relative h-full flex items-center px-8 max-w-7xl mx-auto z-10">
          <div className="flex items-center gap-8">
            <img
              src={currentHero.cover_url}
              alt={currentHero.name}
              className="w-64 h-80 object-cover rounded-lg shadow-2xl border border-white/10"
            />
            <div className="flex-1 space-y-4">
              <div className="inline-flex items-center gap-2 px-3 py-1 bg-orange-500/20 text-orange-400 rounded-full text-sm font-medium border border-orange-500/20">
                🔥 EM ALTA
              </div>
              <h1 className="text-5xl font-bold">{currentHero.name}</h1>
              <div className="flex items-center gap-2 flex-wrap">
                {currentHero.genres.map((g) => (
                  <span
                    key={g}
                    className="px-3 py-1 bg-white/10 rounded-full text-xs"
                  >
                    {g}
                  </span>
                ))}
              </div>
              <div className="flex items-center gap-2">
                ⭐
                <span className="text-2xl font-bold">{currentHero.rating}</span>
                <span className="text-white/50 text-sm">/ 5.0</span>
              </div>
              <div className="flex gap-3">
                <button className="px-6 py-3 bg-red-600 hover:bg-red-700 rounded-lg font-semibold transition flex items-center gap-2">
                  ❤️ Lista de Desejos
                </button>
                <button className="px-6 py-3 bg-white/10 hover:bg-white/20 border border-white/20 rounded-lg font-semibold transition flex items-center gap-2">
                  🔗 Ver Detalhes
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Filtro */}
      <div className="sticky top-0 z-20 bg-gray-950/80 backdrop-blur-md border-b border-gray-800 p-4 shadow-lg">
        <div className="max-w-7xl mx-auto flex items-center gap-4">
          <span className="text-sm text-gray-400">🔍 Filtrar:</span>
          <select
            value={selectedGenre}
            onChange={(e) => setSelectedGenre(e.target.value)}
            className="px-3 py-2 bg-gray-900 text-white rounded-lg text-sm border border-gray-800 focus:ring-2 focus:ring-blue-500 cursor-pointer"
          >
            <option value="all">Todos os Gêneros</option>
            {allGenres.map((g) => (
              <option key={g} value={g}>
                {g}
              </option>
            ))}
          </select>
          <span className="ml-auto text-xs text-gray-400">
            {filteredGames.length} sugestões disponíveis
          </span>
        </div>
      </div>

      <div className="p-8 max-w-7xl mx-auto space-y-12">
        {/* Mais Sugestões */}
        <section>
          <h2 className="text-2xl font-bold mb-4 flex items-center gap-2">
            📈 Mais Sugestões (Recomendadas)
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
            {filteredGames.map((game, i) => (
              <div
                key={game.id}
                className="group relative bg-gray-900 rounded-xl overflow-hidden border border-gray-800 hover:shadow-xl transition cursor-pointer"
              >
                {i < 3 && (
                  <div className="absolute top-2 left-2 z-10 bg-purple-600/90 text-white text-xs font-bold px-2 py-1 rounded-full flex items-center gap-1">
                    ✨ TOP PICK
                  </div>
                )}
                <div className="aspect-video overflow-hidden relative">
                  <img
                    src={game.cover_url}
                    alt={game.name}
                    className="w-full h-full object-cover group-hover:scale-110 transition-transform duration-500"
                  />
                  <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                    <button className="px-3 py-1 bg-white/90 hover:bg-white text-black rounded text-xs font-semibold">
                      ❤️ Desejos
                    </button>
                    <button className="px-3 py-1 bg-white/90 hover:bg-white text-black rounded text-xs font-semibold">
                      🔗 Detalhes
                    </button>
                  </div>
                </div>
                <div className="p-3">
                  <div className="flex justify-between items-start gap-2 mb-1">
                    <h3 className="font-semibold text-sm truncate">{game.name}</h3>
                    <div className="flex items-center gap-1 shrink-0 bg-yellow-500/10 px-1.5 py-0.5 rounded text-xs text-yellow-500 font-bold">
                      ⭐ {game.rating}
                    </div>
                  </div>
                  <div className="text-xs text-gray-400 truncate">
                    {game.genres.join(", ")}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Ofertas Imperdíveis */}
        <section className="bg-gradient-to-r from-green-500/10 to-emerald-500/10 border border-green-500/20 rounded-xl p-6">
          <h2 className="text-2xl font-bold mb-6 flex items-center gap-2">
            💰 Ofertas Imperdíveis
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {MOCK_DEALS.map((deal) => (
              <div
                key={deal.id}
                className="group bg-gray-900/50 border border-gray-800 rounded-xl overflow-hidden hover:shadow-xl hover:border-green-500/50 transition cursor-pointer"
              >
                <div className="aspect-[3/4] relative overflow-hidden">
                  <img
                    src={deal.cover_url}
                    alt={deal.name}
                    className="w-full h-full object-cover group-hover:scale-105 transition-transform"
                  />
                  <div className="absolute top-2 right-2 bg-red-600/90 text-white text-sm font-bold px-3 py-1 rounded-full">
                    -{deal.discount}%
                  </div>
                </div>
                <div className="p-3">
                  <h3 className="font-semibold text-sm truncate mb-2">{deal.name}</h3>
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <span className="text-gray-500 line-through text-xs">
                        R$ {deal.normalPrice.toFixed(2)}
                      </span>
                      <span className="text-green-400 font-bold text-lg">
                        R$ {deal.salePrice.toFixed(2)}
                      </span>
                    </div>
                  </div>
                  <div className="text-xs text-gray-400">🏪 {deal.store}</div>
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Complete Sua Coleção */}
        <section>
          <h2 className="text-2xl font-bold mb-6 flex items-center gap-2">
            📦 Complete Sua Coleção
          </h2>
          <div className="space-y-6">
            {MOCK_COLLECTION.map((collection) => (
              <div
                key={collection.series}
                className="bg-gray-900 border border-gray-800 rounded-xl p-6"
              >
                <div className="flex items-center justify-between mb-4">
                  <div>
                    <h3 className="text-xl font-semibold">{collection.series}</h3>
                    <p className="text-sm text-gray-400">
                      Você tem: {collection.owned.join(", ")}
                    </p>
                  </div>
                  <div className="text-sm text-gray-400">
                    Faltam {collection.missing.length} jogos
                  </div>
                </div>
                <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
                  {collection.missing.map((game) => (
                    <div
                      key={game.id}
                      className="group cursor-pointer"
                    >
                      <div className="aspect-[3/4] bg-gray-800 rounded-lg mb-2 overflow-hidden relative">
                        <img
                          src={game.cover_url}
                          alt={game.name}
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform"
                        />
                        <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                          <button className="px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded font-semibold text-sm">
                            🛒 Comprar
                          </button>
                        </div>
                      </div>
                      <h4 className="text-sm font-medium truncate mb-1">{game.name}</h4>
                      <p className="text-sm font-bold text-green-400">
                        R$ {game.price.toFixed(2)}
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Lançamentos Aguardados */}
        <section>
          <h2 className="text-2xl font-bold mb-6 flex items-center gap-2">
            🚀 Lançamentos Aguardados
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {MOCK_UPCOMING.map((game) => (
              <div
                key={game.id}
                className="group bg-gray-900 border border-gray-800 rounded-xl overflow-hidden hover:shadow-xl hover:border-blue-500/50 transition cursor-pointer"
              >
                <div className="aspect-[3/4] relative overflow-hidden">
                  <img
                    src={game.cover_url}
                    alt={game.name}
                    className="w-full h-full object-cover group-hover:scale-105 transition-transform"
                  />
                  <div className="absolute top-2 left-2 bg-blue-600/90 text-white text-xs font-bold px-2 py-1 rounded-full">
                    📅 {game.releaseDate}
                  </div>
                </div>
                <div className="p-3">
                  <h3 className="font-semibold text-sm truncate mb-1">{game.name}</h3>
                  <div className="text-xs text-gray-400 truncate">
                    {game.genres.join(", ")}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
};

export default Trending;
