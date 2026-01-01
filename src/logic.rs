use crate::utils::{run_cmd, shell_quote};

use objc2_app_kit::NSRunningApplication;
use objc2_foundation::ns_string;

pub fn is_logic_pro_running() -> bool {
    objc2::rc::autoreleasepool(|_| {
        let bid = ns_string!("com.apple.logic10");
        let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(bid);
        apps.count() > 0
    })
}

pub fn get_logic_project_name() -> Option<String> {
    let script = r#"
        if application "Logic Pro" is not running then
            return ""
        end if
        tell application "Logic Pro"
            try
                return name of front document
            on error
                return ""
            end try
        end tell
    "#;

    let cmd = format!("osascript -e {}", shell_quote(script));
    let name = run_cmd(&cmd);
    if name.is_empty() {
        return None;
    }

    name.trim().strip_suffix(".logicx").map(|s| s.to_string())
}
