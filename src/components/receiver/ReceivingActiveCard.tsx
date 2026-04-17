import { Square } from 'lucide-react'
import type { TransferProgress } from '@/types/sender.ts'
import { TransferProgressBar } from '../sender/TransferProgressBar'
import { useTranslation } from '@/i18n'

interface ReceivingActiveCardProps {
  isReceiving: boolean
  isTransporting: boolean
  isCompleted: boolean
  ticket: string
  transferProgress: TransferProgress | null
  fileNames: string[]
  onReceive: () => Promise<void>
  onStopReceiving: () => Promise<void>
}

export function ReceivingActiveCard({ 
  isTransporting,
  isCompleted,
  transferProgress,
  onStopReceiving 
}: ReceivingActiveCardProps) {
  const { t } = useTranslation()
  
  const getStatusClass = () => {
    if (isCompleted) return 'text-status-done'
    if (isTransporting) return 'text-status-active'
    return 'text-status-idle'
  }

  const getIndicatorClass = () => {
    if (isCompleted) return 'bg-status-done'
    if (isTransporting) return 'bg-status-active'
    return 'bg-status-idle'
  }

  const getStatusText = () => {
    if (isCompleted) return t('common:receiver.downloadCompleted')
    if (isTransporting) return t('common:receiver.downloadingInProgress')
    return t('common:receiver.connectingToSender')
  }


  const statusClass = getStatusClass()
  const indicatorClass = getIndicatorClass()
  const statusText = getStatusText()

  return (
    <div className="space-y-4">
      <div className="p-4 rounded-lg absolute top-0 left-0">
        <div className="flex items-center mb-2">
          <div className={`h-2 w-2 rounded-full mr-2 ${indicatorClass}`}></div>
          <p className={`text-sm font-medium ${statusClass}`}>
            {statusText}
          </p>
        </div>
      </div>
      
      <p className="text-xs text-center text-white/70">
        {t('common:receiver.keepAppOpen')}
      </p>
        
      {isTransporting && transferProgress && (
        <TransferProgressBar progress={transferProgress} />
      )}
       
      <button
        onClick={onStopReceiving}
        className="absolute top-0 right-6 w-10 h-10 rounded-full font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 flex items-center justify-center p-0 btn-app-destructive"
        aria-label="Stop receiving"
      >
        <Square className="w-4 h-4" fill="currentColor" />
      </button>
    </div>
  )
}