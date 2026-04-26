import {REPOSITORY_URL, SPONSORING_URL} from "@/lib/author.ts";
import {openUrl} from "@tauri-apps/plugin-opener";
import {LanguageSwitcher} from "@/components/LanguageSwitcher.tsx";
import {useTranslation} from "@/i18n";
import {useEffect, useState} from "react";
import {getVersion} from "@tauri-apps/api/app";
import {ThemeSwitcher} from "@/components/ThemeSwitcher.tsx";

export function Footer() {
    const [appVersion, setAppVersion] = useState('.....');
    const {t} = useTranslation()

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
                <ThemeSwitcher/>
                <LanguageSwitcher/>
            </div>
        </div>
    )
}

