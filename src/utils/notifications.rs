use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Notification, NotificationOptions, NotificationPermission};

#[derive(Clone)]
pub(crate) struct Notifications {
    avatar: Option<String>,
    displayname: String,
    content: String,
}

impl Notifications {
    pub fn new(avatar: Option<String>, displayname: String, content: String) -> Self {
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

            Notification::request_permission_with_permission_callback(cb.as_ref().unchecked_ref());
            cb.forget();
        } else {
            self.show_actual();
        }
    }

    fn show_actual(&self) {
        let mut options = NotificationOptions::new() as NotificationOptions;
        let options = options.body(&self.content).tag("daydream") as &mut NotificationOptions;
        let options = if self.avatar.is_some() {
            options.icon(self.avatar.as_ref().unwrap())
        } else {
            options
        };
        Notification::new_with_options(&self.displayname, &options);
    }
}
