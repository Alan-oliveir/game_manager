import { ChevronDown, ChevronRight } from 'lucide-react';
import { useCallback, useMemo } from 'react';
import { type CellComponentProps, Grid } from 'react-window';

import { LibraryGameCard } from '@/components';
import { type GridRow, useElementWidth, useGroupedLibrary } from '@/hooks';
import { Game, Platform, PlatformDisplayNames } from '@/types';
import { Badge } from '@/ui/badge';

const GAP = 24;
const CARD_TEXT_AREA_HEIGHT = 76;
const HEADER_ROW_HEIGHT = 48;

let cachedScrollbarWidth: number | null = null;

function getScrollbarWidth(): number {
  if (cachedScrollbarWidth !== null) return cachedScrollbarWidth;

  if (typeof document === 'undefined') return 0;

  const outer = document.createElement('div');
  outer.className = 'custom-scrollbar';
  Object.assign(outer.style, {
    visibility: 'hidden',
    position: 'absolute',
    top: '-9999px',
    width: '100px',
    height: '100px',
    overflowY: 'scroll',
  });

  const inner = document.createElement('div');
  inner.style.height = '200px';
  outer.appendChild(inner);
  document.body.appendChild(outer);

  cachedScrollbarWidth = outer.offsetWidth - outer.clientWidth;
  document.body.removeChild(outer);

  return cachedScrollbarWidth;
}

function getColumnCount(containerWidth: number): number {
  if (containerWidth >= 1280) return 5;

  if (containerWidth >= 1024) return 4;

  if (containerWidth >= 768) return 3;

  return 2;
}

interface LibraryGameGridProps {
  games: Game[];
  groupByPlatform?: boolean;
  onGameClick: (game: Game) => void;
  onToggleFavorite: (id: string) => void;
  onAddToPlaylist: (id: string) => void;
  onEditGame: (game: Game) => void;
  onDeleteGame: (id: string) => void;
  isInPlaylist: (id: string) => boolean;
}

interface CellProps {
  rows: GridRow[];
  gridWidth: number;
  onTogglePlatform: (platform: Platform) => void; // era string
  onGameClick: (game: Game) => void;
  onToggleFavorite: (id: string) => void;
  onAddToPlaylist: (id: string) => void;
  onEditGame: (game: Game) => void;
  onDeleteGame: (id: string) => void;
  isInPlaylist: (id: string) => boolean;
}

function GridCell({
  columnIndex,
  rowIndex,
  style,
  rows,
  gridWidth,
  onTogglePlatform,
  onGameClick,
  onToggleFavorite,
  onAddToPlaylist,
  onEditGame,
  onDeleteGame,
  isInPlaylist,
}: CellComponentProps<CellProps>) {
  const row = rows[rowIndex];

  if (!row) return <div style={style} />;

  if (row.type === 'header') {
    if (columnIndex !== 0) return null;

    return (
      <div
        style={{ ...style, width: gridWidth, left: 0 }}
        className="flex items-center"
      >
        <button
          onClick={() => onTogglePlatform(row.platform)}
          className="hover:bg-accent flex w-full items-center gap-2 rounded-md px-2 py-2 text-left font-semibold"
        >
          {row.collapsed ? (
            <ChevronRight size={16} />
          ) : (
            <ChevronDown size={16} />
          )}
          {PlatformDisplayNames[row.platform] ?? row.platform}
          <Badge variant="secondary">{row.count}</Badge>
        </button>
      </div>
    );
  }

  const game = row.games[columnIndex];

  if (!game) return <div style={style} />;

  return (
    <div style={{ ...style, padding: GAP / 2 }}>
      <LibraryGameCard
        game={game}
        onGameClick={onGameClick}
        onToggleFavorite={onToggleFavorite}
        onAddToPlaylist={onAddToPlaylist}
        onEditGame={onEditGame}
        onDeleteGame={onDeleteGame}
        isInPlaylist={isInPlaylist}
      />
    </div>
  );
}

export function LibraryGameGrid({
  games,
  groupByPlatform = false,
  onGameClick,
  onToggleFavorite,
  onAddToPlaylist,
  onEditGame,
  onDeleteGame,
  isInPlaylist,
}: Readonly<LibraryGameGridProps>) {
  const { ref, width } = useElementWidth<HTMLDivElement>();

  const { columnCount, columnWidth, cardRowHeight } = useMemo(() => {
    const count = getColumnCount(width);
    const scrollbarWidth = getScrollbarWidth();
    const availableWidth = Math.max(width - scrollbarWidth, 0);
    const colWidth = availableWidth > 0 ? availableWidth / count : 0;
    const cardWidth = Math.max(colWidth - GAP, 0);
    const cardImageHeight = cardWidth * (4 / 3);

    return {
      columnCount: count,
      columnWidth: colWidth,
      cardRowHeight: cardImageHeight + CARD_TEXT_AREA_HEIGHT + GAP,
    };
  }, [width]);

  const { rows, togglePlatform } = useGroupedLibrary(
    games,
    groupByPlatform,
    columnCount
  );

  const rowHeight = useCallback(
    (index: number) =>
      rows[index]?.type === 'header' ? HEADER_ROW_HEIGHT : cardRowHeight,
    [rows, cardRowHeight]
  );

  const cellProps: CellProps = {
    rows,
    gridWidth: columnWidth * columnCount,
    onTogglePlatform: togglePlatform,
    onGameClick,
    onToggleFavorite,
    onAddToPlaylist,
    onEditGame,
    onDeleteGame,
    isInPlaylist,
  };

  return (
    <div ref={ref} className="h-full w-full">
      {width > 0 && (
        <Grid
          className="custom-scrollbar overflow-x-hidden"
          cellComponent={GridCell}
          cellProps={cellProps}
          columnCount={columnCount}
          columnWidth={columnWidth}
          rowCount={rows.length}
          rowHeight={rowHeight}
        />
      )}
    </div>
  );
}
