import { useCallback, useEffect, useRef, useState } from 'react'
import { check } from '@tauri-apps/plugin-updater'
import { useTranslation } from '@/i18n'

export function useAutoUpdater() {
  const { t } = useTranslation()
  const [isChecking, setIsChecking] = useState(false)
  const [statusMessage, setStatusMessage] = useState('')
  const isCheckingRef = useRef(false)
  const hasAutoCheckedRef = useRef(false)

  const checkForUpdates = useCallback(
    async (manual = false) => {
      if (!IS_TAURI || isCheckingRef.current) {
        return
      }

      isCheckingRef.current = true
      setIsChecking(true)

      if (manual) {
        setStatusMessage(t('common:update.checking'))
      }

      try {
        const update = await check()

        if (!update) {
          if (manual) {
            setStatusMessage(t('common:update.upToDate'))
          }
          return
        }

        setStatusMessage(t('common:update.found', { version: update.version }))
        await update.downloadAndInstall()
        setStatusMessage(t('common:update.installed'))
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        if (manual) {
          setStatusMessage(`${t('common:update.failed')}: ${message}`)
        } else {
          console.warn('Auto update check skipped due to error:', message)
        }
      } finally {
        isCheckingRef.current = false
        setIsChecking(false)
      }
    },
    [t],
  )

  useEffect(() => {
    if (hasAutoCheckedRef.current) {
      return
    }

    hasAutoCheckedRef.current = true
    void checkForUpdates(false)
  }, [checkForUpdates])

  return {
    isChecking,
    statusMessage,
    checkForUpdates,
  }
}
