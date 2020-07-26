use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Notification, NotificationOptions, NotificationPermission};

#[derive(Clone)]
pub(crate) struct Notifications {
    avatar: Option<Url>,
    displayname: String,
    content: String,
}

impl Notifications {
    pub fn new(avatar: Option<Url>, displayname: String, content: String) -> Self {
        Notifications {
            avatar,
            displayname,
            content,
        }
    }

    fn notifications_allowed(&self) -> bool {
        match Notification::permission() {
            NotificationPermission::Granted => true,
            _ => false,
        }
    }

    pub fn show(&self) {
        if !self.notifications_allowed() {
            let self_clone = self.clone();
            let cb = Closure::wrap(Box::new(move || {
                if self_clone.notifications_allowed() {
                    self_clone.clone().show_actual();
                }
            }) as Box<dyn FnMut()>);

            if let Err(_e) = Notification::request_permission_with_permission_callback(
                cb.as_ref().unchecked_ref(),
            ) {
                // Noop to please clippy/rust compiler
            }
            cb.forget();
        } else {
            self.show_actual();
        }
    }

    fn show_actual(&self) {
        let mut options_0 = NotificationOptions::new() as NotificationOptions;
        let options_1 = options_0.body(&self.content).tag("daydream") as &mut NotificationOptions;
        let options = match self.clone().avatar {
            None => options_1,
            Some(avatar) => {
                let url = avatar.to_string();
                options_1.icon(&url)
            }
        };
        if let Err(_e) = Notification::new_with_options(&self.displayname, &options) {
            // Noop to please clippy/rust compiler
            // TODO check if we in this case should stop showing notifications
        }
    }
}
