import { useCallback, useState } from 'react';

export function useLocalStoragePlatformPath(key: string) {
  const readStoredValue = () => {
    if (typeof localStorage === 'undefined') {
      return '';
    }

    return localStorage.getItem(key) || '';
  };

  const [value, setValue] = useState(readStoredValue);

  const update = useCallback(
    (newValue: string) => {
      setValue(newValue);

      if (typeof localStorage === 'undefined') {
        return;
      }

      if (newValue) {
        localStorage.setItem(key, newValue);
      } else {
        localStorage.removeItem(key);
      }
    },
    [key]
  );

  return [value, update] as const;
}
