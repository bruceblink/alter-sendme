import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'
import './i18n'
import {initializePlatformStyles} from './lib/platformStyles'
import {applyStoredTheme} from './lib/theme'

applyStoredTheme()
initializePlatformStyles()

// prevent right click menu
document.addEventListener('contextmenu', (e) => e.preventDefault());

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
