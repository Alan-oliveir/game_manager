/**
 * Converte minutos (do banco de dados) em ‘string’ formatada.
 * - Menos de 1h: Retorna em minutos (ex: "45m")
 * - Mais de 1h: Retorna em horas (ex: "1h30", "2h")
 */
export function formatTime(minutes: number | undefined) {
  if (!minutes || minutes === 0) return '0h';

  const h = Math.floor(minutes / 60);
  const m = Math.floor(minutes % 60);

  if (h === 0) {
    return `${m}m`;
  }

  if (m === 0) {
    return `${h}h`;
  }

  // Adiciona o zero à esquerda para os minutos (ex: 2h05)
  const paddedM = m.toString().padStart(2, '0');

  return `${h}h${paddedM}`;
}

/**
 * Converte horas em número real (usado pelo HowLongToBeat) para o formato xxhyy, descartando os segundos.
 * Ex: 1.5 -> "1h30", 2.25 -> "2h15", 3.0 -> "3h"
 */
export function formatHours(hoursFloat: number | undefined): string {
  if (!hoursFloat || hoursFloat === 0) return '0h';

  const h = Math.floor(hoursFloat);

  // Math.round é usado ao invés de floor para prevenir erros de precisão de ponto flutuante do JavaScript
  // (ex: 0.1 * 60 resulta em 5.999999999999996)
  const m = Math.round((hoursFloat - h) * 60);

  // Tratamento caso o arredondamento alcance 60 minutos
  if (m === 60) {
    return `${h + 1}h`;
  }

  if (h === 0) {
    return `${m}m`;
  }

  if (m === 0) {
    return `${h}h`;
  }

  const paddedM = m.toString().padStart(2, '0');

  return `${h}h${paddedM}`;
}
