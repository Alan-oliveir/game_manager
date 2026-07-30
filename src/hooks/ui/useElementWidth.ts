import { useEffect, useRef, useState } from 'react';

/**
 * Mede a largura de um elemento via ResizeObserver, atualizando sempre que
 * o contêiner muda de tamanho (redimensionar janela, colapsar sidebar, etc).
 * Usado para calcular o número de colunas e o tamanho das células da grade
 * virtualizada de jogos.
 */
export function useElementWidth<T extends HTMLElement = HTMLDivElement>() {
  const ref = useRef<T>(null);
  const [width, setWidth] = useState(0);

  useEffect(() => {
    const el = ref.current;

    if (!el) return;

    const updateWidth = () => setWidth(el.getBoundingClientRect().width);
    updateWidth();

    const observer = new ResizeObserver(updateWidth);
    observer.observe(el);

    return () => observer.disconnect();
  }, []);

  return { ref, width };
}
