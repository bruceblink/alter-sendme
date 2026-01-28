import {REPOSITORY_URL, SPONSORING_URL} from "@/lib/author.ts";
import {VERSION_DISPLAY} from "@/lib/version.ts";
import {openUrl} from "@tauri-apps/plugin-opener";
import {LanguageSwitcher} from "@/components/LanguageSwitcher.tsx";
import {useTranslation} from "@/i18n";

export function Footer() {
    const {t} = useTranslation()

    return (
        <div className="w-full h-10 text-center text-xs flex items-center justify-center relative">
        <span>
            <a target="_blank"
               href={`${REPOSITORY_URL}/releases/tag/${VERSION_DISPLAY}`}
               className="btn text-sm ml-1 font-mono">
                {VERSION_DISPLAY}
            </a>
        </span>
            <button
                onClick={async () => {
                    try {
                        await openUrl(`${SPONSORING_URL}`)
                    } catch (error) {
                        console.error('Failed to open URL:', error)
                    }
                }}
                className="absolute left-6 bottom-2 px-2 py-1 text-xs transition-colors hover:opacity-80"
                style={{
                    color: 'var(--app-main-view-fg)',
                    textDecoration: 'underline',
                    backgroundColor: 'transparent',
                    border: 'none',
                    cursor: 'pointer',
                }}
            >
                {t('common:donate')}
            </button>
            <div className="absolute right-4 bottom-2">
                <LanguageSwitcher/>
            </div>
        </div>
    )
}

