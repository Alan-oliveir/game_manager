import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { WishlistGame } from "../types";
import {
  RefreshCw,
  DollarSign,
  ExternalLink,
  Loader2,
  ShoppingCart,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";

export default function Wishlist() {
  const [games, setGames] = useState<WishlistGame[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const fetchWishlist = async () => {
    try {
      const result = await invoke<WishlistGame[]>("get_wishlist");
      setGames(result);
    } catch (error) {
      console.error("Erro ao buscar wishlist:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRemove = async (id: string, name: string) => {
    if (!confirm(`Remover ${name} da lista de desejos?`)) return;

    try {
      await invoke("remove_from_wishlist", { id });
      setGames((prev) => prev.filter((g) => g.id !== id));
    } catch (error) {
      console.error(error);
      alert("Erro ao remover jogo.");
    }
  };

  const handleRefreshPrices = async () => {
    setIsRefreshing(true);
    try {
      await invoke("refresh_prices");
      await fetchWishlist();
    } catch (error) {
      console.error(error);
      alert("Erro ao atualizar preços.");
    } finally {
      setIsRefreshing(false);
    }
  };

  const handleOpenLink = async (url: string) => {
    try {
      await open(url);
    } catch (error) {
      console.error("Erro ao abrir link:", error);
      alert("Erro ao abrir o link no navegador.");
    }
  };

  useEffect(() => {
    fetchWishlist();
  }, []);

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <Loader2 className="animate-spin" />
      </div>
    );
  }

  if (games.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground">
        <div className="bg-muted p-6 rounded-full mb-4">
          <ShoppingCart className="w-12 h-12 opacity-20" />
        </div>
        <h3 className="text-lg font-medium">Sua lista está vazia</h3>
        <p className="text-sm">
          Vá para a aba "Em Alta" para descobrir novos jogos.
        </p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-8">
      <div className="mb-8 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-red-500/10 rounded-lg text-red-500">
            <ShoppingCart size={24} />
          </div>
          <div>
            <h1 className="text-2xl font-bold">Lista de Desejos</h1>
            <p className="text-muted-foreground text-sm">
              Monitorando {games.length} {games.length === 1 ? "jogo" : "jogos"}
            </p>
          </div>
        </div>
        <Button
          onClick={handleRefreshPrices}
          disabled={isRefreshing || games.length === 0}
          variant="outline"
        >
          <RefreshCw
            className={`mr-2 h-4 w-4 ${isRefreshing ? "animate-spin" : ""}`}
          />
          {isRefreshing ? "Buscando Ofertas..." : "Atualizar Preços"}
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {games.map((game) => (
          <div
            key={game.id}
            className="group bg-card border border-border rounded-xl overflow-hidden hover:shadow-lg transition-all flex flex-col"
          >
            <div className="relative aspect-video bg-muted overflow-hidden">
              {game.cover_url ? (
                <img
                  src={game.cover_url}
                  alt={game.name}
                  className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center text-muted-foreground">
                  Sem Imagem
                </div>
              )}

              <button
                onClick={() => handleRemove(game.id, game.name)}
                className="absolute top-2 right-2 p-2 bg-black/60 text-white rounded-full opacity-0 group-hover:opacity-100 transition-opacity hover:bg-red-600"
                title="Remover da lista"
              >
                <Trash2 size={16} />
              </button>
            </div>

            <div className="p-4 flex flex-col flex-1">
              <h3 className="font-semibold line-clamp-1 mb-1" title={game.name}>
                {game.name}
              </h3>
              <p className="text-xs text-muted-foreground mb-4">
                Adicionado em {new Date(game.added_at).toLocaleDateString()}
              </p>

              <div className="mt-auto flex items-center justify-between gap-2">
                <div className="text-sm font-medium">
                  {game.current_price !== null ? (
                    <div className="flex flex-col">
                      <span
                        className={
                          game.on_sale
                            ? "text-green-500 font-bold"
                            : "text-foreground"
                        }
                      >
                        US$ {game.current_price.toFixed(2)}
                      </span>
                      {game.on_sale && (
                        <span className="text-xs text-green-500/80">
                          OFERTA!
                        </span>
                      )}
                    </div>
                  ) : (
                    <span className="text-muted-foreground text-xs flex items-center gap-1">
                      <DollarSign size={12} /> Aguardando...
                    </span>
                  )}
                </div>

                <Button
                  size="sm"
                  variant="outline"
                  className="h-8 gap-2"
                  disabled={!game.store_url}
                  onClick={() => {
                    if (game.store_url) {
                      handleOpenLink(game.store_url);
                    }
                  }}
                >
                  <ExternalLink size={14} />
                  Loja
                </Button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
