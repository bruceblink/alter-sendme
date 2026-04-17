import { CheckCircle, XCircle } from 'lucide-react'
import type { SuccessScreenProps } from '@/types/sender.ts'
import { useTranslation } from '@/i18n'
import { formatFileSize, formatDuration, formatSpeed } from '@/lib/utils'

function calculateAverageSpeed(fileSizeBytes: number, durationMs: number): number {
  if (durationMs === 0) return 0
  const durationSeconds = durationMs / 1000
  return fileSizeBytes / durationSeconds
}

export function TransferSuccessScreen({ metadata, onDone }: SuccessScreenProps) {
  const wasStopped = metadata.wasStopped || false
  const isDirectory = metadata.pathType === 'directory'
  const { t } = useTranslation()
  
  const handleDone = () => {
    onDone()
  }
  
  return (
    <div className="flex flex-col items-center justify-center space-y-6 ">
      <div className="flex items-center justify-center">
        {wasStopped ? (
          <XCircle size={44} className="text-red-500" />
        ) : (
          <CheckCircle size={44} className="text-status-active" />
        )}
      </div>
      
      <div className="text-center">
        <h2 className="mb-2 text-2xl font-semibold text-app-fg">
          {wasStopped ? t('common:transfer.stopped') : t('common:transfer.complete')}
        </h2>
        <p className="text-sm text-white/60">
          {wasStopped ? t('common:transfer.wasStopped') : t('common:transfer.successMessage')}
        </p>
      </div>
      
      <div className="w-full max-w-full p-4 rounded-lg bg-white/5">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="mr-2 text-sm font-medium text-white/70">
              {isDirectory ? t('common:transfer.folder') : t('common:transfer.file')}:
            </span>
            <span className="max-w-full text-sm truncate text-app-fg" title={metadata.fileName}>
              {metadata.fileName}
            </span>
          </div>
          
          {metadata.downloadPath && (
            <div className="flex items-center justify-between">
              <span className="mr-2 text-sm font-medium text-white/70">
                {t('common:transfer.downloadPath')}:
              </span>
              <span className="max-w-full text-sm truncate text-app-fg" title={metadata.downloadPath}>
                {metadata.downloadPath}
              </span>
            </div>
          )}
          
          <div className="flex items-center justify-between">
            <span className="mr-2 text-sm font-medium text-white/70">
              {isDirectory ? t('common:transfer.folderSize') : t('common:transfer.fileSize')}:
            </span>
            <span className="text-sm text-app-fg">
              {wasStopped ? 'NA' : formatFileSize(metadata.fileSize)}
            </span>
          </div>
          
          <div className="flex items-center justify-between">
            <span className="mr-2 text-sm font-medium text-white/70">
              {t('common:transfer.duration')}:
            </span>
            <span className="text-sm text-app-fg">
              {wasStopped ? '0ms' : formatDuration(metadata.duration)}
            </span>
          </div>
          
          <div className="flex items-center justify-between">
            <span className="mr-2 text-sm font-medium text-white/70">
              {t('common:transfer.avgSpeed')}:
            </span>
            <span className="text-sm text-app-fg">
              {wasStopped ? 'NA' : formatSpeed(calculateAverageSpeed(metadata.fileSize, metadata.duration))}
            </span>
          </div>
        </div>
      </div>
      
      <button
        onClick={handleDone}
        className="w-full max-w-sm px-6 py-3 font-medium transition-colors rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 btn-app-primary"
      >
        {t('common:transfer.done')}
      </button>
    </div>
  )
}
