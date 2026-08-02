import { getCurrentWindow } from '@tauri-apps/api/window';
import { toast as sonnerToast } from 'sonner';

type ToastOptions = Parameters<typeof sonnerToast.success>[1];
type ToastId = ReturnType<typeof sonnerToast.success>;

type ToastVariant =
  | 'default'
  | 'info'
  | 'success'
  | 'warning'
  | 'error'
  | 'loading';

type HybridToast = ((message: string, options?: ToastOptions) => ToastId) & {
  info: (message: string, options?: ToastOptions) => ToastId;
  success: (message: string, options?: ToastOptions) => ToastId;
  warning: (message: string, options?: ToastOptions) => ToastId;
  error: (message: string, options?: ToastOptions) => ToastId;
  loading: (message: string, options?: ToastOptions) => ToastId;
  dismiss: (toastId?: ToastId) => void;
};

type WindowState = {
  isVisible: boolean;
  isFocused: boolean;
};

type SonnerWithWarning = typeof sonnerToast & {
  warning: typeof sonnerToast.success;
};

const windowState: WindowState = {
  isVisible: true,
  isFocused: true,
};

const sonner = sonnerToast as SonnerWithWarning;

let initialized = false;
let nativePermissionRequested = false;

function createToastId(): ToastId {
  return (
    globalThis.crypto?.randomUUID?.() ??
    `native-toast-${Date.now()}-${Math.random()}`
  );
}

function getDescription(options: ToastOptions): string | undefined {
  return typeof options?.description === 'string'
    ? options.description
    : undefined;
}

function shouldUseNativeNotification(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }

  return (
    !windowState.isVisible ||
    !windowState.isFocused ||
    document.hidden ||
    !document.hasFocus()
  );
}

async function requestNativeNotificationPermission(): Promise<boolean> {
  if (typeof window === 'undefined' || typeof Notification === 'undefined') {
    return Promise.resolve(false);
  }

  if (Notification.permission === 'granted') {
    return Promise.resolve(true);
  }

  if (Notification.permission === 'denied') {
    return Promise.resolve(false);
  }

  try {
    const permission = await Notification.requestPermission();

    return permission === 'granted';
  } catch {
    return false;
  }
}

async function syncWindowState() {
  if (typeof window === 'undefined') {
    return;
  }

  windowState.isVisible = !document.hidden;
  windowState.isFocused = document.hasFocus();

  try {
    const appWindow = getCurrentWindow();
    windowState.isVisible = await appWindow.isVisible();
    windowState.isFocused = await appWindow.isFocused();
  } catch {
    // Fallback para o estado do DOM quando a API da janela não estiver disponível.
  }
}

function showSonnerToast(
  variant: ToastVariant,
  message: string,
  options?: ToastOptions
): ToastId {
  switch (variant) {
    case 'info':
      return sonner.info(message, options);
    case 'success':
      return sonner.success(message, options);
    case 'warning':
      return sonner.warning(message, options);
    case 'error':
      return sonner.error(message, options);
    case 'loading':
      return sonner.loading(message, options);
    default:
      return sonner(message, options);
  }
}

function showNativeNotification(
  variant: Exclude<ToastVariant, 'loading'>,
  message: string,
  options?: ToastOptions
): ToastId {
  const id = createToastId();

  void (async () => {
    if (typeof window === 'undefined' || typeof Notification === 'undefined') {
      showSonnerToast(variant, message, options);

      return;
    }

    if (!nativePermissionRequested) {
      nativePermissionRequested = true;
      const granted = await requestNativeNotificationPermission();

      if (!granted) {
        showSonnerToast(variant, message, options);

        return;
      }
    }

    const notification = new Notification('Playlite', {
      body:
        [message, getDescription(options)].filter(Boolean).join('\n\n') ||
        message,
      icon: '/app-icon.png',
      silent: false,
    });

    notification.onclick = async () => {
      try {
        const appWindow = getCurrentWindow();
        await appWindow.show();
        await appWindow.setFocus();
      } catch {
        // Ignora falhas ao trazer a janela para frente.
      }
    };
  })();

  return id;
}

function showToast(
  variant: ToastVariant,
  message: string,
  options?: ToastOptions
): ToastId {
  const isUpdatingExisting =
    !!options && 'id' in options && options.id !== undefined;

  if (
    !isUpdatingExisting &&
    shouldUseNativeNotification() &&
    variant !== 'loading'
  ) {
    return showNativeNotification(variant, message, options);
  }

  return showSonnerToast(variant, message, options);
}

const toastImpl: HybridToast = ((message: string, options?: ToastOptions) =>
  showToast('default', message, options)) as HybridToast;

toastImpl.info = (message: string, options?: ToastOptions) =>
  showToast('info', message, options);
toastImpl.success = (message: string, options?: ToastOptions) =>
  showToast('success', message, options);
toastImpl.warning = (message: string, options?: ToastOptions) =>
  showToast('warning', message, options);
toastImpl.error = (message: string, options?: ToastOptions) =>
  showToast('error', message, options);
toastImpl.loading = (message: string, options?: ToastOptions) =>
  showToast('loading', message, options);
toastImpl.dismiss = (toastId?: ToastId) => {
  sonner.dismiss(toastId);
};

export const toast = toastImpl;

export async function initializeToastRouting() {
  if (initialized) {
    return;
  }

  initialized = true;

  if (typeof window === 'undefined') {
    return;
  }

  const updateFromDom = () => {
    windowState.isVisible = !document.hidden;
    windowState.isFocused = document.hasFocus();
  };

  updateFromDom();

  document.addEventListener('visibilitychange', updateFromDom);
  window.addEventListener('focus', updateFromDom);
  window.addEventListener('blur', updateFromDom);

  await syncWindowState();
}
