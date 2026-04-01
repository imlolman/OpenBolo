pub fn type_text(text: &str) {
    use enigo::{Enigo, Keyboard, Settings};
    match Enigo::new(&Settings::default()) {
        Ok(mut enigo) => {
            if let Err(e) = enigo.text(text) {
                eprintln!("[openbolo] Failed to inject text: {:?}", e);
            }
        }
        Err(e) => {
            eprintln!("[openbolo] Failed to create Enigo instance: {:?}", e);
        }
    }
}
