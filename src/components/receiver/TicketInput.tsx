import { Download } from 'lucide-react'
import type { TicketInputProps } from '@/types/sender.ts'
import { useTranslation } from '@/i18n'

export function TicketInput({ 
  ticket, 
  isReceiving, 
  savePath,
  onTicketChange, 
  onBrowseFolder,
  onReceive 
}: TicketInputProps) {
  const { t } = useTranslation()
  
  return (
    <div className="space-y-4">
      <div>
        <label className="block mb-2 text-sm font-medium text-app-fg">
          {t('common:receiver.saveToFolder')}
        </label>
        <div className="flex gap-2">
          <div className="p-3 rounded-md text-sm font-mono flex items-center w-[85%] bg-white/10 text-app-fg">
            {savePath || t('common:receiver.noFolderSelected')}
          </div>
          <button
            onClick={onBrowseFolder}
            disabled={isReceiving}
            className="w-[15%] py-3 px-4 rounded-md font-medium text-sm transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center btn-app-accent"
          >
            {t('common:browse')}
          </button>
        </div>
      </div>

      <div>
        <label className="block mb-2 text-sm font-medium text-app-fg">
          {t('common:receiver.pasteTicket')}
        </label>
        <div className="flex gap-2 p-0.5">
          <textarea
            value={ticket}
            onChange={(e) => onTicketChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                if (ticket.trim() && !isReceiving) {
                  onReceive();
                }
              }
            }}
            placeholder={t('common:receiver.ticketPlaceholder')}
            title={t('common:receiver.pasteTicket')}
            className="p-3 rounded-md text-sm font-mono resize-none focus:outline-none focus:ring-2 w-[85%] bg-white/10 border border-white/20 text-app-fg leading-[1.4] break-words overflow-x-hidden"
            rows={6}
          />
          <button
            onClick={onReceive}
            disabled={!ticket.trim() || isReceiving}
            title={t('common:receive')}
            className={`w-[15%] py-3 px-4 rounded-md font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-80 flex items-center justify-center ${(!ticket.trim() || isReceiving) ? 'btn-app-accent' : 'btn-app-primary'}`}
          >
            <Download className="w-8 h-8" />
          </button>
        </div>
      </div>
    </div>
  )
}
