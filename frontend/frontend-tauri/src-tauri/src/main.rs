#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let cache_dir = app.path_resolver().app_cache_dir().unwrap();
            fs_extra::dir::create_all(cache_dir.join("core-options"), false)?;
            fs_extra::dir::create_all(cache_dir.join("content"), false)?;
            fs_extra::dir::create_all(cache_dir.join("saves"), true)?;
            fs_extra::dir::create_all(cache_dir.join("states"), true)?;
            fs_extra::dir::create_all(cache_dir.join("cache"), false)?;
            fs_extra::dir::create_all(cache_dir.join("screenshots"), false)?;

            let config_dir = app.path_resolver().app_config_dir().unwrap();
            fs_extra::dir::create_all(config_dir.join("remaps"), false)?;

            let resource_dir = app.path_resolver().resource_dir().unwrap();
            //let tmp_dir = std::env::temp_dir().join("gisst");

            // copy files where they need to go
            let ra_cfg_path = app.path_resolver().resolve_resource("ra-config-base.cfg");
            let ra_cfg = std::fs::read_to_string(ra_cfg_path.unwrap())?;
            let ra_cfg = ra_cfg.replace("$RESOURCE", &resource_dir.to_string_lossy());
            let ra_cfg = ra_cfg.replace("$CONFIG", &config_dir.to_string_lossy());
            let ra_cfg = ra_cfg.replace("$CACHE", &cache_dir.to_string_lossy());
            //ra_cfg.replace("$TMP", tmp_dir.to_string_lossy());
            std::fs::write(cache_dir.join("retroarch.cfg"), ra_cfg)?;
            // for demo only: copy content dir to cache dir
            fs_extra::dir::copy(
                resource_dir.join("content"),
                cache_dir,
                &fs_extra::dir::CopyOptions::new().overwrite(true),
            )?;

            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                use tauri::Manager;
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
