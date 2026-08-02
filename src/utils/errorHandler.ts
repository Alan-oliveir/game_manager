import i18n from '@/i18n';
import { AppError, getErrorMessage, isAppError } from '@/types/errors';
import { toast } from '@/utils/toast';

/**
 * Opções para o handler de erros
 */
interface ErrorHandlerOptions {
  /** Mensagem padrão se não houver uma específica */
  defaultMessage?: string;
  /** Se deve mostrar toast automaticamente */
  showToast?: boolean;
  /** Tipo de toast (error, warning, info) */
  toastType?: 'error' | 'warning' | 'info';
  /** Callback personalizado para logging */
  onError?: (error: unknown) => void;
}

/**
 * Handler genérico para erros do backend
 * Trata AppError estruturados e fornece feedback apropriado
 */
export function handleBackendError(
  error: unknown,
  options: ErrorHandlerOptions = {}
): string {
  const {
    defaultMessage = i18n.t('errors:generic_desc'),
    showToast = true,
    toastType = 'error',
    onError,
  } = options;

  // Log do erro se callback fornecido
  if (onError) {
    onError(error);
  } else {
    console.error('Backend error:', error);
  }

  let message: string;

  // Trata AppError estruturado
  if (isAppError(error)) {
    message = formatAppError(error);
  } else {
    message = getErrorMessage(error) || defaultMessage;
  }

  // Mostra toast se solicitado
  if (showToast) {
    switch (toastType) {
      case 'error':
        toast.error(message);
        break;
      case 'warning':
        toast.warning(message);
        break;
      case 'info':
        toast.info(message);
        break;
    }
  }

  return message;
}

/**
 * Formata um AppError para exibição ao usuário
 */
function formatAppError(error: AppError): string {
  switch (error.type) {
    case 'ValidationError':
      return i18n.t('errors:error_msg_validation_error', {
        message: error.message,
      });

    case 'DatabaseError':
      return i18n.t('errors:error_msg_database_save_error', {
        message: error.message,
      });

    case 'NetworkError':
      return i18n.t('errors:error_msg_connection_error', {
        message: error.message,
      });

    case 'NotFound':
      return i18n.t('errors:error_msg_not_found', {
        message: error.message,
      });

    case 'IoError':
      return i18n.t('errors:error_msg_error_accessing_file', {
        message: error.message,
      });

    case 'SerializationError':
      return i18n.t('errors:error_msg_error_processing_data', {
        message: error.message,
      });

    case 'AlreadyExists':
      return i18n.t('errors:error_msg_already_exists', {
        message: error.message,
      });

    case 'MutexError':
      return i18n.t('errors:error_msg_mutex_busy');

    default:
      return error.message;
  }
}

/**
 * Helper específico para erros de API key ausente
 */
export function handleMissingApiKey(apiName: string): void {
  toast.warning(i18n.t('errors:error_msg_missing_api_key', { apiName }), {
    duration: 5000,
  });
}

/**
 * Helper para erros de rede com retry
 */
export function handleNetworkError(
  error: unknown,
  retryCallback?: () => void
): void {
  const message = isAppError(error)
    ? error.message
    : i18n.t('errors:error_msg_generic_network');

  if (retryCallback) {
    toast.error(message, {
      action: {
        label: i18n.t('errors:action_retry'),
        onClick: retryCallback,
      },
    });
  } else {
    toast.error(message);
  }
}

/**
 * Helper para validações
 */
export function handleValidationError(error: unknown): string {
  if (isAppError(error) && error.type === 'ValidationError') {
    toast.warning(error.message);

    return error.message;
  }

  const message = getErrorMessage(error);
  toast.warning(message);

  return message;
}
