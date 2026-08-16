import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import {
  Copy,
  ExternalLink,
  FileJson,
  Loader2,
  Plus,
  RefreshCw,
  ShoppingCart,
  Ticket,
  Trash2,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { ActionButton, StandardGameCard } from '@/components';
import AddWishlist from '@/dialogs/AddWishlist';
import { useWishlist, useWishlistFilter } from '@/hooks';
import { useConfirm } from '@/providers/ConfirmProvider';
import { Button } from '@/ui/button';
import { Separator } from '@/ui/separator';
import { openExternalLink, toast } from '@/utils';

interface WishlistProps {
  searchTerm?: string;
}

export default function Wishlist({ searchTerm = '' }: WishlistProps) {
  const { t } = useTranslation('wishlist');
  const {
    games,
    isLoading,
    isRefreshing,
    removeGame,
    refreshPrices,
    refreshList,
  } = useWishlist();

  const [showAddModal, setShowAddModal] = useState(false);
  const { confirm } = useConfirm();

  const handleRemoveClick = async (id: string, name: string) => {
    const confirmed = await confirm({
      title: t('remove_from_wishlist_title'),
      description: t('remove_from_wishlist_description', { name }),
      confirmText: t('remove_button'),
      cancelText: t('cancel_button'),
    });

    if (!confirmed) return;

    try {
      await removeGame(id);
      toast.success(t('game_removed_success', { name }));
    } catch {
      toast.error(t('remove_game_error'));
    }
  };

  const handleRefreshClick = async () => {
    try {
      await refreshPrices();
      toast.success(t('prices_updated_success'));
    } catch {
      toast.error(t('update_prices_error'));
    }
  };

  // Função para importar via JSON (Steam ou ITAD)
  useEffect(() => {
    const unlisten = listen('wishlist_updated', () => {
      toast.success(t('covers_updated'));
      refreshList();
    });

    return () => {
      unlisten.then(f => f());
    };
  }, [refreshList]);

  // Usa hook para filtrar jogos
  const filteredGames = useWishlistFilter(games, searchTerm);

  const handleImportJson = async () => {
    let loadingToast: string | number | undefined;

    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });

      if (selected) {
        loadingToast = toast.loading(t('importing_list'));
        const count = await invoke<number>('import_wishlist', {
          filePath: selected,
        });
        toast.dismiss(loadingToast);

        if (count > 0) {
          toast.success(t('games_imported_success', { count }));
          refreshList();
          invoke('fetch_wishlist_covers').catch(console.error);
        } else {
          toast.info(t('no_new_games_found'));
        }
      }
    } catch (error) {
      console.error(error);

      if (loadingToast) {
        toast.dismiss(loadingToast);
      }

      toast.error(t('import_error', { error }));
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Loader2 className="text-primary animate-spin" size={32} />
      </div>
    );
  }

  // Empty state
  if (games.length === 0) {
    return (
      <div className="text-muted-foreground flex flex-1 flex-col items-center justify-center">
        <ShoppingCart className="mb-4 h-16 w-16 opacity-20" />
        <h3 className="text-lg font-medium">{t('empty_wishlist_title')}</h3>
        <p className="mb-4 text-sm">{t('empty_wishlist_description')}</p>
        <div className="flex gap-2">
          <Button onClick={() => setShowAddModal(true)} variant="outline">
            <Plus className="mr-2 h-4 w-4" /> {t('add_manually_button')}
          </Button>
          <Button onClick={handleImportJson} variant="outline">
            <FileJson className="mr-2 h-4 w-4" /> {t('import_json_button')}
          </Button>
        </div>

        <AddWishlist
          isOpen={showAddModal}
          onClose={() => setShowAddModal(false)}
          onSuccess={refreshList}
        />
      </div>
    );
  }

  return (
    <div className="custom-scrollbar flex-1 overflow-y-auto p-8 pb-20">
      {/* HEADER DA PÁGINA */}
      <div className="space-y-6">
        <div className="flex items-center gap-3">
          <div className="rounded-lg bg-purple-500/10 p-2 text-purple-400">
            <ShoppingCart size={24} />
          </div>
          <div>
            <h1 className="text-2xl font-bold">{t('wishlist_title')}</h1>
            <p className="text-muted-foreground text-sm">
              {t('monitoring_label')} {games.length}{' '}
              {games.length === 1 ? t('game_singular') : t('games_plural')}
            </p>
          </div>
        </div>

        {/* BARRA DE FERRAMENTAS (Abaixo do Header) */}
        <div className="flex flex-wrap gap-2">
          <Button
            onClick={handleRefreshClick}
            disabled={isRefreshing || games.length === 0}
            variant="outline"
            className="flex-1 sm:flex-none"
          >
            <RefreshCw
              className={`mr-2 h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`}
            />
            {isRefreshing ? t('searching_offers') : t('update_prices_button')}
          </Button>
          <div className="flex-1 sm:flex-none" /> {/* Espaçador */}
          <Button
            onClick={() => setShowAddModal(true)}
            variant="secondary"
            className="flex-1 sm:flex-none"
          >
            <Plus className="mr-2 h-4 w-4" /> {t('add_game_button')}
          </Button>
          <Button
            onClick={handleImportJson}
            variant="secondary"
            className="flex-1 sm:flex-none"
          >
            <FileJson className="mr-2 h-4 w-4" /> {t('import_file_button')}
          </Button>
        </div>
      </div>

      <Separator className="my-6" />

      {/* GRID DE JOGOS */}
      <div className="grid grid-cols-2 gap-4 sm:gap-6 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
        {filteredGames.length === 0 ? (
          <div className="text-muted-foreground col-span-full py-10 text-center">
            {t('no_games_found_for_search', { searchTerm })}
          </div>
        ) : (
          filteredGames.map(game => {
            // Formata preço (lógica inline sem hooks)
            let priceDisplay = t('waiting_for_price');

            if (game.currentPrice !== null && game.currentPrice !== undefined) {
              const currencySymbol =
                game.currency === 'BRL'
                  ? 'R$'
                  : game.currency === 'USD'
                    ? 'US$'
                    : game.currency || '';
              priceDisplay = `${currencySymbol} ${game.currentPrice.toFixed(2)}`;
            }

            // Constrói URL de loja (lógica inline sem hooks)
            const targetUrl =
              game.storeUrl ||
              (game.itadId
                ? `https://isthereanydeal.com/game/${game.itadId}/`
                : null);
            const storeLabel = game.store
              ? t('open_in_store', { store: game.store })
              : targetUrl
                ? t('view_on_itad')
                : t('no_link_available');

            return (
              <StandardGameCard
                id={game.id.toString()}
                key={game.id}
                title={game.name}
                coverUrl={game.coverUrl}
                subtitle={priceDisplay}
                badge={
                  game.voucher ? (
                    <div className="flex items-center gap-1 font-mono text-xs">
                      <Ticket size={10} />
                      <span>{game.voucher}</span>
                    </div>
                  ) : game.onSale ? (
                    t('deal_badge')
                  ) : undefined
                }
                actions={
                  <>
                    {/* Botão de Copiar Cupom (Só aparece se tiver voucher) */}
                    {game.voucher && (
                      <ActionButton
                        icon={Copy}
                        variant="glass"
                        size={16}
                        onClick={() => {
                          navigator.clipboard.writeText(game.voucher || '');
                          toast.success(
                            t('coupon_copied', { coupon: game.voucher })
                          );
                        }}
                        tooltip={t('copy_coupon_tooltip', {
                          coupon: game.voucher,
                        })}
                      />
                    )}
                    <ActionButton
                      icon={Trash2}
                      variant="destructive"
                      size={16}
                      onClick={() => handleRemoveClick(game.id, game.name)}
                      tooltip={t('remove_tooltip')}
                    />
                    <ActionButton
                      icon={ExternalLink}
                      variant="secondary"
                      size={16}
                      disabled={!targetUrl}
                      onClick={() => {
                        if (targetUrl) openExternalLink(targetUrl);
                      }}
                      tooltip={storeLabel}
                    />
                  </>
                }
              />
            );
          })
        )}
      </div>
      <AddWishlist
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        onSuccess={refreshList}
      />
    </div>
  );
}
