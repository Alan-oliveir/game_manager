# Gestão de Mensagens de Erro

## Decisão Arquitetural

As mensagens de erro do aplicativo são centralizadas em `src/constants/errorMessages.ts` ao invés de estarem espalhadas pelos services.

## Estrutura

```
src/
  constants/
    errorMessages.ts       # Constantes de mensagens + helpers de parsing
  services/
    settingsService.ts     # Usa ERROR_MESSAGES e parseBackupError()
    librariesService.ts    # Apenas lança erros do backend
  hooks/
    useSettings.ts         # Usa ERROR_MESSAGES para comparação
```

## Benefícios

### 1. **Centralização**
- Todas as mensagens em um único local
- Fácil de encontrar e atualizar
- Evita duplicação de mensagens

### 2. **Consistência**
- Mesma mensagem para o mesmo tipo de erro
- Formato padronizado
- Experiência uniforme para o usuário

### 3. **Manutenibilidade**
- Mudanças de texto não requerem editar múltiplos arquivos
- Facilita revisão de UX Writing
- Reduz chance de inconsistências

### 4. **Internacionalização (i18n)**
- Preparado para suportar múltiplos idiomas no futuro
- Basta criar arquivo `errorMessages.en.ts`, `errorMessages.pt.ts`, etc.
- Importação condicional baseada no locale

### 5. **Testabilidade**
- Constantes podem ser facilmente mockadas
- Helper `parseBackupError()` pode ser testado isoladamente
- Separação clara entre lógica de negócio e apresentação

## Padrão de Uso

### No Service

```typescript
import { ERROR_MESSAGES, parseBackupError } from "../constants/errorMessages";

try {
  // ... lógica
} catch (error: any) {
  // Verifica se foi cancelamento (não mostra erro ao usuário)
  if (error.message === ERROR_MESSAGES.CANCELLED) {
    throw new Error(ERROR_MESSAGES.CANCELLED);
  }

  // Parse do erro para mensagem amigável
  const friendlyError = parseBackupError(error);
  
  // Fallback para mensagem genérica
  if (friendlyError === String(error)) {
    throw new Error(ERROR_MESSAGES.BACKUP_IMPORT_FAILED);
  }
  
  throw new Error(friendlyError);
}
```

### No Hook

```typescript
import { ERROR_MESSAGES } from "../constants/errorMessages";

catch (error: any) {
  // Não mostra erro se o usuário cancelou
  if (error.message !== ERROR_MESSAGES.CANCELLED) {
    setStatus({ type: "error", message: error.message });
  } else {
    setStatus({ type: null, message: "" });
  }
}
```

## Categorias de Mensagens

1. **Permissões** - Erros de diálogo/sistema
2. **Backup** - Erros de importação/exportação
3. **Sistema** - Erros de mutex, I/O, etc.
4. **Validação** - Erros de entrada do usuário
5. **Operações canceladas** - Não devem ser mostradas ao usuário

## Helper Functions

### `parseBackupError(error: unknown): string`
Converte erros técnicos do backend em mensagens amigáveis para o usuário.

**Exemplo:**
```typescript
// Input: "dialog.save not allowed in ACL"
// Output: "Permissão necessária para salvar arquivos."
```

### `matchesErrorPattern(error: unknown, patterns: string[]): boolean`
Verifica se um erro contém algum dos padrões especificados.

**Exemplo:**
```typescript
matchesErrorPattern(error, ["dialog.save", "dialog:allow-save"])
// true se o erro contém qualquer um dos padrões
```

## Quando NÃO usar errorMessages.ts

- **Erros que vêm diretamente do backend**: Se o backend Rust já retorna uma mensagem amigável, não precisa parsear
- **Erros de validação de formulário**: Use validação inline no componente
- **Logs técnicos**: Use `console.error()` com a mensagem original

## Evolução Futura

### i18n Support
```typescript
// errorMessages.ts
export const ERROR_MESSAGES = {
  [locale]: {
    DIALOG_SAVE_PERMISSION: locale === 'pt' 
      ? "Permissão necessária para salvar arquivos."
      : "Permission required to save files."
  }
}
```

### Tipos Mais Específicos
```typescript
type ErrorCategory = 'permission' | 'backup' | 'system' | 'validation';

interface ErrorMessage {
  code: string;
  category: ErrorCategory;
  message: string;
  action?: string; // Sugestão de ação para o usuário
}
```

### Error Boundaries
Integrar com React Error Boundaries para captura global.

## Referências

- [Padrão de Error Handling - Architecture.md](./architecture.md)
- [Service Layer - Architecture.md](./architecture.md#backend-taurirust)

