import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Loader2, Search, Plus, ImageOff } from "lucide-react";
import { toast } from "sonner";
import {
  wishlistService,
  SteamSearchResult,
} from "../services/wishlistService";

interface AddWishlistModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export default function AddWishlistModal({
  isOpen,
  onClose,
  onSuccess,
}: AddWishlistModalProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SteamSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [addingId, setAddingId] = useState<number | null>(null);

  const handleSearch = async () => {
    if (!query.trim()) return;
    setLoading(true);
    try {
      const data = await wishlistService.searchWishlistGame(query);
      setResults(data);
    } catch (e) {
      toast.error("Erro ao buscar jogos");
    } finally {
      setLoading(false);
    }
  };

  const handleAdd = async (game: SteamSearchResult) => {
    setAddingId(game.id);
    try {
      await wishlistService.addToWishlist(game);
      toast.success("Adicionado à lista!");
      onSuccess();
      onClose();
      setResults([]);
      setQuery("");
    } catch (e) {
      toast.error("Erro ao adicionar jogo");
    } finally {
      setAddingId(null);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Adicionar à Lista de Desejos</DialogTitle>
        </DialogHeader>

        <div className="flex gap-2 my-2">
          <Input
            placeholder="Nome do jogo..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          />
          <Button onClick={handleSearch} disabled={loading}>
            {loading ? (
              <Loader2 className="animate-spin" />
            ) : (
              <Search size={18} />
            )}
          </Button>
        </div>

        <div className="max-h-75 overflow-y-auto custom-scrollbar space-y-2">
          {results.map((game) => (
            <div
              key={game.id}
              className="flex items-center gap-3 p-2 rounded-lg hover:bg-muted/50 border border-transparent hover:border-border transition-colors mr-2"
            >
              <div className="w-12 h-12 shrink-0 bg-background rounded overflow-hidden">
                {game.tiny_image ? (
                  <img
                    src={game.tiny_image}
                    alt=""
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center">
                    <ImageOff size={16} />
                  </div>
                )}
              </div>
              <div className="flex-1 min-w-0">
                <p className="font-medium truncate text-sm">{game.name}</p>
                <p className="text-xs text-muted-foreground">ID: {game.id}</p>
              </div>
              <Button
                size="sm"
                disabled={addingId === game.id}
                onClick={() => handleAdd(game)}
              >
                {addingId === game.id ? (
                  <Loader2 className="animate-spin h-4 w-4" />
                ) : (
                  <Plus size={18} />
                )}
              </Button>
            </div>
          ))}
          {results.length === 0 && !loading && query && (
            <p className="text-center text-sm text-muted-foreground py-4">
              Nenhum jogo encontrado.
            </p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
