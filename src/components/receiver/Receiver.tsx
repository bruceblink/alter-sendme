import { useEffect, useState } from 'react'
import { TicketInput } from './TicketInput'
import { ReceivingActiveCard } from './ReceivingActiveCard'
import { PulseAnimation } from '../sender/PulseAnimation'
import { TransferSuccessScreen } from '../sender/TransferSuccessScreen'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '../ui/alert-dialog'
import { AppAlertDialog } from '../AppAlertDialog'
import { useReceiver } from '@/hooks/useReceiver.ts'
import { Info } from 'lucide-react'
import { useTranslation } from '@/i18n'

interface ReceiverProps {
  onTransferStateChange: (isReceiving: boolean) => void
}

export function Receiver({ onTransferStateChange }: ReceiverProps) {
  const [showInstructionsDialog, setShowInstructionsDialog] = useState(false)
  const { t } = useTranslation()
  
  const {
    ticket,
    isReceiving,
    isTransporting,
    isCompleted,
    savePath,
    alertDialog,
    transferMetadata,
    transferProgress,
    fileNames,
    handleTicketChange,
    handleBrowseFolder,
    handleReceive,
    closeAlert,
    resetForNewTransfer
  } = useReceiver()

  useEffect(() => {
    onTransferStateChange(isReceiving)
  }, [isReceiving, onTransferStateChange])

  return (
    <div className="p-6 space-y-6 relative h-[28rem] overflow-y-auto flex flex-col text-app-fg">

      {!isReceiving ? (
        <>
          <div className="text-center">
            <div className="flex items-center justify-center gap-2 mb-2">
              <h2 className="text-xl font-semibold text-app-fg">
                {t('common:receiver.title')}
              </h2>
              <button
                onClick={() => setShowInstructionsDialog(true)}
                className="p-1 hover:bg-white/10 rounded-full transition-colors absolute top-6 right-6 text-white/60"
                title={t('common:receiver.howToReceive')}
                aria-label={t('common:receiver.howToReceive')}
              >
                <Info size={24} />
              </button>
            </div>
            <p className="text-sm text-white/60">
              {t('common:receiver.subtitle')}
            </p>
          </div>

          <div className="space-y-4 flex-1 flex flex-col">
            <TicketInput
              ticket={ticket}
              isReceiving={isReceiving}
              savePath={savePath}
              onTicketChange={handleTicketChange}
              onBrowseFolder={handleBrowseFolder}
              onReceive={handleReceive}
            />
          </div>
        </>
      ) : isCompleted && transferMetadata ? (
        <div className="flex-1 flex flex-col">
          <TransferSuccessScreen 
            metadata={transferMetadata}
            onDone={resetForNewTransfer}
          />
        </div>
      ) : (
        <>
          <div className="text-center">
            <PulseAnimation 
              isTransporting={isTransporting}
              isCompleted={isCompleted}
              className="mx-auto my-4 flex items-center justify-center" 
            />
          </div>
          <div className="flex-1 flex flex-col">
            <ReceivingActiveCard
              isReceiving={isReceiving}
              isTransporting={isTransporting}
              isCompleted={isCompleted}
              ticket={ticket}
              transferProgress={transferProgress}
              fileNames={fileNames}
              onReceive={handleReceive}
              onStopReceiving={resetForNewTransfer}
            />
          </div>
        </>
      )}

      <AppAlertDialog
        isOpen={alertDialog.isOpen}
        title={alertDialog.title}
        description={alertDialog.description}
        type={alertDialog.type}
        onClose={closeAlert}
      />

      <AlertDialog open={showInstructionsDialog} onOpenChange={setShowInstructionsDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('common:receiver.howToReceive')}</AlertDialogTitle>
            <AlertDialogDescription>
           
            </AlertDialogDescription>
            <ol className="text-sm space-y-2 list-decimal list-inside mt-2 text-white/80">
              <li>{t('common:receiver.instruction1')}</li>
              <li>{t('common:receiver.instruction2')}</li>
              <li>{t('common:receiver.instruction3')}</li>
              <li>{t('common:receiver.instruction4')}</li>
              <li>{t('common:receiver.instruction5')}</li>
            </ol>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogAction onClick={() => setShowInstructionsDialog(false)}>
              {t('common:ok')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
