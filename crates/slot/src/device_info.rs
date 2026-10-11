//! What the app is running on, for the About sticker: unset on the handhelds (the sticker then
//! keeps the original console's model number), the phone's `Build.MODEL` on Android.
use std::sync::OnceLock;

static MODEL: OnceLock<String> = OnceLock::new();
static MAKER: OnceLock<String> = OnceLock::new();

pub fn set_model(model: &str) {
    let model: String = model.trim().chars().take(18).collect();
    if !model.is_empty() {
        let _ = MODEL.set(model);
    }
}

pub fn model() -> Option<&'static str> {
    MODEL.get().map(String::as_str)
}

pub fn set_maker(maker: &str) {
    let maker: String = maker.trim().chars().take(18).collect();
    if !maker.is_empty() {
        let _ = MAKER.set(maker);
    }
}

pub fn maker() -> Option<&'static str> {
    MAKER.get().map(String::as_str)
}
