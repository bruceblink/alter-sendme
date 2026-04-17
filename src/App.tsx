import {useEffect, useRef, useState} from 'react'
import {motion} from 'framer-motion'
import {Sender} from './components/sender/Sender'
import {Receiver} from './components/receiver/Receiver'
import {TitleBar} from './components/TitleBar'
import {TranslationProvider, useTranslation} from './i18n'
import {Footer} from "@/components/Footer";

function AppContent() {
  const [activeTab, setActiveTab] = useState<'send' | 'receive'>('send')
  const [isSharing, setIsSharing] = useState(false)
  const [isReceiving, setIsReceiving] = useState(false)
  const isInitialRender = useRef(false)
  const { t } = useTranslation()

  useEffect(() => {
      isInitialRender.current = true
  }, [])

  return (
    <div className="relative flex flex-col h-screen select-none glass-background">
      {IS_LINUX && <TitleBar title={t('common:appTitle')} />}
      
      {IS_MACOS && (
        <div 
          className="absolute z-10 w-full h-10" 
          data-tauri-drag-region 
        />
      )}
      
      <div className="container flex-1 p-8 mx-auto overflow-auto">
        <div className="max-w-2xl mx-auto">
          <h1
            className="text-3xl font-bold font-mono text-center mb-8 select-none [@media(min-height:680px)]:block hidden"
          >
            {t('common:appTitle')}
          </h1>
          
          <div className="relative flex p-1 mb-6 space-x-1 rounded-lg select-none bg-white/10">
            <motion.div
              layoutId="activeTab"
              className="absolute h-[calc(100%-8px)] rounded-md bg-[var(--app-main-view)] border border-white/20 shadow-sm"
              initial={false}
              animate={{
                left: activeTab === 'send' ? '4px' : 'calc(50% + 2px)',
                width: 'calc(50% - 6px)',
              }}
           
            />
            
            <motion.button
              onClick={() => setActiveTab('send')}
              disabled={isReceiving}
              className={`flex-1 py-2 px-4 rounded-md text-sm font-medium relative z-10 text-app-fg ${
                activeTab === 'send'
                  ? ''
                  : 'opacity-70'
              }`}
             
              whileTap={{ scale: 0.98 }}
              transition={{ duration: 0.2 }}
            >
              {t('common:send')}
            </motion.button>
            <motion.button
              onClick={() => setActiveTab('receive')}
              disabled={isSharing}
              className={`flex-1 py-2 px-4 rounded-md text-sm font-medium relative z-10 text-app-fg ${
                activeTab === 'receive'
                  ? ''
                  : 'opacity-70'
              }`}
             
              whileTap={{ scale: 0.98 }}
              transition={{ duration: 0.2 }}
            >
              {t('common:receive')}
            </motion.button>
          </div>
          
          <div 
            className="overflow-hidden rounded-lg shadow-sm glass-card"
          >
        
              {activeTab === 'send' ? (
                <motion.div
                  key="send"
                  initial={isInitialRender.current ? { opacity: 0, x: -20 } : false }
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 20 }}
                >
                  <Sender onTransferStateChange={setIsSharing} />
                </motion.div>
              ) : (
                <motion.div
                  key="receive"
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                 
                >
                  <Receiver onTransferStateChange={setIsReceiving} />
                </motion.div>
              )}
          
          </div>
        </div>
      </div>
        <Footer/>
    </div>
  )
}

function App() {
  return (
    <TranslationProvider>
      <AppContent />
    </TranslationProvider>
  )
}

export default App
