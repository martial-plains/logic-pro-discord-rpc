use std::cell::OnceCell;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{Ivars, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApp, NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBeep,
    NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{
    NSBundle, NSFileManager, NSHomeDirectory, NSNotification, NSObject, NSObjectProtocol, NSString,
    NSUTF8StringEncoding, ns_string,
};

use crate::state;

const ORG_BUNDLE_ID: &str = "com.isaiah-harvey.logicrpc";

define_class!(
    // SAFETY:
    // - The superclass NSObject does not have any subclassing requirements.
    // - `AppDelegate` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    pub struct AppDelegate {
        status_item: OnceCell<Retained<NSStatusItem>>,
        start_at_login_item: OnceCell<Retained<NSMenuItem>>,
    }

    // SAFETY: No problematic methods on `NSObjectProtocol` are implemented.
    unsafe impl NSObjectProtocol for AppDelegate {}

    // SAFETY: `NSApplicationDelegate` has no safety requirements.
    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn did_finish_launching(&self, notification: &NSNotification) {
            let mtm = self.mtm();
            let app = notification
                .object()
                .unwrap()
                .downcast::<NSApplication>()
                .unwrap();

            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
            self.status_item()
                .set(
                    NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength),
                )
                .unwrap();

            if let Some(button) = self.status_item().get().unwrap().button(mtm) {
                button.setTitle(ns_string!("lgrp"));
                button.setToolTip(Some(ns_string!("Logic Pro RPC")))
            }



            let menu = NSMenu::init(NSMenu::alloc(mtm));

            self.start_at_login_item()
                .set(unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        ns_string!("Start at Login"),
                        Some(sel!(toggleStartAtLogin)),
                        ns_string!(""),
                    )
                })
                .unwrap();
            unsafe {
                self.start_at_login_item()
                    .get()
                    .unwrap()
                    .setTarget(Some(self));
            }

            self.start_at_login_item()
                .get()
                .unwrap()
                .setState(if is_start_at_login_enabled() {
                    NSControlStateValueOn
                } else {
                    NSControlStateValueOff
                });
            menu.addItem(self.start_at_login_item().get().unwrap());

            menu.addItem(&NSMenuItem::separatorItem(mtm));

            let quit_item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    ns_string!("Quit"),
                    Some(sel!(quit)),
                    ns_string!("q"),
                )
            };

            unsafe {
                quit_item.setTarget(Some(self));
                quit_item.setEnabled(true);
            }
            menu.addItem(&quit_item);
            self.status_item().get().unwrap().setMenu(Some(&menu));

            state::start_idle();
        }

        // SAFETY: The signature is correct.
        #[unsafe(method(applicationWillTerminate:))]
        fn will_terminate(&self, _notification: &NSNotification) {

        }


    }

    impl AppDelegate {
        #[unsafe(method(quit))]
        fn quit(&self){
            let app = self.mtm();
            NSApp(app).terminate(None);
        }

        #[unsafe(method(toggleStartAtLogin))]
        fn toggle_start_at_login(&self) {
            if self.start_at_login_item().get().unwrap().state() == NSControlStateValueOn {
                disable_start_login();
                self.start_at_login_item().get().unwrap().setState(NSControlStateValueOff);
            } else {
                let ok = enable_start_login();
                self.start_at_login_item().get().unwrap().setState(if ok {
                    NSControlStateValueOn
                } else {
                    NSControlStateValueOff
                });

                if !ok {
                    NSBeep();
                }
            }
        }
    }
);

impl AppDelegate {
    // FIXME: Make it possible to avoid this boilerplate.
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm);
        let this = this.set_ivars(Ivars::<Self> {
            status_item: Default::default(),
            start_at_login_item: Default::default(),
        });
        // SAFETY: `AppDelegate` is safe to initialize.
        unsafe { msg_send![super(this), init] }
    }
}

/// Configure the application delegate.
///
/// Normally you'd specify the name of the app delegate in `Info.plist`, and
/// let `NSApplication::main` initialize your delegate. But since we're
/// compiling with Cargo, that won't work, and we have to set the delegate
/// manually.
pub fn set_application_delegate(app: &NSApplication) {
    let delegate = AppDelegate::new(app.mtm());
    let object = ProtocolObject::from_ref(&*delegate);
    app.setDelegate(Some(object));
}

fn is_start_at_login_enabled() -> bool {
    NSFileManager::defaultManager().fileExistsAtPath(&launch_agent_plist_path())
}

fn launch_agent_plist_path() -> Retained<NSString> {
    NSHomeDirectory().stringByAppendingPathComponent(&NSString::from_str(&format!(
        "Library/LaunchAgents/{}.plist",
        ORG_BUNDLE_ID
    )))
}

fn enable_start_login() -> bool {
    let directory = launch_agent_plist_path().stringByDeletingLastPathComponent();
    unsafe {
        NSFileManager::defaultManager()
            .createDirectoryAtPath_withIntermediateDirectories_attributes_error(
                &directory, true, None,
            )
            .unwrap()
    };
    let plist = plist_contents();

    plist
        .writeToFile_atomically_encoding_error(
            &launch_agent_plist_path(),
            true,
            NSUTF8StringEncoding,
        )
        .is_ok()
}

fn plist_contents() -> Retained<NSString> {
    NSString::from_str(&format!(
        r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n
         "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" 
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n
         <plist version=\"1.0\">\n
         <dict>\n"
           <key>Label</key>\n
           <string>{}</string>\n
           <key>ProgramArguments</key>\n
           <array>\n
             <string>{}</string>\n
           </array>\n
           <key>RunAtLoad</key>\n
           <true/>\n
           <key>KeepAlive</key>\n
           <false/>\n
         </dict>\n
         </plist>\n"#,
        ORG_BUNDLE_ID,
        executable_path().unwrap_or_default()
    ))
}

fn executable_path() -> Option<Retained<NSString>> {
    NSBundle::mainBundle().executablePath()
}

fn disable_start_login() -> bool {
    let uid_string = NSString::from_str(&format!("{}", unsafe { libc::getuid() }));
    let domain = NSString::from_str(&format!("gui/{}", uid_string));
    run_launch_ctl(&[
        "unload",
        domain.to_string().as_str(),
        launch_agent_plist_path().to_string().as_str(),
    ])
    .unwrap();
    NSFileManager::defaultManager()
        .removeItemAtPath_error(&launch_agent_plist_path())
        .is_ok()
}

fn run_launch_ctl(args: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new("launchctl").args(args).output()
}
