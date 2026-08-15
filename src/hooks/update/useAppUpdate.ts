import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

import { getAppVersionInfo } from '@/services/updaterService.ts';
import { toast } from '@/utils/toast';

export type UpdateType = 'none' | 'patch' | 'minor' | 'major';

/**
 * Parse a version string into its components.
 *
 * @param version
 * @returns An object with major, minor, and patch numbers.
 */
function parseVersion(version: string): {
  major: number;
  minor: number;
  patch: number;
} {
  const parts = version.split('.');

  return {
    major: Number.parseInt(parts[0] || '0', 10),
    minor: Number.parseInt(parts[1] || '0', 10),
    patch: Number.parseInt(parts[2] || '0', 10),
  };
}

/**
 * Custom hook to check for app updates and manage update state.
 *
 * @returns An object with:
 *   - updateType: Type of update ('none', 'patch', 'minor', 'major')
 *   - isMajorOpen: Boolean indicating if major update modal is open
 *   - closeMajorModal: Function to close the major update modal
 */
export function useAppUpdate() {
  const [updateType, setUpdateType] = useState<UpdateType>('none');
  const [isMajorOpen, setIsMajorOpen] = useState(false);
  const [updaterEnabled, setUpdaterEnabled] = useState(false);

  useEffect(() => {
    invoke<boolean>('is_updater_enabled')
      .then(setUpdaterEnabled)
      .catch(() => setUpdaterEnabled(false));
  }, []);

  useEffect(() => {
    if (!updaterEnabled) return;

    async function checkUpdate() {
      try {
        const { currentVersion, previousVersion } = await getAppVersionInfo();

        if (!previousVersion) return;

        const current = parseVersion(currentVersion);
        const previous = parseVersion(previousVersion);

        if (current.major > previous.major) {
          setUpdateType('major');
          setIsMajorOpen(true);

          return;
        }

        if (current.minor > previous.minor) {
          setUpdateType('minor');
          toast.success(`Playlite atualizado para v${currentVersion}`);

          return;
        }

        if (current.patch > previous.patch) {
          setUpdateType('patch');
          toast(`Correções aplicadas (v${currentVersion})`);
        }
      } catch (error) {
        console.error('Erro ao verificar atualização:', error);
      }
    }

    checkUpdate();
  }, []);

  return {
    updateType,
    isMajorOpen,
    closeMajorModal: () => setIsMajorOpen(false),
  };
}
