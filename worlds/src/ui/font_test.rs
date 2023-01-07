use super::font::Font;

#[test]
pub fn load_font() {
    let font = Font::new("C:/Users/benwyngaard/Documents/Projects/rust_worlds/resources/verdana_sdf.fnt");
    assert!(font.is_ok(), "Failed to create font: {}", font.err().unwrap());

    println!("{:#?}", font.ok());

    assert!(false)
}