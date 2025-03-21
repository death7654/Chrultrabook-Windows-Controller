// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod activity_light;
mod custom_fan;
mod execute;
mod get_all_windows;
mod helper;
mod open_window;
mod save_to_files;
mod save_to_local;
mod temps;

//external crates
use open;
use tauri::image::Image;
use tauri::menu::{IconMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, EventTarget, Manager};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_clipboard_manager::ClipboardExt;

//open windows
#[tauri::command]
async fn open_window(
    handle: tauri::AppHandle,
    window: tauri::Window,
    name: &str,
    width: f64,
    height: f64,
) -> Result<(), ()> {
    let window_open = get_all_windows::window(&window, name);
    if window_open == true {
        window.get_webview_window(name).unwrap().show().unwrap();
    } else {
        open_window::new_window(&handle, name, name, width, height, true).await;
    }
    Ok(())
}

//commands
#[tauri::command]
fn execute(handle: tauri::AppHandle, program: &str, arguments: Vec<String>, reply: bool) -> String {
    execute::execute_relay(handle, &program, arguments, reply)
}
#[tauri::command]
fn diagnostics(handle: tauri::AppHandle, selected: &str) -> String {
    let output;
    match selected {
        "Boot Timestraps" => {
            output = execute(handle, "cbmem", helper::to_vec_string(vec!["-t"]), true)
        }
        "Coreboot Log" => {
            output = execute(handle, "cbmem", helper::to_vec_string(vec!["-c1"]), true)
        }
        "Coreboot Extended Log" => {
            output = execute(handle, "cbmem", helper::to_vec_string(vec!["-c"]), true)
        }
        "EC Console Log" => {
            output = execute(
                handle,
                "ectool",
                helper::to_vec_string(vec!["console"]),
                true,
            )
        }
        "Battery Information" => {
            output = execute(
                handle,
                "ectool",
                helper::to_vec_string(vec!["battery"]),
                true,
            )
        }
        "EC Chip Information" => {
            output = execute(
                handle,
                "ectool",
                helper::to_vec_string(vec!["chipinfo"]),
                true,
            )
        }
        "SPI Information" => {
            output = execute(
                handle,
                "ectool",
                helper::to_vec_string(vec!["flashspiinfo"]),
                true,
            )
        }
        "EC Protocol Information" => {
            output = execute(
                handle,
                "ectool",
                helper::to_vec_string(vec!["protoinfo"]),
                true,
            )
        }
        "Temperature Sensor Information" => {
            output = execute(
                handle,
                "ectool",
                helper::to_vec_string(vec!["tempsinfo", "all"]),
                true,
            )
        }
        "Power Delivery Information" => {
            output = execute(handle, "ectool", helper::to_vec_string(vec!["pdlog"]), true)
        }
        _ => output = "Select An Option".to_string(),
    }
    return output.trim().to_string();
}

#[tauri::command]
fn copy(handle: tauri::AppHandle, text: String) {
    handle.clipboard().write_text(text).unwrap();
}
#[tauri::command]
fn save(app: tauri::AppHandle, filename: String, content: String) {
    save_to_files::select_path(app, filename, content);
}

#[tauri::command]
fn local_storage(function: &str, option: &str, value: &str) -> String {
    return save_to_local::local_storage(function, option, value);
}

#[tauri::command]
fn get_temps(handle: tauri::AppHandle) -> u16 {
    let temps = execute(
        handle,
        "ectool",
        helper::to_vec_string(vec!["temps", "all"]),
        true,
    );
    return temps::get_temp(temps);
}

#[tauri::command]
fn boardname(handle: tauri::AppHandle) -> String {
    #[cfg(windows)]
    {
        return execute::execute_relay(
            handle,
            "wmic",
            helper::to_vec_string(vec!["baseboard", "get", "Product"]),
            true,
        )
        .trim()
        .split("\n")
        .map(|x| x.to_string())
        .collect::<Vec<String>>()[2]
            .clone();
    }
    #[cfg(target_os = "linux")]
    {
        return execute::execute_relay(
            handle,
            "cat",
            vec!["/sys/class/dmi/id/product_name".to_string()],
            true,
        )
        .trim()
        .to_string();
    }

    #[cfg(target_os = "macos")]
    {
        return String::from("unknown");
    }
}
#[tauri::command]
fn os() -> String {
    #[cfg(target_os = "macos")]
    {
        return String::from("macOS");
    }
    return String::from("not macOS");
}

#[tauri::command]
fn change_activity_light(selected: String) {
    activity_light::set_color(selected);
}

