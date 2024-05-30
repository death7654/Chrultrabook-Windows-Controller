pub async fn new_window(
    handle: &tauri::AppHandle,
    label: &str,
    angular_path: &str, //same as routing path in app.routes.ts
    width: f64,
    height: f64,
) -> tauri::WebviewWindow<tauri::Wry> {
    return tauri::WebviewWindowBuilder::new(
        handle,
        label, /* the unique window label */
        tauri::WebviewUrl::App(angular_path.parse().unwrap()),
    )
    .inner_size(width, height)
    .resizable(false)
    .maximizable(false)
    .build()
    .unwrap();
}
