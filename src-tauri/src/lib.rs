mod adblock;
mod discord;

use discord::TrackInfo;
use serde::Deserialize;
use tauri::Manager;
use tauri::webview::PageLoadEvent;
use tauri::WebviewWindowBuilder;

use std::time::Duration;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrackData {
    title: String,
    artist: String,
    artwork_url: String,
    track_url: String,
    is_playing: bool,
    #[serde(default)]
    elapsed_ms: i64,
    #[serde(default)]
    duration_ms: i64,
}

impl TrackData {
    fn into_info(self) -> TrackInfo {
        TrackInfo {
            title: self.title,
            artist: self.artist,
            artwork_url: self.artwork_url,
            track_url: self.track_url,
            is_playing: self.is_playing,
            elapsed_ms: self.elapsed_ms,
            duration_ms: self.duration_ms,
        }
    }
}

pub fn run() {
    let adblock_js = adblock::get_adblock_script();
    let scraper_js = adblock::get_track_scraper_js();
    let zoom_js = adblock::get_zoom_js();
    let zoom_poll_js = adblock::get_zoom_poll_js();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(discord::DiscordState::new())
        .setup(move |app| {
            let state = app.state::<discord::DiscordState>().inner().clone();
            discord::spawn_rpc_thread(state);

            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon-32.png"))?;

            let data_dir = std::path::PathBuf::from(
                std::env::var("LOCALAPPDATA").unwrap_or_default(),
            )
            .join("Soundcloud");

            let webview = WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External("https://soundcloud.com".parse().unwrap()),
            )
            .title("SoundCloud")
            .inner_size(1280.0, 800.0)
            .min_inner_size(900.0, 600.0)
            .resizable(true)
            .decorations(true)
            .icon(icon.clone())?
            .data_directory(data_dir)
            .on_page_load(move |webview, payload| {
                if payload.event() == PageLoadEvent::Finished {
                    let _ = webview.eval(adblock_js);
                    let _ = webview.eval(zoom_js);
                }
            })
            .build()?;

            let ds = app.state::<discord::DiscordState>().inner().clone();
            let wv = webview.clone();
            let wv_zoom = webview.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(500));
                let ds2 = ds.clone();
                let _ = wv.eval_with_callback(scraper_js, move |result| {
                    if let Ok(data) = serde_json::from_str::<TrackData>(&result) {
                        if !data.title.is_empty() {
                            ds2.update(data.into_info());
                        }
                    }
                });
                let wvz = wv_zoom.clone();
                let _ = wv_zoom.eval_with_callback(zoom_poll_js, move |result| {
                    if let Ok(factor) = result.parse::<f64>() {
                        let _ = wvz.set_zoom(factor);
                    }
                });
            });

            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::TrayIconBuilder;

            let show = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build app");

    let ds_exit = app.state::<discord::DiscordState>().inner().clone();
    app.run(move |_app, event| {
        if let tauri::RunEvent::ExitRequested { .. } = event {
            ds_exit.update(TrackInfo::default());
            std::thread::sleep(Duration::from_millis(300));
        }
    });
}