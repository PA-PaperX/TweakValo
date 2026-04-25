import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { t, type Lang } from './i18n';
import {
  applyNvidiaProfile,
  applyStretch,
  checkGamePath,
  checkRiotClient,
  getConfig,
  getGpuInfo,
  getResolutions,
  launchGame,
  openNvidiaCp,
  revertStretch,
  updateConfigField,
  setTimerResolution,
  purgeStandbyList,
  type AppConfig,
  type GpuInfo,
} from './lib/commands';
import './index.css';

const fadeIn = {
  initial: { opacity: 0, y: 6 },
  animate: { opacity: 1, y: 0 },
  transition: { duration: 0.18 },
};

type LogEntry = { time: string; msg: string };
type Tone = 'neutral' | 'good' | 'warn' | 'accent';

function getLogColor(msg: string) {
  const lower = msg.toLowerCase();
  if (
    lower.includes('error') ||
    lower.includes('fail') ||
    lower.includes('timeout') ||
    lower.includes('ล้มเหลว') ||
    lower.includes('ข้อผิดพลาด')
  ) {
    return 'text-rose-400 font-medium';
  }
  if (
    lower.includes('success') ||
    lower.includes('ready') ||
    lower.includes('done') ||
    lower.includes('เสร็จ') ||
    lower.includes('detected') ||
    lower.includes('เจอกลุ่ม') ||
    lower.includes('triggered')
  ) {
    return 'text-emerald-400 font-medium';
  }
  if (lower.includes('warning') || lower.includes('alert')) {
    return 'text-amber-400';
  }
  if (lower.includes('nvidia') || lower.includes('gpu')) {
    return 'text-sky-400';
  }
  if (lower.includes('mods') || lower.includes('blood') || lower.includes('vng')) {
    return 'text-fuchsia-400';
  }
  if (lower.includes('riot client') || lower.includes('play')) {
    return 'text-orange-400';
  }
  return 'text-zinc-300';
}

function SectionCard({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <section className="panel p-5 sm:p-6">
      <div className="space-y-1">
        <p className="section-caption">{title}</p>
        <p className="text-sm leading-6 text-zinc-500">{description}</p>
      </div>
      <div className="mt-5">{children}</div>
    </section>
  );
}

function StatusBadge({ label, tone = 'neutral' }: { label: string; tone?: Tone }) {
  return <span className={`status-badge tone-${tone}`}>{label}</span>;
}

function ToggleRow({
  title,
  description,
  risk,
  checked,
  onChange,
}: {
  title: string;
  description: string;
  risk: string;
  checked: boolean;
  onChange: (nextValue: boolean) => void;
}) {
  return (
    <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4">
      <div className="flex items-start justify-between gap-4">
        <div className="space-y-1">
          <h3 className="text-sm font-semibold text-zinc-950">{title}</h3>
          <p className="text-sm leading-6 text-zinc-500">{description}</p>
        </div>
        <label className="toggle-switch">
          <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
          <span className="toggle-slider" />
        </label>
      </div>
      <p className="mt-3 flex items-start gap-2 text-xs leading-5 text-zinc-500">
        <span className="mt-1 h-2 w-2 rounded-full bg-emerald-500" />
        <span>{risk}</span>
      </p>
    </div>
  );
}

function ModalShell({
  children,
  onClose,
}: {
  children: ReactNode;
  onClose: () => void;
}) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="modal-backdrop fixed inset-0 z-50 flex items-center justify-center p-4"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      {children}
    </motion.div>
  );
}

