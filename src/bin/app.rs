#![recursion_limit = "1024"]
extern crate console_error_panic_hook;
//extern crate wee_alloc;
use console_error_panic_hook::set_once as set_panic_hook;
use i18n_embed::{language_loader, I18nEmbed, WebLanguageRequester};
use rust_embed::RustEmbed;

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"] // path to the compiled localization resources
struct Translations;

language_loader!(DaydreamLanguageLoader);

//#[global_allocator]
//static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn main() {
    // If the `console_error_panic_hook` feature is enabled this will set a panic hook, otherwise
    // it will do nothing.
    set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());

    let translations = Translations {};
    let language_loader = DaydreamLanguageLoader::new();
    let requested_languages = WebLanguageRequester::requested_languages();

    i18n_embed::select(&language_loader, &translations, &requested_languages).unwrap();

    yew::start_app::<daydream::app::App>();
}
