import {REPOSITORY_URL, SPONSORING_URL} from "@/lib/author.ts";
import {openUrl} from "@tauri-apps/plugin-opener";
import {LanguageSwitcher} from "@/components/LanguageSwitcher.tsx";
import {useTranslation} from "@/i18n";
import {useEffect, useState} from "react";
import {getVersion} from "@tauri-apps/api/app";
import {ThemeSwitcher} from "@/components/ThemeSwitcher.tsx";
import {useAutoUpdater} from "@/hooks/useAutoUpdater.ts";

export function Footer() {
    const [appVersion, setAppVersion] = useState('.....');
    const {t} = useTranslation()
    const {isChecking, statusMessage, checkForUpdates} = useAutoUpdater()

    useEffect(() => {
        void getVersion().then(setAppVersion);
    }, []);

    return (
        <div className="relative flex items-center justify-center w-full h-10 text-xs text-center">
        <span>
            <a target="_blank"
               href={`${REPOSITORY_URL}/releases/tag/v${appVersion}`}
               className="ml-1 font-mono text-sm btn">
                v{appVersion}
            </a>
        </span>
            {statusMessage && (
                <span className="absolute text-[11px] opacity-80 left-1/2 -translate-x-1/2 bottom-2 max-w-[40%] truncate" title={statusMessage}>
                    {statusMessage}
                </span>
            )}
            <button
                onClick={async () => {
                    try {
                        await openUrl(`${SPONSORING_URL}`)
                    } catch (error) {
                        console.error('Failed to open URL:', error)
                    }
                }}
                className="absolute px-2 py-1 text-xs underline transition-colors cursor-pointer left-6 bottom-2 hover:opacity-80 text-app-fg"
            >
                {t('common:donate')}
            </button>
            <div className="absolute flex items-center gap-1 right-4 bottom-2">
                <button
                    onClick={() => void checkForUpdates(true)}
                    disabled={isChecking}
                    className="flex items-center gap-1 px-2 py-1 text-xs underline transition-colors cursor-pointer hover:opacity-80 text-app-fg disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    {isChecking ? t('common:update.checking') : t('common:update.checkNow')}
                </button>
                <ThemeSwitcher/>
                <LanguageSwitcher/>
            </div>
        </div>
    )
}

