import { Upload, CheckCircle, ChevronRight, ChevronDown, Loader2 } from 'lucide-react'
import type { DropzoneProps } from '@/types/sender.ts'
import { useTranslation } from '@/i18n'
import { basenameFromPath } from '@/lib/path'

export function Dropzone({ 
  isDragActive, 
  selectedPath, 
  pathType, 
  showFullPath, 
  isLoading, 
  onToggleFullPath 
}: DropzoneProps) {
  const { t } = useTranslation()
  const getDropzoneClasses = () => {
    const pb = (selectedPath && !isLoading) ? 'pb-8' : 'pb-16'
    const drag = isDragActive
      ? 'border-[var(--app-accent)] bg-[rgba(45,120,220,0.1)]'
      : 'border-white/20 bg-[var(--app-main-view)]'
    return `border-2 border-dashed rounded-[var(--radius-lg)] px-16 pt-16 ${pb} text-center cursor-pointer ${drag} text-app-fg flex items-center justify-center min-h-48 transition-[border-color,background-color] duration-200`
  }

  const getStatusText = () => {
    if (isLoading) return t('common:sender.preparingForTransport')
    if (isDragActive) return t('common:sender.dropFilesHere')
    if (selectedPath) {
      if (pathType === 'directory') return t('common:sender.folderSelected')
      if (pathType === 'file') return t('common:sender.fileSelected')
      return t('common:sender.itemSelected')
    }
    return t('common:sender.dragAndDrop')
  }

  const getSubText = () => {
    if (isLoading) return t('common:sender.pleaseWaitProcessing')
    if (selectedPath) {
      return (
        <div>
          <div className="font-medium cursor-pointer hover:opacity-80 transition-opacity flex items-center justify-center"
            onClick={onToggleFullPath}
            title="Click to toggle full path"
          >
            {basenameFromPath(selectedPath)}
            <span className="-mr-2">
              {showFullPath ? (
                <ChevronDown className="p-0.5 h-6 w-6" size={16} />
              ) : (
                <ChevronRight className="p-0.5 h-6 w-6" size={16} />
              )}
            </span>
          </div>
          <div className={`text-xs mt-1 opacity-75 break-all transition-opacity ${showFullPath ? 'visible' : 'invisible'}`}>
            {selectedPath}
          </div>
        </div>
      )
    }
    return t('common:sender.orBrowse')
  }

  return (
    <div className={getDropzoneClasses()}>
      <div className="space-y-4 w-full">
        <div className="flex justify-center">
          {isLoading ? (
            <Loader2 className="h-12 w-12 animate-spin text-status-done" />
          ) : selectedPath ? (
            <CheckCircle className="h-12 w-12 text-status-active" />
          ) : (
            <Upload className={`h-12 w-12 ${isDragActive ? 'text-status-done' : 'text-white/60'}`} />
          )}
        </div>
        
        <div>
          <p className="text-lg font-medium mb-2 text-app-fg">
            {getStatusText()}
          </p>
          <div className="text-sm text-white/60">
            {getSubText()}
          </div>
        </div>
      </div>
    </div>
  )
}
