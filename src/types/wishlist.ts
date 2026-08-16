export interface WishlistGame {
  id: string;
  name: string;
  coverUrl: string | null;
  storeUrl: string | null;
  store: string | null;
  currentPrice: number | null;
  normalPrice: number | null;
  lowestPrice: number | null;
  currency: string | null;
  onSale: boolean;
  voucher?: string | null;
  addedAt: string | null;
  itadId: string | null;
}
