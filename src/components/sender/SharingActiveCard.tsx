import { useState, useEffect, useRef } from 'react'
import { Copy, CheckCircle, Square } from 'lucide-react'
import type { SharingControlsProps, TicketDisplayProps } from '@/types/sender.ts'
import { TransferProgressBar } from './TransferProgressBar'
import { useTranslation } from '@/i18n'
import { basenameFromPath } from '@/lib/path'

export function SharingActiveCard({ 
  selectedPath, 
  pathType,
  ticket, 
  copySuccess,
  transferProgress,
  isTransporting,
  isCompleted,
  onCopyTicket, 
  onStopSharing 
}: SharingControlsProps) {
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
    if (isCompleted) return t('common:sender.transferCompleted')
    if (isTransporting) return t('common:sender.sharingInProgress')
    return t('common:sender.listeningForConnection')
  }

  const statusClass = getStatusClass()
  const indicatorClass = getIndicatorClass()
  const statusText = getStatusText()

  const [cumulativeBytesTransferred, setCumulativeBytesTransferred] = useState(0)
  const [transferStartTime, setTransferStartTime] = useState<number | null>(null)
  const previousBytesRef = useRef<number>(0)
  const maxBytesRef = useRef<number>(0)
  const isFolderTransfer = pathType === 'directory' && isTransporting

  useEffect(() => {
    if (isTransporting && pathType === 'directory') {
      setCumulativeBytesTransferred(0)
      setTransferStartTime(Date.now())
      previousBytesRef.current = 0
      maxBytesRef.current = 0
    }
  }, [isTransporting, pathType])

  useEffect(() => {
    if (isFolderTransfer && transferProgress) {
      const currentBytes = transferProgress.bytesTransferred
      const previousBytes = previousBytesRef.current
      const maxBytes = maxBytesRef.current

      if (currentBytes > maxBytes) {
        maxBytesRef.current = currentBytes
      }

      if (previousBytes > 0 && currentBytes < previousBytes * 0.5 && maxBytes > 0) {
        setCumulativeBytesTransferred(prev => prev + maxBytes)
        maxBytesRef.current = currentBytes
        previousBytesRef.current = currentBytes
      } else if (currentBytes === 0 && previousBytes > 0 && maxBytes > 0) {
        setCumulativeBytesTransferred(prev => prev + maxBytes)
        maxBytesRef.current = 0
        previousBytesRef.current = 0
      } else if (currentBytes > previousBytes) {
        previousBytesRef.current = currentBytes
      } else if (currentBytes < previousBytes && currentBytes >= previousBytes * 0.5) {
        previousBytesRef.current = currentBytes
      }
    }
  }, [isFolderTransfer, transferProgress?.bytesTransferred])

  const totalTransferredBytes = isFolderTransfer && transferProgress
    ? cumulativeBytesTransferred + transferProgress.bytesTransferred
    : transferProgress?.bytesTransferred ?? 0

  const [calculatedSpeed, setCalculatedSpeed] = useState(0)
  
  useEffect(() => {
    if (isFolderTransfer && transferProgress && transferStartTime) {
      const updateSpeed = () => {
        const elapsed = (Date.now() - transferStartTime) / 1000.0
        const speed = elapsed > 0 ? totalTransferredBytes / elapsed : 0
        setCalculatedSpeed(speed)
      }
      
      updateSpeed()
      const interval = setInterval(updateSpeed, 500)
      return () => clearInterval(interval)
    } else if (transferProgress) {
      setCalculatedSpeed(transferProgress.speedBps)
    } else {
      setCalculatedSpeed(0)
    }
  }, [isFolderTransfer, transferProgress, transferStartTime, totalTransferredBytes])

  // Calculate percentage and create progress object for folders
  const folderProgress = isFolderTransfer && transferProgress
    ? {
        bytesTransferred: totalTransferredBytes,
        totalBytes: transferProgress.totalBytes,
        speedBps: calculatedSpeed,
        percentage: transferProgress.totalBytes > 0 
          ? (totalTransferredBytes / transferProgress.totalBytes) * 100 
          : 0
      }
    : null

  return (
    <div className="space-y-4">
      <div className="absolute top-0 left-0 p-4 rounded-lg">
        <p className="text-xs mb-4 max-w-[30rem] truncate text-white/70">
          <strong className="mr-1">{t('common:sender.fileLabel')}</strong> {selectedPath ? basenameFromPath(selectedPath) : ''}
        </p>

        <div className="flex items-center mb-2">
          <div className={`h-2 w-2 rounded-full mr-2 ${indicatorClass}`}></div>
          <p className={`text-sm font-medium ${statusClass}`}>
            {statusText}
          </p>
        </div>
      </div>
      
      <p className="text-xs text-center text-white/70">
        {t('common:sender.keepAppOpen')}
      </p>
        
      {!isTransporting && ticket && (
        <TicketDisplay 
          ticket={ticket} 
          copySuccess={copySuccess} 
          onCopyTicket={onCopyTicket} 
        />
      )}
      
      {isTransporting && transferProgress && (
        folderProgress ? (
          <TransferProgressBar progress={folderProgress} />
        ) : (
          <TransferProgressBar progress={transferProgress} />
        )
      )}
       
    
      <button
        onClick={onStopSharing}
        className="absolute top-0 right-6 w-10 h-10 rounded-full font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 flex items-center justify-center p-0 btn-app-destructive"
        aria-label="Stop sharing"
        title={t('common:sender.stopSharing')}
      >
        <Square className="w-4 h-4" fill="currentColor" />
      </button>
    </div>
  )
}

export function TicketDisplay({ ticket, copySuccess, onCopyTicket }: TicketDisplayProps) {
  const { t } = useTranslation()
  
  return (
    <div className="space-y-3">
      <label className="block text-sm font-medium text-app-fg">
        {t('common:sender.shareThisTicket')}
      </label>
      <div className="flex gap-2">
        <input
          type="text"
          value={ticket}
          readOnly
          title={t('common:sender.shareThisTicket')}
          className="flex-1 p-3 font-mono text-xs rounded-md bg-white/10 border border-white/20 text-app-fg"
        />
        <button
          onClick={onCopyTicket}
          className={`px-3 py-2 transition-colors rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 ${copySuccess ? 'btn-app-primary' : 'btn-glass'}`}
          title={t('common:sender.copyToClipboard')}
        >
          {copySuccess ? <CheckCircle className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
        </button>
      </div>
      <p className="text-xs text-white/60">
        {t('common:sender.sendThisTicket')}
      </p>
    </div>
  )
}
