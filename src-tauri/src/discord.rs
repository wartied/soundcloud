use discord_rich_presence::{
    activity::{Activity, ActivityType, Assets, Button, StatusDisplayType, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APP_ID: &str = "1501127642001313802";
const RECONNECT_SECS: u64 = 15;

#[derive(Clone, Default, PartialEq)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub artwork_url: String,
    pub track_url: String,
    pub is_playing: bool,
    pub elapsed_ms: i64,
    pub duration_ms: i64,
}

pub struct SharedState {
    info: TrackInfo,
    dirty: bool,
}

#[derive(Clone)]
pub struct DiscordState {
    inner: Arc<Mutex<SharedState>>,
}

impl DiscordState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SharedState {
                info: TrackInfo::default(),
                dirty: false,
            })),
        }
    }

    pub fn update(&self, new: TrackInfo) {
        if let Ok(mut state) = self.inner.lock() {
            if state.info != new {
                state.info = new;
                state.dirty = true;
            }
        }
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub fn spawn_rpc_thread(state: DiscordState) {
    thread::spawn(move || loop {
        let mut client = DiscordIpcClient::new(APP_ID);

        if client.connect().is_err() {
            thread::sleep(Duration::from_secs(RECONNECT_SECS));
            continue;
        }

        loop {
            thread::sleep(Duration::from_millis(250));

            let info = {
                let mut guard = match state.inner.lock() {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                if !guard.dirty {
                    continue;
                }
                guard.dirty = false;
                guard.info.clone()
            };

            if info.title.is_empty() || !info.is_playing {
                let _ = client.clear_activity();
                continue;
            }

            let now = now_ms();
            let start_ms = now - info.elapsed_ms;

            let mut timestamps = Timestamps::new().start(start_ms);
            if info.duration_ms > 0 {
                timestamps = timestamps.end(start_ms + info.duration_ms);
            }

            let has_url = !info.track_url.is_empty();
            let buttons_vec;

            let mut activity = Activity::new()
                .activity_type(ActivityType::Listening)
                .status_display_type(StatusDisplayType::Details)
                .details(&info.artist)
                .state(&info.title)
                .assets(
                    Assets::new()
                        .large_image(if info.artwork_url.is_empty() {
                            "soundcloud_logo"
                        } else {
                            &info.artwork_url
                        })
                        .small_image("soundcloud_logo")
                        .small_text("SoundCloud"),
                )
                .timestamps(timestamps);

            if has_url {
                buttons_vec = vec![Button::new("Listen on SoundCloud", &info.track_url)];
                activity = activity.buttons(buttons_vec);
            }

            if client.set_activity(activity).is_err() {
                let _ = client.close();
                break;
            }
        }
    });
}
