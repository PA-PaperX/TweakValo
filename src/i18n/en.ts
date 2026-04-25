export const en = {
  appName: 'TweakValo',
  appTag: 'VALORANT utility',
  appSubtitle: 'Minimalist performance tuning and stretch utility for VALORANT',
  developedBy: 'Developed by PaperX',

  safeMode: 'Safe Mode',
  safeModeDesc: 'Performance optimizations that are 100% safe and do not modify game files.',
  advMode: 'Advanced Mode',
  advModeDesc: 'High-risk features that modify game behavior. May trigger Vanguard bans. Use with extreme caution.',

  loading: 'Loading...',
  warningTitle: 'Use at your own risk',
  warningBody: 'Review each setting before launching. This app can touch local game files and NVIDIA settings.',
  disclaimer: 'Use carefully. You are responsible for the changes made on your machine while this tool is running.',

  gpuButton: 'GPU mode',
  summaryGame: 'Game path',
  summaryRiot: 'Riot Client',
  summaryGpu: 'Detected GPU',
  detecting: 'Detecting...',

  gameFolderTitle: 'Game folder',
  gameFolderDesc: 'Choose the VALORANT installation folder. Leave it blank if the default path is already correct.',
  gameFolderPlaceholder: 'C:\\Riot Games\\VALORANT',
  browse: 'Browse',
  pathReadyCustom: 'Custom path ready',
  pathReadyDefault: 'Default path detected',
  pathMissing: 'Path not found',

  contentTitle: 'Content options',
  contentDesc: 'Keep only the options you actually use.',
  matureContent: 'Mature Content (Effect Blood)',
  matureContentDesc: 'Add mature content patches before launching the game',
  riskVeryLow: 'Risk Level: Very Low. Only adds/restores external files, no patches to game engine.',
  riskHigh: 'Risk Level: Low. This is not a patch, but replaces the pak file. Backups are restored upon exiting. However, Vanguard Anti-Cheat bans are still possible.',

  launchTitle: 'Launch',
  launchDesc: 'Start VALORANT with the options above. Linked tools will run automatically before play.',
  play: 'Play VALORANT',
  launching: 'Launching...',
  launchManualBadge: 'Manual launch only',
  stretchLinkedBadge: 'Stretch linked to Play',
  nvidiaLinkedBadge: 'Graphics preset linked to Play',
  riotClientTitle: 'Riot Client',
  riotFound: 'Ready',
  riotMissing: 'Needs setup',
  riotBrowse: 'Locate client',

  stretchTitle: 'True Stretch',
  stretchDesc: 'Pick a preset, then apply it directly or link it to the main Play button.',
  resolution: 'Resolution',
  customRes: 'Custom resolution',
  customWidth: 'Width',
  customHeight: 'Height',
  applyStretch: 'Apply stretch',
  revertStretch: 'Revert',
  stretchHint: 'Updates VALORANT config files and stores a backup so you can roll back quickly.',
  riskNoneConfig: 'Risk level: none. This changes only local config files.',

  nvidiaTitle: 'Graphics profile',
  nvidiaDesc: 'Apply an NVIDIA profile with your preferred LOD bias preset.',
  lodPreset: 'Preset',
  presetLow: 'Low',
  presetMedium: 'Medium',
  presetHigh: 'High',
  presetCompetitive: 'Potato',
  applyNvidia: 'Apply NVIDIA profile',
  nvidiaHint: 'Applies Transparency Supersampling and LOD Bias to the VALORANT driver profile.',
  riskNoneDriver: 'Risk level: none. This changes only NVIDIA driver settings.',
  nvidiaDisabled: 'These controls are only available when an NVIDIA GPU mode is selected.',

  linkToLaunch: 'Link to Play',
  unlinkFromLaunch: 'Remove from Play',
  statusLinked: 'Linked',
  statusManual: 'Manual',

  logTitle: 'Activity log',
  logDesc: 'Recent actions and launch messages will appear here.',
  logEmpty: 'No activity yet',

  welcomeEyebrow: 'Welcome',
  welcomeTitle: 'Announcement',
  welcomeBody: 'TweakValo was created to help you apply True Stretch resolutions for wider player models, and tweak NVIDIA Profiles to lower graphics quality for the smoothest gameplay—all in one clean window.',
  welcomeFollow: 'Follow',
  welcomeAcceptRisk: 'I understand the risks and accept full responsibility for any changes made.',
  welcomeClose: 'Start using the app',

  gpuPickerTitle: 'GPU override',
  gpuPickerSubtitle: 'Select the mode that best matches your machine.',
  gpuModeAuto: 'Auto detect',
  gpuModeNvidia: 'NVIDIA only',
  gpuModeHybrid: 'Hybrid NVIDIA + Intel/AMD',
  gpuModeNonNvidia: 'Intel/AMD only',
  gpuPickerNote: 'This override is saved and used again on the next app start.',

  nvidiaTutorialTitle: 'Create a custom resolution first',
  nvidiaTutorialSubtitle: 'Resolution {w}x{h} is not available in your current display mode list.',
  nvidiaTutorialSteps: `One-time setup in NVIDIA Control Panel:

1. Open NVIDIA Control Panel.
2. Go to Display > Change resolution.
3. Click Customize.
4. Enable resolutions not exposed by the display.
5. Create a custom resolution.
6. Set Horizontal pixels to {w}.
7. Set Vertical lines to {h}.
8. Test and save the resolution.
9. Return here and apply stretch again.`,
  openNvidiaCp: 'Open NVIDIA Control Panel',
  close: 'Close',

  apply: 'Apply',
  cancel: 'Cancel',
  error: 'Error',
  startSeq: 'Starting launch sequence...',
  done: 'Done',
  logStretchApplying: 'Applying stretch preset...',
  logStretchReverting: 'Reverting stretch settings...',
  logNvidiaApplying: 'Applying NVIDIA profile...',
  logStretchLinked: 'Stretch linked to the Play button',
  logStretchUnlinked: 'Stretch removed from the Play button',
  logNvidiaLinked: 'NVIDIA profile linked to the Play button',
  logNvidiaUnlinked: 'NVIDIA profile removed from the Play button',
  logGpuSaved: 'GPU override saved',

  systemTitle: 'System & Memory',
  systemDesc: 'Automate Windows system management to reduce latency and in-game micro-stutters.',
  timerTitle: 'Enable 0.5ms Timer',
  timerDesc: 'Reduces Windows input latency for sharper and more responsive gameplay (Timer Resolution).',
  standbyTitle: 'Purge Standby Memory',
  standbyDesc: 'Clears system RAM cache upon game launch to prevent micro-stutters.',
  priorityTitle: 'Force High Priority',
  priorityDesc: 'Set VALORANT process priority to High to reduce input lag and CPU micro-stutters.',
  affinityTitle: 'Optimize CPU Affinity',
  affinityDesc: 'Forces VALORANT to run on strictly optimized physical cores (avoids Core 0 OS interrupts).',
  logSettingTimer: 'Setting Timer Resolution to 0.5ms...',
  logPurgingMemory: 'Purging Standby Memory cache...',
};
