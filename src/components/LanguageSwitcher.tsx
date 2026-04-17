import { useState, useRef, useEffect } from 'react'
import { useAppTranslation } from '@/i18n'
import { ChevronDown } from 'lucide-react'

const LANGUAGES = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Русский' },
  { value: 'sr', label: 'Српски' }, 
  { value: 'fr', label: 'Français' },
  { value: 'zh-CN', label: '简体中文' },
  { value: 'zh-TW', label: '繁體中文' },
  { value: 'de', label: 'Deutsch' },
  { value: 'ja', label: '日本語' },
  { value: 'th', label: 'Thai' },
  { value: 'it', label: 'Italiano' },
  { value: 'cs', label: 'Čeština' },
  { value: 'es', label: 'Español' },
  { value: 'pt-BR', label: 'Português' },
  { value: 'ar', label: 'العربية' },
  { value: 'fa', label: 'فارسی' },
  { value: 'ko', label: '한국어' },
  { value: 'hi', label: 'हिन्दी' },
  { value: 'pl', label: 'Polski' },
  { value: 'uk', label: 'Українська' },
  { value: 'tr', label: 'Türkçe' },
  { value: 'no', label: 'Norsk' },
]

export function LanguageSwitcher() {
  const { i18n } = useAppTranslation()
  const [isOpen, setIsOpen] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  const currentLanguage = LANGUAGES.find(lang => lang.value === i18n.language) || LANGUAGES[0]

  const changeLanguage = (lng: string) => {
    i18n.changeLanguage(lng)
    setIsOpen(false)
    window.dispatchEvent(new Event('languagechange'))
  }

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside)
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [isOpen])

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-1 px-2 py-1 text-xs transition-colors hover:opacity-80 text-app-fg underline cursor-pointer"
      >
        {currentLanguage.label}
        <ChevronDown 
          size={12} 
          className={`transition-transform ${isOpen ? 'rotate-0' : 'rotate-180'}`}
        />
      </button>

      {isOpen && (
        <div
          className="language-dropdown absolute right-0 bottom-full mb-1 rounded-md shadow-lg overflow-y-auto z-50 bg-[var(--app-main-view)] border border-white/20 min-w-[120px] max-h-[30vh]"
        >
          {LANGUAGES.map((lang) => (
            <button
              key={lang.value}
              onClick={() => changeLanguage(lang.value)}
              className={`w-full text-left px-3 py-2 text-xs transition-colors cursor-pointer text-app-fg hover:bg-white/15 ${
                i18n.language === lang.value ? 'bg-white/10' : 'bg-transparent'
              }`}
            >
              {lang.label}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

