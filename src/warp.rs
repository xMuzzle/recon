use std::ffi::c_void;
use std::ptr;

use core_foundation::{
    base::{CFType, TCFType},
    array::CFArray,
    string::CFString,
};

type AXUIElementRef = *const c_void;
type AXError = i32;
const K_AX_ERROR_SUCCESS: AXError = 0;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: core_foundation_sys::string::CFStringRef,
        value: *mut *const c_void,
    ) -> AXError;
    fn AXUIElementPerformAction(
        element: AXUIElementRef,
        action: core_foundation_sys::string::CFStringRef,
    ) -> AXError;
    fn AXIsProcessTrusted() -> bool;
}

fn get_attr_value(element: AXUIElementRef, attr: &str) -> Option<*const c_void> {
    let cf_attr = CFString::new(attr);
    let mut value: *const c_void = ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(element, cf_attr.as_concrete_TypeRef(), &mut value)
    };
    if err == K_AX_ERROR_SUCCESS && !value.is_null() {
        Some(value)
    } else {
        None
    }
}

fn get_attr_string(element: AXUIElementRef, attr: &str) -> Option<String> {
    get_attr_value(element, attr).map(|v| {
        let cf: CFString = unsafe { TCFType::wrap_under_get_rule(v as _) };
        cf.to_string()
    })
}

fn get_attr_children(element: AXUIElementRef) -> Vec<AXUIElementRef> {
    match get_attr_value(element, "AXChildren") {
        Some(v) => {
            let arr: CFArray<CFType> = unsafe { TCFType::wrap_under_get_rule(v as _) };
            (0..arr.len())
                .map(|i| {
                    let item = arr.get(i).unwrap();
                    item.as_CFTypeRef() as AXUIElementRef
                })
                .collect()
        }
        None => vec![],
    }
}

fn perform_action(element: AXUIElementRef, action: &str) -> bool {
    let cf_action = CFString::new(action);
    let err = unsafe { AXUIElementPerformAction(element, cf_action.as_concrete_TypeRef()) };
    err == K_AX_ERROR_SUCCESS
}

/// Check if this process has accessibility permissions.
pub fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Send Cmd+N keystroke to Warp via osascript to jump to tab N (1-9).
pub fn switch_to_tab_number(n: u8) {
    if !(1..=9).contains(&n) {
        return;
    }
    let script = format!(
        r#"tell application "Warp" to activate
delay 0.15
tell application "System Events"
    keystroke "{n}" using command down
end tell"#
    );
    let _ = std::process::Command::new("osascript")
        .args(["-e", &script])
        .output();
}

/// Get the titles of all Warp tabs via the Window menu's AX tree.
/// Returns them in display order (matching Cmd+1, Cmd+2, ...).
pub fn get_tab_titles() -> Vec<String> {
    let pid = match find_warp_pid() {
        Some(p) => p,
        None => return vec![],
    };
    let app = unsafe { AXUIElementCreateApplication(pid) };
    if app.is_null() {
        return vec![];
    }

    let menu_bar = match get_attr_value(app, "AXMenuBar") {
        Some(v) => v as AXUIElementRef,
        None => return vec![],
    };

    let menu_bar_items = get_attr_children(menu_bar);
    let window_menu_bar_item = match menu_bar_items.iter().find(|item| {
        get_attr_string(**item, "AXTitle")
            .map(|t| t == "Window")
            .unwrap_or(false)
    }) {
        Some(item) => *item,
        None => return vec![],
    };

    let submenus = get_attr_children(window_menu_bar_item);
    let window_menu = match submenus.first() {
        Some(m) => *m,
        None => return vec![],
    };

    let mut titles = Vec::new();
    for item in get_attr_children(window_menu) {
        let id = get_attr_string(item, "AXIdentifier").unwrap_or_default();
        if id == "makeKeyAndOrderFront:" {
            let title = get_attr_string(item, "AXTitle").unwrap_or_default();
            titles.push(title);
        }
    }
    titles
}

fn find_warp_pid() -> Option<i32> {
    let output = std::process::Command::new("ps")
        .args(["-eo", "pid,comm"])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&output.stdout);
    for line in s.lines() {
        if line.contains("Warp.app") && !line.contains("terminal-server") {
            return line.trim().split_whitespace().next()?.parse().ok();
        }
    }
    None
}

fn _find_tab_menu_action(app: AXUIElementRef, action_title: &str) -> Option<AXUIElementRef> {
    let menu_bar = get_attr_value(app, "AXMenuBar")? as AXUIElementRef;
    let menu_bar_items = get_attr_children(menu_bar);

    let tab_menu_item = menu_bar_items.iter().find(|item| {
        get_attr_string(**item, "AXTitle")
            .map(|t| t == "Tab")
            .unwrap_or(false)
    })?;

    let submenus = get_attr_children(*tab_menu_item);
    let tab_menu = *submenus.first()?;
    let items = get_attr_children(tab_menu);

    items.into_iter().find(|item| {
        get_attr_string(*item, "AXTitle")
            .map(|t| t == action_title)
            .unwrap_or(false)
    })
}
