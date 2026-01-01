#![cfg_attr(feature = "unstable-darwin-objc", feature(darwin_objc))]

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSRunningApplication};
use objc2_foundation::NSBundle;

mod app_delegate;
pub mod discord;
pub mod logic;
pub mod state;
pub mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mtm = MainThreadMarker::new().unwrap();
    objc2::rc::autoreleasepool(|_| {
        let app = NSApplication::sharedApplication(mtm);
        app_delegate::set_application_delegate(&app);
        let bid = NSBundle::mainBundle().bundleIdentifier();

        if let Some(bid) = bid {
            let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bid);

            if apps.count() > 1 {
                return;
            }
        }
        app.run();
    });

    Ok(())
}
