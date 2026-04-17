import { CheckCircle, AlertCircle, Info } from 'lucide-react'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from './ui/alert-dialog'
import type { AlertType } from '../types/sender'

interface AppAlertDialogProps {
  isOpen: boolean
  title: string
  description: string
  type?: AlertType
  onClose: () => void
}

const typeIcon: Record<AlertType, React.ReactNode> = {
  success: <CheckCircle className="h-5 w-5 text-status-active" />,
  error:   <AlertCircle  className="h-5 w-5 text-red-400" />,
  info:    <Info         className="h-5 w-5 text-status-done" />,
}

export function AppAlertDialog({ 
  isOpen, 
  title, 
  description, 
  type = 'info',
  onClose 
}: AppAlertDialogProps) {
  return (
    <AlertDialog open={isOpen} onOpenChange={onClose}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2">
            {typeIcon[type]}
            {title}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {description}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogAction onClick={onClose}>
            OK
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
