use fontdue::{Font, FontSettings, layout::Layout};

pub struct FontContext {
    font: Font
}

impl FontContext {
    pub fn new() -> Self {
        Self {
            font: Font::from_bytes(
                      include_bytes!("../resources/Poppins-Regular.ttf") as &[u8],
                      FontSettings::default()
                      ).unwrap(),
        }
    }
}
