import { invoke } from '@tauri-apps/api/core';

export interface AppConfig {
  game_path: string;
  riot_client_path: string;
  enable_blood: boolean;
  minimize_to_tray: boolean;
  language: string;
  custom_width: string;
  custom_height: string;
  last_resolution: string;
  gpu_override: string;
  welcome_shown: boolean;
  auto_timer_resolution: boolean;
  auto_standby_cleaner: boolean;
  auto_priority: boolean;
  auto_affinity: boolean;
}

export const getConfig = () => invoke<AppConfig>('get_config');
export const updateConfig = (config: AppConfig) => invoke<void>('update_config', { config });
export const updateConfigField = (key: string, value: string) =>
  invoke<void>('update_config_field', { key, value });

export interface GpuInfo {
  names: string[];
  nvidia: boolean;
  intel: boolean;
  amd: boolean;
  category: string;
}

export const getGpuInfo = () => invoke<GpuInfo>('get_gpu_info');

export interface LaunchResult {
  success: boolean;
  message: string;
}

export const checkRiotClient = (customPath: string) =>
  invoke<LaunchResult>('check_riot_client', { customPath });
export const checkGamePath = (gamePath: string) =>
  invoke<LaunchResult>('check_game_path', { gamePath });
export const launchGame = (gamePath: string, riotClientPath: string, enableBlood: boolean) =>
  invoke<string[]>('launch_game', {
    gamePath,
    riotClientPath,
    enableBlood,
  });
export const checkGameRunning = () => invoke<boolean>('check_game_running');

export const getResolutions = () => invoke<[string, string, string][]>('get_resolutions');

export interface StretchResult {
  success: boolean;
  needs_custom_res: boolean;
  native_w: number;
  native_h: number;
  message: string;
}

export const applyStretch = (resolutionKey: string, customW: number, customH: number) =>
  invoke<StretchResult>('apply_stretch', { resolutionKey, customW, customH });
export const revertStretch = (nativeW: number, nativeH: number) =>
  invoke<StretchResult>('revert_stretch', { nativeW, nativeH });
export const openNvidiaCp = () => invoke<[boolean, string]>('open_nvidia_cp');

export interface OptResult {
  name: string;
  success: boolean;
  message: string;
}

export const runFpsOptimizations = () => invoke<OptResult[]>('run_fps_optimizations');

export const applyGraphicsPreset = (presetFile: string) =>
  invoke<[boolean, string]>('apply_graphics_preset', { presetFile });
export const restoreGraphics = () => invoke<[boolean, string]>('restore_graphics');

export const applyNvidiaProfile = (lodKey: string) =>
  invoke<[boolean, string]>('apply_nvidia_profile', { lodKey });

export async function setTimerResolution(enable: boolean) {
  try {
    const res = await invoke<number>('set_timer_resolution', { enable });
    return { success: true, message: enable ? `Timer resolution applied (${(res / 10000).toFixed(4)} ms)` : 'Timer resolution reverted' };
  } catch (error) {
    return { success: false, message: String(error) };
  }
}

export async function purgeStandbyList() {
  try {
    await invoke<void>('purge_standby_list');
    return { success: true, message: 'Standby memory purged successfully' };
  } catch (error) {
    return { success: false, message: String(error) };
  }
}
