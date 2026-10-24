//! Test plan. The vanilla font is Mojang's file and is not shipped: it is read from the user's own copy.
//! Ways this can break: the path in HUD_CADENCE_FONT is ignored; a missing file panics instead of explaining how to get
//! one; a wrong-size PNG is accepted; the message does not name the path it tried.
use hud_cadence::font::Font;
use std::path::PathBuf;

fn tmp(name: &str, bytes: &[u8]) -> PathBuf {
    let p = std::env::temp_dir().join(format!("hc-font-{}-{name}", std::process::id()));
    std::fs::write(&p, bytes).unwrap();
    p
}

#[test]
fn a_font_file_is_loaded_from_the_given_path() {
    // any 128x128 RGBA png with an alpha channel is a valid font sheet
    let mut png_bytes = Vec::new();
    {
        let mut e = png::Encoder::new(&mut png_bytes, 128, 128);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        let mut w = e.write_header().unwrap();
        w.write_image_data(&vec![255u8; 128 * 128 * 4]).unwrap();
    }
    let p = tmp("ok.png", &png_bytes);
    assert!(Font::from_file(&p).is_ok());
}

#[test]
fn a_missing_file_says_where_to_get_the_font() {
    let e = Font::from_file(std::path::Path::new("definitely/not/here/ascii.png")).err().expect("must fail");
    assert!(e.contains("definitely/not/here/ascii.png"), "names the path: {e}");
    assert!(e.contains("HUD_CADENCE_FONT") && e.contains("font/ascii.png"), "says how to get one: {e}");
}

#[test]
fn a_file_that_is_not_a_font_sheet_is_refused() {
    let p = tmp("bad.png", b"not a png at all");
    assert!(Font::from_file(&p).is_err());
}