export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [lang, setLang] = useState<Lang>('th');
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [launching, setLaunching] = useState(false);
  const [riotFound, setRiotFound] = useState(false);
  const [pathValid, setPathValid] = useState(false);
  const [gpuInfo, setGpuInfo] = useState<GpuInfo | null>(null);
  const [gpuCategory, setGpuCategory] = useState('unknown');
  const [resolutions, setResolutions] = useState<[string, string, string][]>([]);
  const [selectedRes, setSelectedRes] = useState('1440x1080');
  const [customW, setCustomW] = useState('');
  const [customH, setCustomH] = useState('');
  const [stretchLinked, setStretchLinked] = useState(false);
  const [nvidiaLinked, setNvidiaLinked] = useState(false);
  const [lodPreset, setLodPreset] = useState('low');
  const [nativeW, setNativeW] = useState(0);
  const [nativeH, setNativeH] = useState(0);
  const [showWelcome, setShowWelcome] = useState(false);
  const [showGpuPicker, setShowGpuPicker] = useState(false);
  const [showNvTutorial, setShowNvTutorial] = useState<{ w: number; h: number } | null>(null);
  const [autoTimer, setAutoTimer] = useState(false);
  const [autoStandby, setAutoStandby] = useState(false);
  const [autoPriority, setAutoPriority] = useState(false);
  const [autoAffinity, setAutoAffinity] = useState(false);
  const logRef = useRef<HTMLDivElement>(null);
  const langMenuRef = useRef<HTMLDivElement>(null);
  const [showLangMenu, setShowLangMenu] = useState(false);
  const [welcomeAccepted, setWelcomeAccepted] = useState(false);

  const addLog = useCallback((msg: string) => {
    const time = new Date().toLocaleTimeString('en-GB', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
    setLogs((prev) => [...prev, { time, msg }]);
    setTimeout(() => {
      logRef.current?.scrollTo({ top: logRef.current.scrollHeight, behavior: 'smooth' });
    }, 50);
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        const cfg = await getConfig();
        const nextLang = cfg.language === 'en' ? 'en' : 'th';
        setConfig(cfg);
        setLang(nextLang);
        setCustomW(cfg.custom_width || '');
        setCustomH(cfg.custom_height || '');
        if (cfg.last_resolution) {
          setSelectedRes(cfg.last_resolution);
        }
        if (!cfg.welcome_shown) {
          setShowWelcome(true);
        }
        setAutoTimer(cfg.auto_timer_resolution || false);
        setAutoStandby(cfg.auto_standby_cleaner || false);
        setAutoPriority(cfg.auto_priority || false);
        setAutoAffinity(cfg.auto_affinity || false);

        const [riot, path, gpu, res] = await Promise.all([
          checkRiotClient(cfg.riot_client_path),
          checkGamePath(cfg.game_path),
          getGpuInfo(),
          getResolutions(),
        ]);

        setRiotFound(riot.success);
        setPathValid(path.success);
        setGpuInfo(gpu);
        setGpuCategory(cfg.gpu_override && cfg.gpu_override !== 'auto' ? cfg.gpu_override : gpu.category);
        setResolutions(res);
      } catch (error) {
        addLog(`Init error: ${String(error)}`);
      }
    })();
  }, [addLog]);

  useEffect(() => {
    if (!showLangMenu) {
      return;
    }

    const handlePointerDown = (event: MouseEvent) => {
      if (langMenuRef.current && !langMenuRef.current.contains(event.target as Node)) {
        setShowLangMenu(false);
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setShowLangMenu(false);
      }
    };

    window.addEventListener('mousedown', handlePointerDown);
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('mousedown', handlePointerDown);
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [showLangMenu]);

  const getResolutionDimensions = () => {
    if (selectedRes === 'custom') {
      return {
        width: parseInt(customW, 10) || 0,
        height: parseInt(customH, 10) || 0,
      };
    }

    const selected = resolutions.find(([key]) => key === selectedRes);
    if (!selected) {
      return { width: 0, height: 0 };
    }

    const [width, height] = selected[1].split(' x ');
    return {
      width: parseInt(width, 10) || 0,
      height: parseInt(height, 10) || 0,
    };
  };

  const persistGamePath = async (nextPath: string) => {
    await updateConfigField('game_path', nextPath);
    const result = await checkGamePath(nextPath);
    setPathValid(result.success);
  };

  const handleLangChange = async (nextLang: Lang) => {
    setShowLangMenu(false);
    setLang(nextLang);
    setConfig((prev) => (prev ? { ...prev, language: nextLang } : prev));
    await updateConfigField('language', nextLang);
  };

  const handleBrowseFolder = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({ directory: true, title: 'Select VALORANT Folder' });
      if (selected && typeof selected === 'string') {
        setConfig((prev) => (prev ? { ...prev, game_path: selected } : prev));
        await persistGamePath(selected);
      }
    } catch (error) {
      addLog(`Browse error: ${String(error)}`);
    }
  };

  const handleBrowseRiot = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const defaultPath = config?.riot_client_path || 'C:\\Riot Games\\Riot Client\\RiotClientServices.exe';
      const selected = await open({
        title: 'Select RiotClientServices.exe',
        defaultPath,
        filters: [{ name: 'Executable (*RiotClientServices.exe)', extensions: ['exe'] }],
      });

      if (selected && typeof selected === 'string') {
        if (!/riotclientservices\.exe$/i.test(selected)) {
          addLog(
            lang === 'th'
              ? 'กรุณาเลือกไฟล์ RiotClientServices.exe เท่านั้น'
              : 'Please select RiotClientServices.exe only.',
          );
          return;
        }

        await updateConfigField('riot_client_path', selected);
        setConfig((prev) => (prev ? { ...prev, riot_client_path: selected } : prev));
        setRiotFound(true);
      }
    } catch (error) {
      addLog(`Browse error: ${String(error)}`);
    }
  };

  const handleLaunch = async () => {
    if (!config || launching) {
      return;
    }

    setLaunching(true);
    addLog(t(lang, 'startSeq'));

    try {
      if (stretchLinked) {
        const { width, height } = getResolutionDimensions();
        const result = await applyStretch(selectedRes, width, height);

        if (result.success) {
          setNativeW(result.native_w);
          setNativeH(result.native_h);
          addLog(result.message);
        } else if (result.needs_custom_res) {
          setShowNvTutorial({ w: width, h: height });
        } else {
          addLog(result.message);
        }
      }

      if (nvidiaLinked) {
        const [, msg] = await applyNvidiaProfile(lodPreset);
        addLog(msg);
      }

      if (autoStandby) {
        addLog(t(lang, 'logPurgingMemory'));
        const purgeRes = await purgeStandbyList();
        addLog(purgeRes.message);
      }

      if (autoTimer) {
        addLog(t(lang, 'logSettingTimer'));
        const timerRes = await setTimerResolution(true);
        addLog(timerRes.message);
      }

      const results = await launchGame(config.game_path, config.riot_client_path, config.enable_blood);
      results.forEach((msg) => addLog(msg));
      addLog(t(lang, 'done'));
    } catch (error) {
      addLog(`${t(lang, 'error')}: ${String(error)}`);
    } finally {
      setLaunching(false);
    }
  };

  const handleApplyStretch = async () => {
    const { width, height } = getResolutionDimensions();
    addLog(t(lang, 'logStretchApplying'));

    try {
      const result = await applyStretch(selectedRes, width, height);

      if (result.needs_custom_res) {
        setShowNvTutorial({ w: width || 1440, h: height || 1080 });
        return;
      }

      if (result.success) {
        setNativeW(result.native_w);
        setNativeH(result.native_h);
        await updateConfigField('last_resolution', selectedRes);
        if (selectedRes === 'custom') {
          await updateConfigField('custom_width', String(width));
          await updateConfigField('custom_height', String(height));
        }
      }

      addLog(result.message);
    } catch (error) {
      addLog(`${t(lang, 'error')}: ${String(error)}`);
    }
  };

  const handleRevertStretch = async () => {
    addLog(t(lang, 'logStretchReverting'));
    try {
      const result = await revertStretch(nativeW, nativeH);
      addLog(result.message);
    } catch (error) {
      addLog(`${t(lang, 'error')}: ${String(error)}`);
    }
  };

  const handleApplyNvidia = async () => {
    addLog(t(lang, 'logNvidiaApplying'));
    try {
      const [, msg] = await applyNvidiaProfile(lodPreset);
      addLog(msg);
    } catch (error) {
      addLog(`${t(lang, 'error')}: ${String(error)}`);
    }
  };

  const handleRestoreNvidia = async () => {
    addLog(t(lang, 'logNvidiaRestoring'));
    try {
      const [, msg] = await applyNvidiaProfile("restore");
      addLog(msg);
    } catch (error) {
      addLog(`${t(lang, 'error')}: ${String(error)}`);
    }
  };

  const handleWelcomeClose = async () => {
    setShowWelcome(false);
    await updateConfigField('welcome_shown', 'true');
  };

  const isNvidiaDisabled = gpuCategory === 'non_nvidia';
  const gpuNames = gpuInfo?.names.join(', ') || t(lang, 'detecting');
  const languageMenuLabel = lang === 'th' ? 'ภาษา' : 'Language';
  const selectedResolutionLabel =
    selectedRes === 'custom'
      ? `${customW || '-'} x ${customH || '-'}`
      : (resolutions.find(([key]) => key === selectedRes)?.[1] ?? selectedRes);

  if (!config) {
    return (
      <div className="flex h-full items-center justify-center bg-zinc-50">
        <div className="text-sm text-zinc-500">{t(lang, 'loading')}</div>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <div className="app-scroll">
        <div className="mx-auto flex w-full max-w-4xl flex-col gap-4">
          <motion.header {...fadeIn} className="panel p-5 sm:p-6">
            <div className="flex flex-col gap-5">
              <div className="flex items-start justify-between gap-3">
                <button className="secondary-button toolbar-button" onClick={() => setShowGpuPicker(true)}>
                  {t(lang, 'gpuButton')}
                </button>

                <div className="relative" ref={langMenuRef}>
                  <button
                    className="secondary-button icon-button"
                    type="button"
                    aria-haspopup="menu"
                    aria-expanded={showLangMenu}
                    aria-label={languageMenuLabel}
                    title={languageMenuLabel}
                    onClick={() => setShowLangMenu((prev) => !prev)}
                  >
                    <svg viewBox="0 0 24 24" aria-hidden="true" className="h-5 w-5">
                      <path
                        d="M12 2.75a9.25 9.25 0 1 0 0 18.5a9.25 9.25 0 0 0 0-18.5Zm6.98 8.5h-3.07a14.3 14.3 0 0 0-1.45-5.04a7.77 7.77 0 0 1 4.52 5.04Zm-6.23-5.8c.77.93 1.48 2.93 1.68 5.8h-4.86c.2-2.87.91-4.87 1.68-5.8c.45-.55.84-.7.75-.7s.3.15.75.7Zm-4.21.76a14.3 14.3 0 0 0-1.45 5.04H4.02a7.77 7.77 0 0 1 4.52-5.04Zm-4.52 6.54h3.07a14.3 14.3 0 0 0 1.45 5.04a7.77 7.77 0 0 1-4.52-5.04Zm5.48 0h4.98c-.21 2.87-.91 4.87-1.68 5.8c-.45.55-.84.7-.75.7s-.3-.15-.75-.7c-.77-.93-1.47-2.93-1.8-5.8Zm0-1.5c.2-2.87.91-4.87 1.68-5.8c.45-.55.84-.7.75-.7s.3.15.75.7c.77.93 1.47 2.93 1.68 5.8H9.5Zm4.98 1.5h4.5a7.77 7.77 0 0 1-4.52 5.04a14.29 14.29 0 0 0 1.45-5.04Z"
                        fill="currentColor"
                      />
                    </svg>
                  </button>

                  <AnimatePresence initial={false}>
                    {showLangMenu && (
                      <motion.div
                        initial={{ opacity: 0, y: -4 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -4 }}
                        transition={{ duration: 0.14 }}
                        className="language-menu"
                        role="menu"
                      >
                        {(['th', 'en'] as Lang[]).map((option) => (
                          <button
                            key={option}
                            className={`language-option ${lang === option ? 'is-active' : ''}`}
                            type="button"
                            role="menuitemradio"
                            aria-checked={lang === option}
                            onClick={() => {
                              void handleLangChange(option);
                            }}
                          >
                            {option.toUpperCase()}
                          </button>
                        ))}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              </div>

              <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                <div className="space-y-2">
                  <p className="section-caption">{t(lang, 'appTag')}</p>
                  <div>
                    <h1 className="text-3xl font-semibold tracking-tight text-zinc-950">{t(lang, 'appName')}</h1>
                    <p className="mt-2 max-w-2xl text-sm leading-6 text-zinc-500">{t(lang, 'appSubtitle')}</p>
                  </div>
                  <p className="text-xs font-medium text-zinc-400">{t(lang, 'developedBy')}</p>
                </div>
              </div>

              <div className="grid gap-3 md:grid-cols-3">
                <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4">
                  <p className="section-caption">{t(lang, 'summaryGame')}</p>
                  <p className="mt-2 break-all text-sm font-medium text-zinc-950">
                    {config.game_path || 'C:\\Riot Games\\VALORANT'}
                  </p>
                </div>
                <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4">
                  <p className="section-caption">{t(lang, 'summaryRiot')}</p>
                  <div className="mt-2">
                    <StatusBadge
                      tone={riotFound ? 'good' : 'warn'}
                      label={riotFound ? t(lang, 'riotFound') : t(lang, 'riotMissing')}
                    />
                  </div>
                </div>
                <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4">
                  <p className="section-caption">{t(lang, 'summaryGpu')}</p>
                  <p className="mt-2 break-words text-sm font-medium text-zinc-950">{gpuNames}</p>
                </div>
              </div>
            </div>
          </motion.header>

          <motion.section
            {...fadeIn}
            transition={{ delay: 0.04, duration: 0.24 }}
            className="rounded-2xl border border-amber-200/80 bg-amber-50 p-4"
          >
            <div className="flex items-start gap-3">
              <span className="mt-1 h-2.5 w-2.5 rounded-full bg-amber-500" />
              <div className="space-y-1">
                <h2 className="text-sm font-semibold text-amber-900">{t(lang, 'warningTitle')}</h2>
                <p className="text-sm leading-6 text-amber-700">{t(lang, 'warningBody')}</p>
              </div>
            </div>
          </motion.section>

          <div className="mt-2 space-y-1">
            <h2 className="text-xl font-semibold text-emerald-600">{t(lang, 'safeMode')}</h2>
            <p className="text-sm text-zinc-500">{t(lang, 'safeModeDesc')}</p>
          </div>

          <div className="grid items-start gap-4 lg:grid-cols-2">
            {/* ── Left Column ── */}
            <div className="flex flex-col gap-4">
              <motion.div {...fadeIn} transition={{ delay: 0.08, duration: 0.24 }}>
                <SectionCard title={t(lang, 'gameFolderTitle')} description={t(lang, 'gameFolderDesc')}>
                  <div className="flex flex-col gap-3">
                    <div className="flex flex-col gap-3 sm:flex-row">
                      <input
                        className="input-field flex-1"
                        placeholder={t(lang, 'gameFolderPlaceholder')}
                        value={config.game_path}
                        onChange={(event) =>
                          setConfig((prev) => (prev ? { ...prev, game_path: event.target.value } : prev))
                        }
                        onBlur={() => {
                          void persistGamePath(config.game_path);
                        }}
                      />
                      <button className="secondary-button sm:min-w-28" onClick={handleBrowseFolder}>
                        {t(lang, 'browse')}
                      </button>
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      <StatusBadge
                        tone={pathValid ? 'good' : 'warn'}
                        label={
                          pathValid
                            ? config.game_path
                              ? t(lang, 'pathReadyCustom')
                              : t(lang, 'pathReadyDefault')
                            : t(lang, 'pathMissing')
                        }
                      />
                    </div>
                  </div>
                </SectionCard>
              </motion.div>

              <motion.div {...fadeIn} transition={{ delay: 0.12, duration: 0.24 }}>
                <SectionCard title={t(lang, 'stretchTitle')} description={t(lang, 'stretchDesc')}>
                  <div className={`space-y-4 ${isNvidiaDisabled ? 'opacity-60' : ''}`}>
                    <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                      <div className="space-y-1">
                        <p className="text-sm font-semibold text-zinc-950">{t(lang, 'resolution')}</p>
                        <StatusBadge
                          tone={stretchLinked ? 'accent' : 'neutral'}
                          label={stretchLinked ? t(lang, 'statusLinked') : t(lang, 'statusManual')}
                        />
                      </div>
                      <button
                        className="secondary-button sm:min-w-36"
                        disabled={isNvidiaDisabled}
                        onClick={() => {
                          const nextValue = !stretchLinked;
                          setStretchLinked(nextValue);
                          addLog(t(lang, nextValue ? 'logStretchLinked' : 'logStretchUnlinked'));
                        }}
                      >
                        {stretchLinked ? t(lang, 'unlinkFromLaunch') : t(lang, 'linkToLaunch')}
                      </button>
                    </div>

                    <div className="grid gap-3">
                      <select
                        className="select-field"
                        disabled={isNvidiaDisabled}
                        value={selectedRes}
                        onChange={(event) => setSelectedRes(event.target.value)}
                      >
                        {resolutions.map(([key, label, description]) => (
                          <option key={key} value={key}>
                            {label} - {description}
                          </option>
                        ))}
                        <option value="custom">{t(lang, 'customRes')}</option>
                      </select>

                      <AnimatePresence initial={false}>
                        {selectedRes === 'custom' && (
                          <motion.div
                            initial={{ opacity: 0, height: 0 }}
                            animate={{ opacity: 1, height: 'auto' }}
                            exit={{ opacity: 0, height: 0 }}
                            className="grid gap-3 overflow-hidden sm:grid-cols-2"
                          >
                            <input
                              className="input-field"
                              inputMode="numeric"
                              placeholder={t(lang, 'customWidth')}
                              disabled={isNvidiaDisabled}
                              value={customW}
                              onChange={(event) => setCustomW(event.target.value.replace(/[^0-9]/g, ''))}
                            />
                            <input
                              className="input-field"
                              inputMode="numeric"
                              placeholder={t(lang, 'customHeight')}
                              disabled={isNvidiaDisabled}
                              value={customH}
                              onChange={(event) => setCustomH(event.target.value.replace(/[^0-9]/g, ''))}
                            />
                          </motion.div>
                        )}
                      </AnimatePresence>
                    </div>

                    <div className="flex flex-col gap-3 sm:flex-row">
                      <button className="secondary-button flex-1" disabled={isNvidiaDisabled} onClick={handleApplyStretch}>
                        {t(lang, 'applyStretch')}
                      </button>
                      <button className="secondary-button sm:min-w-32" disabled={isNvidiaDisabled} onClick={handleRevertStretch}>
                        {t(lang, 'revertStretch')}
                      </button>
                    </div>

                    <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4">
                      <p className="text-sm font-medium text-zinc-950">{selectedResolutionLabel}</p>
                      <p className="mt-1 text-sm leading-6 text-zinc-500">{t(lang, 'stretchHint')}</p>
                      <p className="mt-3 flex items-start gap-2 text-xs leading-5 text-zinc-500">
                        <span className="mt-1 h-2 w-2 rounded-full bg-emerald-500" />
                        <span>{t(lang, 'riskNoneConfig')}</span>
                      </p>
                    </div>

                    {isNvidiaDisabled && <p className="text-sm leading-6 text-zinc-500">{t(lang, 'nvidiaDisabled')}</p>}
                  </div>
                </SectionCard>
              </motion.div>

              <motion.div {...fadeIn} transition={{ delay: 0.16, duration: 0.24 }}>
                <SectionCard title={t(lang, 'systemTitle')} description={t(lang, 'systemDesc')}>
                  <div className="flex flex-col gap-6">
                    <ToggleRow
                      title={t(lang, 'timerTitle')}
                      description={t(lang, 'timerDesc')}
                      risk={t(lang, 'riskNoneConfig')}
                      checked={autoTimer}
                      onChange={(nextValue) => {
                        setAutoTimer(nextValue);
                        void updateConfigField('auto_timer_resolution', String(nextValue));
                      }}
                    />
                    <ToggleRow
                      title={t(lang, 'standbyTitle')}
                      description={t(lang, 'standbyDesc')}
                      risk={t(lang, 'riskNoneConfig')}
                      checked={autoStandby}
                      onChange={(nextValue) => {
                        setAutoStandby(nextValue);
                        void updateConfigField('auto_standby_cleaner', String(nextValue));
                      }}
                    />
                    <ToggleRow
                      title={t(lang, 'priorityTitle')}
                      description={t(lang, 'priorityDesc')}
                      risk={t(lang, 'riskNoneConfig')}
                      checked={autoPriority}
                      onChange={(nextValue) => {
                        setAutoPriority(nextValue);
                        void updateConfigField('auto_priority', String(nextValue));
                      }}
                    />
                    <ToggleRow
                      title={t(lang, 'affinityTitle')}
                      description={t(lang, 'affinityDesc')}
                      risk={t(lang, 'riskNoneConfig')}
                      checked={autoAffinity}
                      onChange={(nextValue) => {
                        setAutoAffinity(nextValue);
                        void updateConfigField('auto_affinity', String(nextValue));
                      }}
                    />
                  </div>
                </SectionCard>
              </motion.div>
            </div>

            {/* ── Right Column ── */}
            <div className="flex flex-col gap-4">
              <motion.div {...fadeIn} transition={{ delay: 0.08, duration: 0.24 }}>
                <SectionCard title={t(lang, 'launchTitle')} description={t(lang, 'launchDesc')}>
                  <div className="space-y-4">
                    <div className="flex flex-wrap gap-2">
                      {stretchLinked && <StatusBadge tone="accent" label={t(lang, 'stretchLinkedBadge')} />}
                      {nvidiaLinked && <StatusBadge tone="accent" label={t(lang, 'nvidiaLinkedBadge')} />}
                      {!stretchLinked && !nvidiaLinked && (
                        <StatusBadge tone="neutral" label={t(lang, 'launchManualBadge')} />
                      )}
                    </div>

                    <button className="primary-button w-full" disabled={launching} onClick={handleLaunch}>
                      {launching ? t(lang, 'launching') : t(lang, 'play')}
                    </button>

                    <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4">
                      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                        <div className="space-y-1">
                          <p className="text-sm font-semibold text-zinc-950">{t(lang, 'riotClientTitle')}</p>
                          <StatusBadge
                            tone={riotFound ? 'good' : 'warn'}
                            label={riotFound ? t(lang, 'riotFound') : t(lang, 'riotMissing')}
                          />
                        </div>
                        <button className="secondary-button sm:min-w-28" onClick={handleBrowseRiot}>
                          {t(lang, 'riotBrowse')}
                        </button>
                      </div>
                    </div>
                  </div>
                </SectionCard>
              </motion.div>

              <motion.div {...fadeIn} transition={{ delay: 0.12, duration: 0.24 }}>
                <SectionCard title={t(lang, 'logTitle')} description={t(lang, 'logDesc')}>
                  <div ref={logRef} className="log-surface h-40 overflow-y-auto">
                    {logs.length === 0 ? (
                      <p className="text-zinc-400">{t(lang, 'logEmpty')}</p>
                    ) : (
                      logs.map((entry, index) => (
                        <p key={`${entry.time}-${index}`} className={`text-sm leading-6 ${getLogColor(entry.msg)}`}>
                          <span className="mr-1.5 opacity-60 text-zinc-400 font-normal">[{entry.time}]</span>
                          <span>{entry.msg}</span>
                        </p>
                      ))
                    )}
                  </div>
                </SectionCard>
              </motion.div>

              <motion.div {...fadeIn} transition={{ delay: 0.16, duration: 0.24 }}>
                <SectionCard title={t(lang, 'nvidiaTitle')} description={t(lang, 'nvidiaDesc')}>
                  <div className={`space-y-4 ${isNvidiaDisabled ? 'opacity-60' : ''}`}>
                    <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                      <div className="space-y-1">
                        <p className="text-sm font-semibold text-zinc-950">{t(lang, 'lodPreset')}</p>
                        <StatusBadge
                          tone={nvidiaLinked ? 'accent' : 'neutral'}
                          label={nvidiaLinked ? t(lang, 'statusLinked') : t(lang, 'statusManual')}
                        />
                      </div>
                      <button
                        className="secondary-button sm:min-w-36"
                        disabled={isNvidiaDisabled}
                        onClick={() => {
                          const nextValue = !nvidiaLinked;
                          setNvidiaLinked(nextValue);
                          addLog(t(lang, nextValue ? 'logNvidiaLinked' : 'logNvidiaUnlinked'));
                        }}
                      >
                        {nvidiaLinked ? t(lang, 'unlinkFromLaunch') : t(lang, 'linkToLaunch')}
                      </button>
                    </div>

                    <select
                      className="select-field"
                      disabled={isNvidiaDisabled}
                      value={lodPreset}
                      onChange={(event) => setLodPreset(event.target.value)}
                    >
                      <option value="low">{t(lang, 'presetLow')}</option>
                      <option value="medium">{t(lang, 'presetMedium')}</option>
                      <option value="high">{t(lang, 'presetHigh')}</option>
                      <option value="dindommum">{t(lang, 'presetCompetitive')}</option>
                    </select>

                    <div className="flex flex-col gap-3 sm:flex-row">
                      <button className="secondary-button flex-1" disabled={isNvidiaDisabled} onClick={handleApplyNvidia}>
                        {t(lang, 'applyNvidia')}
                      </button>
                      <button className="secondary-button sm:min-w-32" disabled={isNvidiaDisabled} onClick={handleRestoreNvidia}>
                        {t(lang, 'restoreNvidia')}
                      </button>
                    </div>

                    <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4">
                      <p className="text-sm leading-6 text-zinc-500">{t(lang, 'nvidiaHint')}</p>
                      <p className="mt-3 flex items-start gap-2 text-xs leading-5 text-zinc-500">
                        <span className="mt-1 h-2 w-2 rounded-full bg-emerald-500" />
                        <span>{t(lang, 'riskNoneDriver')}</span>
                      </p>
                    </div>

                    {isNvidiaDisabled && <p className="text-sm leading-6 text-zinc-500">{t(lang, 'nvidiaDisabled')}</p>}
                  </div>
                </SectionCard>
              </motion.div>

            </div>
          </div>

          <div className="mt-6 space-y-1 border-t border-zinc-200/80 pt-6">
            <h2 className="text-xl font-semibold text-rose-600">{t(lang, 'advMode')}</h2>
            <p className="text-sm text-zinc-500">{t(lang, 'advModeDesc')}</p>
          </div>

          <div className="grid items-start gap-4 lg:grid-cols-2">
            <div className="flex flex-col gap-4">
              <motion.div {...fadeIn} transition={{ delay: 0.20, duration: 0.24 }}>
                <SectionCard title={t(lang, 'contentTitle')} description={t(lang, 'contentDesc')}>
                  <ToggleRow
                    title={t(lang, 'matureContent')}
                    description={t(lang, 'matureContentDesc')}
                    risk={t(lang, 'riskHigh')}
                    checked={config.enable_blood}
                    onChange={(nextValue) => {
                      setConfig((prev) => (prev ? { ...prev, enable_blood: nextValue } : prev));
                      void updateConfigField('enable_blood', String(nextValue));
                    }}
                  />
                </SectionCard>
              </motion.div>
            </div>
          </div>

          <motion.p
            {...fadeIn}
            transition={{ delay: 0.32, duration: 0.24 }}
            className="pb-4 text-center text-xs leading-6 text-zinc-400"
          >
            {t(lang, 'disclaimer')}
          </motion.p>
        </div>
      </div>

      <AnimatePresence>
        {showWelcome && (
          <ModalShell onClose={() => void handleWelcomeClose()}>
            <motion.div
              initial={{ scale: 0.96, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.96, opacity: 0 }}
              className="panel w-full max-w-lg p-6"
            >
              <div className="space-y-4">
                <div className="space-y-2">
                  <p className="section-caption">{t(lang, 'welcomeEyebrow')}</p>
                  <h2 className="text-2xl font-semibold tracking-tight text-zinc-950">{t(lang, 'welcomeTitle')}</h2>
                  <p className="text-sm leading-6 text-zinc-500">{t(lang, 'welcomeBody')}</p>
                </div>

                <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4">
                  <p className="text-sm font-semibold text-zinc-950">{t(lang, 'welcomeFollow')}</p>
                  <div className="mt-3 space-y-2 text-sm">
                    {[
                      ['Discord', 'https://discord.gg/XpPrx7vUjm'],
                      ['YouTube', 'https://www.youtube.com/@PaperX-kr3yb'],
                      ['TikTok', 'https://www.tiktok.com/@paflimhacker'],
                    ].map(([name, url]) => (
                      <a key={name} href={url} target="_blank" rel="noopener" className="link-text block break-all">
                        {name}
                      </a>
                    ))}
                  </div>
                </div>

                <label className="flex cursor-pointer items-start gap-3 rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4 transition hover:bg-zinc-100">
                  <input
                    type="checkbox"
                    className="mt-0.5 h-4 w-4 appearance-none rounded border border-zinc-300 bg-white checked:border-zinc-950 checked:bg-zinc-950 flex-shrink-0"
                    style={{
                      backgroundImage: welcomeAccepted ? `url("data:image/svg+xml,%3Csvg viewBox='0 0 16 16' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z' fill='white'/%3E%3C/svg%3E")` : 'none',
                      backgroundSize: '100% 100%'
                    }}
                    checked={welcomeAccepted}
                    onChange={(e) => setWelcomeAccepted(e.target.checked)}
                  />
                  <span className="text-sm font-medium text-zinc-950">{t(lang, 'welcomeAcceptRisk')}</span>
                </label>

                <button 
                  className="primary-button w-full" 
                  disabled={!welcomeAccepted} 
                  onClick={() => void handleWelcomeClose()}
                >
                  {t(lang, 'welcomeClose')}
                </button>
              </div>
            </motion.div>
          </ModalShell>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {showGpuPicker && (
          <ModalShell onClose={() => setShowGpuPicker(false)}>
            <motion.div
              initial={{ scale: 0.96, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.96, opacity: 0 }}
              className="panel w-full max-w-md p-6"
            >
              <div className="space-y-4">
                <div className="space-y-1">
                  <p className="section-caption">{t(lang, 'gpuButton')}</p>
                  <h2 className="text-2xl font-semibold tracking-tight text-zinc-950">{t(lang, 'gpuPickerTitle')}</h2>
                  <p className="text-sm leading-6 text-zinc-500">{t(lang, 'gpuPickerSubtitle')}</p>
                </div>

                <div className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4 text-sm text-zinc-500">
                  {gpuInfo?.names.map((name) => (
                    <p key={name}>{name}</p>
                  )) || <p>{t(lang, 'detecting')}</p>}
                </div>

                <div className="space-y-2">
                  {[
                    { value: 'auto', label: t(lang, 'gpuModeAuto') },
                    { value: 'nvidia_only', label: t(lang, 'gpuModeNvidia') },
                    { value: 'hybrid', label: t(lang, 'gpuModeHybrid') },
                    { value: 'non_nvidia', label: t(lang, 'gpuModeNonNvidia') },
                  ].map((option) => (
                    <label
                      key={option.value}
                      className={`flex cursor-pointer items-center gap-3 rounded-2xl border p-4 transition ${
                        gpuCategory === option.value
                          ? 'border-zinc-950 bg-zinc-950 text-white'
                          : 'border-zinc-200/70 bg-zinc-50 text-zinc-600'
                      }`}
                    >
                      <input
                        type="radio"
                        name="gpu-mode"
                        className="h-4 w-4 accent-zinc-950"
                        checked={gpuCategory === option.value}
                        onChange={() => setGpuCategory(option.value)}
                      />
                      <span className="text-sm font-medium">{option.label}</span>
                    </label>
                  ))}
                </div>

                <p className="text-xs leading-5 text-zinc-400">{t(lang, 'gpuPickerNote')}</p>

                <div className="flex flex-col gap-3 sm:flex-row">
                  <button
                    className="primary-button flex-1"
                    onClick={async () => {
                      await updateConfigField('gpu_override', gpuCategory);
                      addLog(t(lang, 'logGpuSaved'));
                      setShowGpuPicker(false);
                    }}
                  >
                    {t(lang, 'apply')}
                  </button>
                  <button className="secondary-button flex-1" onClick={() => setShowGpuPicker(false)}>
                    {t(lang, 'cancel')}
                  </button>
                </div>
              </div>
            </motion.div>
          </ModalShell>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {showNvTutorial && (
          <ModalShell onClose={() => setShowNvTutorial(null)}>
            <motion.div
              initial={{ scale: 0.96, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.96, opacity: 0 }}
              className="panel w-full max-w-lg p-6"
            >
              <div className="space-y-4">
                <div className="space-y-1">
                  <p className="section-caption">{t(lang, 'stretchTitle')}</p>
                  <h2 className="text-2xl font-semibold tracking-tight text-zinc-950">
                    {t(lang, 'nvidiaTutorialTitle')}
                  </h2>
                  <p className="text-sm leading-6 text-zinc-500">
                    {t(lang, 'nvidiaTutorialSubtitle', { w: showNvTutorial.w, h: showNvTutorial.h })}
                  </p>
                </div>

                <pre className="rounded-2xl border border-zinc-200/70 bg-zinc-50 p-4 text-sm leading-6 text-zinc-600 whitespace-pre-wrap">
                  {t(lang, 'nvidiaTutorialSteps', { w: showNvTutorial.w, h: showNvTutorial.h })}
                </pre>

                <div className="flex flex-col gap-3 sm:flex-row">
                  <button
                    className="primary-button flex-1"
                    onClick={async () => {
                      try {
                        const [, msg] = await openNvidiaCp();
                        addLog(msg);
                      } catch (error) {
                        addLog(`${t(lang, 'error')}: ${String(error)}`);
                      }
                      setShowNvTutorial(null);
                    }}
                  >
                    {t(lang, 'openNvidiaCp')}
                  </button>
                  <button className="secondary-button flex-1" onClick={() => setShowNvTutorial(null)}>
                    {t(lang, 'close')}
                  </button>
                </div>
              </div>
            </motion.div>
          </ModalShell>
        )}
      </AnimatePresence>
    </div>
  );
}