#[tauri::command]
fn autostart(app: AppHandle, value: bool) {
    let autolaunch = app.autolaunch();

    let enabled_state = autolaunch.is_enabled().unwrap_or(false);
    if value || !enabled_state {
        let _ = autolaunch.enable();
    } else if !value || enabled_state {
        let _ = autolaunch.disable();
    }
}
#[tauri::command]
async fn get_json() -> String {
    let output = local_storage("get", "profiles", "");
    let default_array = String::from("[{\"id\":0,\"name\":\"Default\",\"array\":[0,10,25,40,60,80,95,100,100,100,100,100,100],\"selected\":true,\"disabled\":true,\"class\":\"transparent\",\"img_class\":\"btn-outline-info\",\"img\":\"\\uF4CB\"},{\"id\":1,\"name\":\"Aggressive\",\"array\":[0,10,40,50,60,90,100,100,100,100,100,100,100],\"selected\":false,\"disabled\":true,\"class\":\"transparent\",\"img_class\":\"btn-outline-info\",\"img\":\"\\uF4CB\"},{\"id\":2,\"name\":\"Quiet\",\"array\":[0,15,20,30,40,55,90,100,100,100,100,100,100],\"selected\":false,\"disabled\":true,\"class\":\"transparent\",\"img_class\":\"btn-outline-info\",\"img\":\"\\uF4CB\"}]");
    if output.contains("[") == false {
        return default_array;
    } else {
        return output;
    }
}

#[tauri::command]
fn set_custom_fan(handle: tauri::AppHandle, temp: i16, array: Vec<i8>) {
    let fan_speed = custom_fan::calculate_fan_percentage(temp, array).to_string();
    execute::execute_relay(
        handle,
        "ectool",
        helper::to_vec_string(vec!["fanduty", &fan_speed]),
        false,
    );
}

#[tauri::command]
fn transfer_fan_curves(app: AppHandle, curves: String) {
    app.emit_to(EventTarget::webview_window("main"), "fan_curve", &curves)
        .expect("failure to transmit data");
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            //to hide app if user wants it hidden upon boot
            let start_app_in_tray = local_storage("get", "start_app_tray", " ");
            if start_app_in_tray == "true" {
                let window = app.get_webview_window("main").unwrap();
                window.hide().unwrap();
            }

            let img = IconMenuItemBuilder::new("Chrultrabook Tools")
                .id("app")
                .enabled(false)
                .icon(Image::from_bytes(include_bytes!("../icons/icon.png")).unwrap())
                .build(app)
                .unwrap();
            let show = MenuItemBuilder::new("Show").id("show").build(app).unwrap();
            let hide = MenuItemBuilder::new("Hide").id("hide").build(app).unwrap();
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app).unwrap();
            let github = MenuItemBuilder::new("Check for Updates")
                .id("github")
                .build(app)
                .unwrap();

            // we could opt handle an error case better than calling unwrap
            let menu = MenuBuilder::new(app)
                .item(&img)
                .separator()
                .items(&[&show, &hide, &quit])
                .separator()
                .item(&github)
                .build()
                .unwrap();

            let _ = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "quit" => app.exit(0),
                    "hide" => {
                        let window = app.webview_windows();
                        for (_, window) in window.iter() {
                            let _ = window.hide();
                        }
                    }
                    "show" => {
                        let window = app.webview_windows();
                        for (_, window) in window.iter() {
                            let _ = window.show();
                        }
                    }
                    "github" => {
                        let _ =
                            open::that("https://github.com/death7654/Chrultrabook-Tools/releases");
                    }
                    _ => {}
                })
                .tooltip("Chrultrabook Tools")
                .build(app);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let close_app_to_tray = local_storage("get", "app_tray", " ");
                if close_app_to_tray == "true" {
                    if window.label() == "main" {
                        let windows = window.webview_windows();
                        for (_, window) in windows.iter() {
                            let window_name = window.label();
                            if window_name == "main" {
                                window.hide().unwrap()
                            } else {
                                window
                                    .get_webview_window(window_name)
                                    .expect("notfound")
                                    .close()
                                    .unwrap()
                            }
                        }
                        api.prevent_close();
                    } else {
                        let _ = window.close().unwrap();
                    }
                } else {
                    if window.label() == "main" {
                        let windows = window.webview_windows();
                        for (_, window) in windows.iter() {
                            let window_name = window.label();
                            window
                                .get_webview_window(window_name)
                                .expect("notfound")
                                .close()
                                .unwrap()
                        }
                    } else {
                        let _ = window.close();
                    }
                }
            }
            _ => {}
        })
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--auto"]),
        ))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            open_window,
            execute,
            diagnostics,
            copy,
            save,
            local_storage,
            get_temps,
            boardname,
            os,
            change_activity_light,
            autostart,
            get_json,
            set_custom_fan,
            transfer_fan_curves
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
