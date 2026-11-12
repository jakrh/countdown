#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            // Hides the app from the Dock and cmd + tab. Must be set here, not via
            // Info.plist's `LSUIElement`: tao re-applies the activation policy in
            // `applicationDidFinishLaunching` and discards whatever the plist set.
            #[cfg(target_os = "macos")]
            _app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
