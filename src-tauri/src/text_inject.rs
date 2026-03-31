use std::ffi::c_void;

type CGEventRef = *mut c_void;
type CGEventSourceRef = *mut c_void;

extern "C" {
    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        virtual_key: u16,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventKeyboardSetUnicodeString(
        event: CGEventRef,
        string_length: u64,
        unicode_string: *const u16,
    );
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CFRelease(cf: *const c_void);
}

const KCG_HID_EVENT_TAP: u32 = 0;

pub fn type_text(text: &str) {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    for chunk in utf16.chunks(8) {
        unsafe {
            let down = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0, true);
            let up = CGEventCreateKeyboardEvent(std::ptr::null_mut(), 0, false);
            if !down.is_null() && !up.is_null() {
                CGEventKeyboardSetUnicodeString(down, chunk.len() as u64, chunk.as_ptr());
                CGEventKeyboardSetUnicodeString(up, chunk.len() as u64, chunk.as_ptr());
                CGEventSetFlags(down, 0);
                CGEventSetFlags(up, 0);
                CGEventPost(KCG_HID_EVENT_TAP, down);
                CGEventPost(KCG_HID_EVENT_TAP, up);
                CFRelease(down);
                CFRelease(up);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
