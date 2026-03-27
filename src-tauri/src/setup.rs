use my_mind_core::input::{FocusManager, InputSimulator};
use my_mind_tauri::commands::recording;
use my_mind_tauri::state::AppState;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tracing::{error, info, warn};

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Check accessibility permission at startup (macOS: shows system prompt if not granted)
    #[cfg(target_os = "macos")]
    {
        let trusted = InputSimulator::request_accessibility_permission();
        if trusted {
            info!("[setup] Accessibility permission granted");
        } else {
            warn!("[setup] Accessibility permission NOT granted — paste will not work until granted");
        }
    }

    // Setup system tray
    setup_tray(app)?;

    // Setup global shortcut
    setup_shortcut(app)?;

    // Create the recording overlay window (hidden by default)
    let _window = tauri::WebviewWindowBuilder::new(
        app,
        "overlay",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("My Mind")
    .inner_size(360.0, 180.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .center()
    .visible(false)
    .build()?;

    info!("App setup complete");
    Ok(())
}

fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "History", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit My Mind", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &history, &separator, &quit])?;

    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png"))?;

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(false)
        .menu(&menu)
        .tooltip("My Mind - Voice to Text")
        .on_menu_event(|app, event| {
            if event.id() == "settings" {
                info!("Settings clicked from tray menu");
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.set_focus();
                } else {
                    match tauri::WebviewWindowBuilder::new(
                        app,
                        "settings",
                        tauri::WebviewUrl::App("index.html".into()),
                    )
                    .title("My Mind - Settings")
                    .inner_size(640.0, 520.0)
                    .min_inner_size(480.0, 400.0)
                    .decorations(true)
                    .always_on_top(false)
                    .center()
                    .visible(true)
                    .build()
                    {
                        Ok(_) => info!("Settings window created"),
                        Err(e) => error!("Failed to create settings window: {}", e),
                    }
                }
            } else if event.id() == "history" {
                info!("History clicked from tray menu");
                if let Some(window) = app.get_webview_window("history") {
                    let _ = window.set_focus();
                } else {
                    match tauri::WebviewWindowBuilder::new(
                        app,
                        "history",
                        tauri::WebviewUrl::App("index.html".into()),
                    )
                    .title("My Mind - History")
                    .inner_size(700.0, 500.0)
                    .min_inner_size(500.0, 400.0)
                    .decorations(true)
                    .always_on_top(false)
                    .center()
                    .visible(true)
                    .build()
                    {
                        Ok(_) => info!("History window created"),
                        Err(e) => error!("Failed to create history window: {}", e),
                    }
                }
            } else if event.id() == "quit" {
                info!("Quit from tray menu");
                app.exit(0);
            }
        })
        .build(app)?;

    info!("System tray initialized");
    Ok(())
}

fn setup_shortcut(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut: Shortcut = "Alt+Space".parse()?;

    app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
        let app_handle = app.clone();

        match event.state() {
            ShortcutState::Pressed => {
                info!("Shortcut pressed");

                // Capture the frontmost app ONLY on first press (before recording starts).
                // On second press (stop), previous_app already holds the correct target app.
                // previous_app is reset to None via .take() after pipeline completes.
                let state = app_handle.state::<AppState>();
                {
                    let current = state.previous_app.lock().unwrap();
                    if current.is_none() {
                        drop(current);
                        match FocusManager::get_frontmost_app() {
                            Ok(Some(bundle_id)) => {
                                info!("[focus] Captured frontmost app: {}", bundle_id);
                                *state.previous_app.lock().unwrap() = Some(bundle_id);
                            }
                            Ok(None) => {
                                warn!("[focus] Could not determine frontmost app");
                            }
                            Err(e) => {
                                warn!("[focus] Error getting frontmost app: {}", e);
                            }
                        }
                    } else {
                        info!("[focus] previous_app already set, skipping capture");
                    }
                }

                // Show overlay window (no set_focus to avoid stealing focus from target app)
                if let Some(window) = app_handle.get_webview_window("overlay") {
                    let _ = window.show();
                }

                // Start recording
                let app_for_task = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_for_task.state::<AppState>();
                    let is_recording = *state.is_recording.lock().await;
                    if !is_recording {
                        // Check recording mode from config
                        let config = state.config.lock().await;
                        let mode = config.shortcuts.mode.clone();
                        drop(config);

                        if mode == "toggle" {
                            // Toggle mode: start on press
                            if let Err(e) = recording::start_recording_inner(&app_for_task).await {
                                error!("Failed to start recording: {}", e);
                            }
                        } else {
                            // Push-to-talk: start on press
                            if let Err(e) = recording::start_recording_inner(&app_for_task).await {
                                error!("Failed to start recording: {}", e);
                            }
                        }
                    } else if mode_is_toggle(&app_for_task).await {
                        // Toggle mode: stop on second press
                        let state = app_for_task.state::<AppState>();
                        if let Err(e) = recording::stop_recording_inner(&state).await {
                            error!("Failed to stop recording: {}", e);
                        }
                    }
                });
            }
            ShortcutState::Released => {
                // Push-to-talk: stop on release
                let app_for_task = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if !mode_is_toggle(&app_for_task).await {
                        let state = app_for_task.state::<AppState>();
                        let is_recording = *state.is_recording.lock().await;
                        if is_recording {
                            if let Err(e) = recording::stop_recording_inner(&state).await {
                                error!("Failed to stop recording: {}", e);
                            }
                        }
                    }
                });
            }
        }
    })?;

    info!("Global shortcut registered: Alt+Space");
    Ok(())
}

async fn mode_is_toggle(app: &tauri::AppHandle) -> bool {
    let state = app.state::<AppState>();
    let config = state.config.lock().await;
    config.shortcuts.mode == "toggle"
}
