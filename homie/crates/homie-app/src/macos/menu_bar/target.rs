use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSApplication, NSButton, NSPanel, NSStatusBarButton};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSPoint};

use homie_proto::SessionId;

use super::POPUP_WIDTH;
use crate::store::SessionStore;

pub(super) struct MenuBarTargetIvars {
    panel: Retained<NSPanel>,
    button: Retained<NSStatusBarButton>,
    store: Arc<RwLock<SessionStore>>,
    session_ids: RefCell<Vec<SessionId>>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements. AppKit invokes these
    // control actions on the main thread and the class does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = MenuBarTargetIvars]
    pub(super) struct MenuBarTarget;

    unsafe impl NSObjectProtocol for MenuBarTarget {}

    impl MenuBarTarget {
        #[unsafe(method(toggleHomieMenu:))]
        fn toggle_menu(&self, _sender: Option<&AnyObject>) {
            let panel = &self.ivars().panel;
            if panel.isVisible() {
                panel.orderOut(None);
                return;
            }

            // Updates are skipped while the panel is hidden; ask for one
            // fresh snapshot so the first visible frame shows current rows.
            self.ivars()
                .store
                .write()
                .expect("session store lock poisoned")
                .request_snapshot_publish();

            if let Some(window) = self.ivars().button.window() {
                let button_rect = self
                    .ivars()
                    .button
                    .convertRect_toView(self.ivars().button.bounds(), None);
                let screen_rect = window.convertRectToScreen(button_rect);
                panel.setFrameTopLeftPoint(NSPoint::new(
                    screen_rect.origin.x + (screen_rect.size.width - POPUP_WIDTH) / 2.0,
                    screen_rect.origin.y - 4.0,
                ));
            }
            panel.orderFront(None);
        }

        #[unsafe(method(openHomie:))]
        fn open_homie(&self, _sender: Option<&AnyObject>) {
            self.show_main_window();
        }

        #[unsafe(method(selectSession:))]
        fn select_session(&self, sender: Option<&AnyObject>) {
            if let Some(tag) = sender
                .and_then(|sender| sender.downcast_ref::<NSButton>())
                .map(|button| button.tag())
                && let Some(id) = self.ivars().session_ids.borrow().get(tag as usize).cloned()
            {
                self.ivars()
                    .store
                    .write()
                    .expect("session store lock poisoned")
                    .select(id);
            }
            self.show_main_window();
        }

        #[unsafe(method(quitHomie:))]
        fn quit_homie(&self, _sender: Option<&AnyObject>) {
            NSApplication::sharedApplication(self.mtm()).terminate(None);
        }
    }
);

impl MenuBarTarget {
    pub(super) fn new(
        mtm: MainThreadMarker,
        panel: Retained<NSPanel>,
        button: Retained<NSStatusBarButton>,
        store: Arc<RwLock<SessionStore>>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(MenuBarTargetIvars {
            panel,
            button,
            store,
            session_ids: RefCell::new(Vec::new()),
        });
        // SAFETY: NSObject's init is its designated initializer.
        unsafe { msg_send![super(this), init] }
    }

    pub(super) fn set_session_ids(&self, ids: Vec<SessionId>) {
        self.ivars().session_ids.replace(ids);
    }

    pub(super) fn show_main_window(&self) {
        self.ivars().panel.orderOut(None);
        let app = NSApplication::sharedApplication(self.mtm());
        app.activate();
        for window in app.windows().iter() {
            if window.canBecomeMainWindow() {
                window.makeKeyAndOrderFront(None);
                break;
            }
        }
    }
}
