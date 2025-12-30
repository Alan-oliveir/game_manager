import {
  RefreshCw,
  ShoppingCart,
  Trash2,
  ExternalLink,
  Loader2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { useWishlist } from "../hooks/useWishlist";
import { openExternalLink } from "../utils/navigation";
import StandardGameCard from "@/components/StandardGameCard";
import {toast} from "sonner";

export default function Wishlist() {
  const { games, isLoading, isRefreshing, removeGame, refreshPrices } =
    useWishlist();

  const handleRemoveClick = async (id: string, name: string) => {
    if (!confirm(`Remover ${name} da lista de desejos?`)) return;
    try {
      await removeGame(id);
    } catch {
      toast.error("Erro ao remover jogo.");
    }
  };

  const handleRefreshClick = async () => {
    try {
      await refreshPrices();
    } catch {
      toast.error("Erro ao atualizar preços.");
    }
  };

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
      <div className="mb-6 flex items-center justify-between">
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
          onClick={handleRefreshClick}
          disabled={isRefreshing || games.length === 0}
          variant="outline"
        >
          <RefreshCw
            className={`mr-2 h-4 w-4 ${isRefreshing ? "animate-spin" : ""}`}
          />
          {isRefreshing ? "Buscando Ofertas..." : "Atualizar Preços"}
        </Button>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
        {games.map((game) => {
          // Formatação do Preço para o subtítulo
          const priceDisplay =
            game.current_price !== null
              ? `US$ ${game.current_price.toFixed(2)}`
              : "Aguardando preço...";

          return (
            <StandardGameCard
              key={game.id}
              title={game.name}
              coverUrl={game.cover_url}
              subtitle={priceDisplay}
              badge={game.on_sale ? "OFERTA!" : undefined}
              // Ações no Hover: Remover e Link Loja
              actions={
                <>
                  <Button
                    size="icon"
                    variant="destructive"
                    className="rounded-full h-10 w-10 shadow-lg"
                    onClick={() => handleRemoveClick(game.id, game.name)}
                    title="Remover"
                  >
                    <Trash2 size={16} />
                  </Button>

                  <Button
                    size="icon"
                    variant="secondary"
                    className="rounded-full h-10 w-10 shadow-lg"
                    disabled={!game.store_url}
                    onClick={() => {
                      if (game.store_url) openExternalLink(game.store_url);
                    }}
                    title="Ir para Loja"
                  >
                    <ExternalLink size={16} />
                  </Button>
                </>
              }
            />
          );
        })}
      </div>
    </div>
  );
}
