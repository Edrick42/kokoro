//! Kokoro — Skael SVG Sprite Generator
//! Usage: cargo run --bin generate_skael_svg

mod svg_common;
use svg_common::*;
use std::path::{Path, PathBuf};

const BODY: Rgb = Rgb(120, 180, 130);
const BELLY: Rgb = Rgb(155, 210, 160);
const CREST: Rgb = Rgb(160, 100, 80);
const EYE: Rgb = Rgb(30, 30, 30);
const IRIS: Rgb = Rgb(200, 160, 50);

fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("skael")
}

fn gen_egg(dir: &Path) {
    let mut b = SpriteBuilder::new(200, 260);
    let (cx, cy) = (100.0, 140.0);
    let egg = vec![
        (cx, cy-100.0), (cx+32.0, cy-85.0), (cx+55.0, cy-45.0), (cx+52.0, cy+15.0),
        (cx+38.0, cy+60.0), (cx, cy+90.0),
        (cx-38.0, cy+60.0), (cx-52.0, cy+15.0), (cx-55.0, cy-45.0), (cx-32.0, cy-85.0),
    ];
    solid_body(&mut b, &egg, Rgb(160, 190, 150), false);
    // Crystal facet texture
    b.polygon(&[(cx+8.0, cy-30.0), (cx+22.0, cy-18.0), (cx+12.0, cy-5.0), (cx-2.0, cy-15.0)], &Rgb(140, 175, 135).hex(), None);
    b.polygon(&[(cx-15.0, cy+15.0), (cx-5.0, cy+25.0), (cx-18.0, cy+35.0)], &Rgb(145, 178, 138).hex(), None);
    b.save_png("body_idle.png", dir);
}

fn gen_cub(dir: &Path) {
    let mut b = SpriteBuilder::new(280, 310);
    let (cx, cy) = (140.0, 140.0);

    // Body
    let body = vec![
        (cx, cy-82.0), (cx+45.0, cy-70.0), (cx+72.0, cy-22.0),
        (cx+68.0, cy+32.0), (cx+42.0, cy+72.0), (cx, cy+82.0),
        (cx-42.0, cy+72.0), (cx-68.0, cy+32.0), (cx-72.0, cy-22.0),
        (cx-45.0, cy-70.0),
    ];
    solid_body(&mut b, &body, BODY, false);
    b.ellipse(cx, cy+8.0, 42.0, 35.0, &BELLY.hex(), None);

    // Stub tail
    let tail = vec![(cx-10.0, cy+78.0), (cx+10.0, cy+78.0), (cx+6.0, cy+115.0), (cx-6.0, cy+115.0)];
    solid_body(&mut b, &tail, BODY, false);

    // Feet
    let fl = vec![(cx-35.0, cy+75.0), (cx-12.0, cy+75.0), (cx-15.0, cy+100.0), (cx-38.0, cy+100.0)];
    let fr = vec![(cx+12.0, cy+75.0), (cx+35.0, cy+75.0), (cx+38.0, cy+100.0), (cx+15.0, cy+100.0)];
    solid_body(&mut b, &fl, BODY, false);
    solid_body(&mut b, &fr, BODY, false);

    // Eyes — big solid black diamonds
    let ey = cy-12.0;
    let es = 13.0;
    b.polygon(&[(cx-24.0, ey-es), (cx-24.0+es, ey), (cx-24.0, ey+es), (cx-24.0-es, ey)], &EYE.hex(), None);
    b.polygon(&[(cx+24.0, ey-es), (cx+24.0+es, ey), (cx+24.0, ey+es), (cx+24.0-es, ey)], &EYE.hex(), None);

    // Snout
    b.ellipse(cx, cy+18.0, 12.0, 8.0, &BODY.darken(0.85).hex(), None);

    b.save_png("body_idle.png", dir);
}

fn gen_adult(dir: &Path) {
    let mut b = SpriteBuilder::new(360, 440);
    let cx = 180.0;

    // Crests behind
    let cl = vec![(cx-42.0, 25.0), (cx-30.0, 5.0), (cx-18.0, 20.0)];
    let cr = vec![(cx+18.0, 20.0), (cx+30.0, 5.0), (cx+42.0, 25.0)];
    solid_body(&mut b, &cl, CREST, false);
    solid_body(&mut b, &cr, CREST, false);

    // Head
    let head = vec![
        (cx, 15.0), (cx+52.0, 22.0), (cx+68.0, 58.0),
        (cx+58.0, 98.0), (cx, 108.0),
        (cx-58.0, 98.0), (cx-68.0, 58.0), (cx-52.0, 22.0),
    ];
    solid_body(&mut b, &head, BODY, false);

    // Body
    let torso = vec![
        (cx+58.0, 102.0), (cx+78.0, 140.0), (cx+82.0, 195.0),
        (cx+72.0, 250.0), (cx+48.0, 278.0),
        (cx-48.0, 278.0), (cx-72.0, 250.0), (cx-82.0, 195.0),
        (cx-78.0, 140.0), (cx-58.0, 102.0),
    ];
    solid_body(&mut b, &torso, BODY, false);
    b.ellipse(cx, 200.0, 48.0, 40.0, &BELLY.hex(), None);

    // Legs
    let ll = vec![(cx-48.0, 272.0), (cx-18.0, 272.0), (cx-16.0, 348.0), (cx-28.0, 370.0), (cx-52.0, 370.0), (cx-52.0, 348.0)];
    let lr = vec![(cx+18.0, 272.0), (cx+48.0, 272.0), (cx+52.0, 348.0), (cx+52.0, 370.0), (cx+28.0, 370.0), (cx+16.0, 348.0)];
    solid_body(&mut b, &ll, BODY, false);
    solid_body(&mut b, &lr, BODY, false);

    // Tail
    let tail = vec![(cx-15.0, 272.0), (cx+15.0, 272.0), (cx+10.0, 365.0), (cx+3.0, 400.0), (cx-3.0, 400.0), (cx-10.0, 365.0)];
    solid_body(&mut b, &tail, BODY, false);

    // Eyes
    let ey = 55.0;
    b.polygon(&[(cx-22.0, ey-10.0), (cx-12.0, ey), (cx-22.0, ey+10.0), (cx-32.0, ey)], &IRIS.hex(), None);
    b.polygon(&[(cx+22.0, ey-10.0), (cx+32.0, ey), (cx+22.0, ey+10.0), (cx+12.0, ey)], &IRIS.hex(), None);
    b.polygon(&[(cx-22.0, ey-7.0), (cx-21.0, ey), (cx-22.0, ey+7.0), (cx-23.0, ey)], &EYE.hex(), None);
    b.polygon(&[(cx+22.0, ey-7.0), (cx+21.0, ey), (cx+22.0, ey+7.0), (cx+23.0, ey)], &EYE.hex(), None);

    // Snout
    b.ellipse(cx, 85.0, 18.0, 12.0, &BODY.darken(0.85).hex(), None);

    b.save_png("body_idle.png", dir);
}

fn main() {
    let base = base_dir();
    for stage in ["egg", "cub", "young", "adult", "elder"] {
        std::fs::create_dir_all(base.join(stage)).ok();
    }
    println!("Generating Skael SVG sprites\n");
    println!("=== Egg ==="); gen_egg(&base.join("egg"));
    println!("\n=== Cub ==="); gen_cub(&base.join("cub"));
    println!("\n=== Young ==="); gen_cub(&base.join("young"));
    println!("\n=== Adult ==="); gen_adult(&base.join("adult"));
    println!("\n=== Elder ==="); gen_adult(&base.join("elder"));
    println!("\nDone!");
}
