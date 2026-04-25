mod config;
mod gpu_detect;
mod game_launcher;
mod stretch;
mod fps_optimizer;
mod graphics_preset;
mod nvidia_profile;
mod system_optimizer;
mod process_director;

use config::{load_config, ConfigState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_config = load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ConfigState(std::sync::Mutex::new(initial_config)))
        .invoke_handler(tauri::generate_handler![
            // Config
            config::get_config,
            config::update_config,
            config::update_config_field,
            // GPU
            gpu_detect::get_gpu_info,
            // Game Launcher
            game_launcher::check_riot_client,
            game_launcher::check_game_path,
            game_launcher::launch_game,
            game_launcher::check_game_running,
            // Stretch
            stretch::get_resolutions,
            stretch::apply_stretch,
            stretch::revert_stretch,
            stretch::open_nvidia_cp,
            // FPS
            fps_optimizer::run_fps_optimizations,
            // Graphics
            graphics_preset::apply_graphics_preset,
            graphics_preset::restore_graphics,
            // NVIDIA Profile
            nvidia_profile::apply_nvidia_profile,
            // System Optimizer (Timer & Memory)
            system_optimizer::set_timer_resolution,
            system_optimizer::purge_standby_list,
        ])
        .setup(|app| {
            process_director::spawn_director_thread();

            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Warn })
                    .build(),
            )?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
