import {
  ExternalLink,
  Loader2,
  Plus,
  RefreshCw,
  ShoppingCart,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { useWishlist } from "../hooks/useWishlist";
import { openExternalLink } from "../utils/navigation";
import StandardGameCard from "@/components/StandardGameCard";
import { toast } from "sonner";
import AddWishlistModal from "@/components/AddWishlistModal";
import { useState } from "react";

export default function Wishlist() {
  const {
    games,
    isLoading,
    isRefreshing,
    removeGame,
    refreshPrices,
    refreshList,
  } = useWishlist();

  const [showAddModal, setShowAddModal] = useState(false);

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
        <ShoppingCart className="w-16 h-16 mb-4 opacity-20" />
        <h3 className="text-lg font-medium">Sua lista está vazia</h3>
        <p className="text-sm">
          Vá para a aba "Em Alta" para descobrir novos jogos.
        </p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto custom-scrollbar p-8">
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

        <Button onClick={() => setShowAddModal(true)} variant="outline">
          <Plus className="mr-2 h-4 w-4" /> Adicionar na Lista
        </Button>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
        {games.map((game) => {
          let priceDisplay = "Aguardando preço...";

          if (
            game.localized_price !== null &&
            game.localized_price !== undefined
          ) {
            const currency =
              game.localized_currency === "BRL"
                ? "R$"
                : game.localized_currency || "R$";
            priceDisplay = `${currency} ${game.localized_price.toFixed(2)}`;
          } else if (
            game.current_price !== null &&
            game.current_price !== undefined
          ) {
            priceDisplay = `US$ ${game.current_price.toFixed(2)}`;
          }

          const targetUrl = game.steam_app_id
            ? `https://store.steampowered.com/app/${game.steam_app_id}/`
            : game.store_url;

          return (
            <StandardGameCard
              key={game.id}
              title={game.name}
              coverUrl={game.cover_url}
              subtitle={priceDisplay}
              badge={game.on_sale ? "OFERTA!" : undefined}
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
                    // Desabilita apenas se não tiver nem ID da Steam nem URL da loja
                    disabled={!targetUrl}
                    onClick={() => {
                      if (targetUrl) openExternalLink(targetUrl);
                    }}
                    title={
                      game.steam_app_id ? "Abrir na Steam" : "Ir para Loja"
                    }
                  >
                    <ExternalLink size={16} />
                  </Button>
                </>
              }
            />
          );
        })}
      </div>

      <AddWishlistModal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        onSuccess={refreshList}
      />
    </div>
  );
}
