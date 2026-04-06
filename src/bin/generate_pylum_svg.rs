//! Kokoro — Pylum SVG Sprite Generator
//! Usage: cargo run --bin generate_pylum_svg

mod svg_common;
use svg_common::*;
use std::path::{Path, PathBuf};

const BODY: Rgb = Rgb(255, 220, 140);
const BELLY: Rgb = Rgb(255, 238, 180);
const WING: Rgb = Rgb(240, 200, 120);
const BEAK: Rgb = Rgb(255, 160, 60);
const EYE: Rgb = Rgb(30, 30, 45);

fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("pylum")
}

fn gen_egg(dir: &Path) {
    let mut b = SpriteBuilder::new(200, 260);
    let (cx, cy) = (100.0, 140.0);
    let egg = vec![
        (cx, cy-100.0), (cx+30.0, cy-90.0), (cx+55.0, cy-55.0), (cx+60.0, cy),
        (cx+55.0, cy+50.0), (cx+35.0, cy+85.0), (cx, cy+95.0),
        (cx-35.0, cy+85.0), (cx-55.0, cy+50.0), (cx-60.0, cy),
        (cx-55.0, cy-55.0), (cx-30.0, cy-90.0),
    ];
    solid_body(&mut b, &egg, Rgb(240, 225, 190), false);
    // Speckles
    b.ellipse(cx-20.0, cy-30.0, 8.0, 6.0, &Rgb(220, 200, 160).hex(), None);
    b.ellipse(cx+12.0, cy+10.0, 6.0, 5.0, &Rgb(225, 205, 165).hex(), None);
    b.ellipse(cx-5.0, cy+45.0, 7.0, 5.0, &Rgb(218, 198, 158).hex(), None);
    b.ellipse(cx+22.0, cy-10.0, 5.0, 4.0, &Rgb(222, 202, 162).hex(), None);
    b.save_png("body_idle.png", dir);
}

fn gen_cub(dir: &Path) {
    let mut b = SpriteBuilder::new(280, 300);
    let (cx, cy) = (140.0, 140.0);

    // Wing stubs behind
    let wl = vec![(cx-75.0, cy-5.0), (cx-95.0, cy+12.0), (cx-72.0, cy+30.0)];
    let wr = vec![(cx+72.0, cy-5.0), (cx+95.0, cy+12.0), (cx+72.0, cy+30.0)];
    solid_body(&mut b, &wl, WING, false);
    solid_body(&mut b, &wr, WING, false);

    // Body
    let body = vec![
        (cx, cy-85.0), (cx+50.0, cy-72.0), (cx+80.0, cy-20.0),
        (cx+75.0, cy+38.0), (cx+48.0, cy+75.0), (cx, cy+85.0),
        (cx-48.0, cy+75.0), (cx-75.0, cy+38.0), (cx-80.0, cy-20.0),
        (cx-50.0, cy-72.0),
    ];
    solid_body(&mut b, &body, BODY, false);
    b.ellipse(cx, cy+8.0, 48.0, 40.0, &BELLY.hex(), None);

    // Tuft on top
    let tuft = vec![(cx-5.0, cy-82.0), (cx+5.0, cy-82.0), (cx+3.0, cy-100.0), (cx-3.0, cy-100.0)];
    solid_body(&mut b, &tuft, BODY, false);

    // Feet
    let fl = vec![(cx-30.0, cy+80.0), (cx-8.0, cy+80.0), (cx-20.0, cy+105.0)];
    let fr = vec![(cx+8.0, cy+80.0), (cx+30.0, cy+80.0), (cx+20.0, cy+105.0)];
    solid_body(&mut b, &fl, BEAK, false);
    solid_body(&mut b, &fr, BEAK, false);

    // Eyes
    let es = 14.0;
    let ey = cy-12.0;
    b.polygon(&[(cx-28.0, ey-es), (cx-28.0+es, ey), (cx-28.0, ey+es), (cx-28.0-es, ey)], &EYE.hex(), None);
    b.polygon(&[(cx+28.0, ey-es), (cx+28.0+es, ey), (cx+28.0, ey+es), (cx+28.0-es, ey)], &EYE.hex(), None);

    // Beak
    let beak = vec![(cx-10.0, cy+12.0), (cx+10.0, cy+12.0), (cx, cy+28.0)];
    solid_body(&mut b, &beak, BEAK, false);

    b.save_png("body_idle.png", dir);
}

fn gen_adult(dir: &Path) {
    let mut b = SpriteBuilder::new(380, 420);
    let cx = 190.0;

    // Wings behind
    let wl = vec![(cx-68.0, 105.0), (cx-58.0, 92.0), (cx-120.0, 145.0), (cx-140.0, 170.0), (cx-125.0, 140.0)];
    let wr = vec![(cx+58.0, 92.0), (cx+68.0, 105.0), (cx+125.0, 140.0), (cx+140.0, 170.0), (cx+120.0, 145.0)];
    solid_body(&mut b, &wl, WING, false);
    solid_body(&mut b, &wr, WING, false);

    // Head
    let head = vec![
        (cx, 15.0), (cx+48.0, 22.0), (cx+62.0, 55.0),
        (cx+52.0, 88.0), (cx, 98.0),
        (cx-52.0, 88.0), (cx-62.0, 55.0), (cx-48.0, 22.0),
    ];
    solid_body(&mut b, &head, BODY, false);

    // Crest
    let crest = vec![(cx-5.0, 18.0), (cx+5.0, 18.0), (cx+3.0, 2.0), (cx-3.0, 2.0)];
    solid_body(&mut b, &crest, BODY, false);

    // Body
    let torso = vec![
        (cx+52.0, 92.0), (cx+72.0, 128.0), (cx+78.0, 178.0),
        (cx+62.0, 232.0), (cx+30.0, 252.0),
        (cx-30.0, 252.0), (cx-62.0, 232.0), (cx-78.0, 178.0),
        (cx-72.0, 128.0), (cx-52.0, 92.0),
    ];
    solid_body(&mut b, &torso, BODY, false);
    b.ellipse(cx, 180.0, 45.0, 38.0, &BELLY.hex(), None);

    // Legs
    let ll = vec![(cx-28.0, 248.0), (cx-12.0, 248.0), (cx-10.0, 310.0), (cx-18.0, 330.0), (cx-32.0, 330.0), (cx-30.0, 310.0)];
    let lr = vec![(cx+12.0, 248.0), (cx+28.0, 248.0), (cx+30.0, 310.0), (cx+32.0, 330.0), (cx+18.0, 330.0), (cx+10.0, 310.0)];
    solid_body(&mut b, &ll, BEAK, false);
    solid_body(&mut b, &lr, BEAK, false);

    // Tail
    let tail = vec![(cx-12.0, 248.0), (cx+12.0, 248.0), (cx+8.0, 345.0), (cx, 355.0), (cx-8.0, 345.0)];
    solid_body(&mut b, &tail, WING, false);

    // Eyes
    let es = 12.0;
    let ey = 52.0;
    b.polygon(&[(cx-25.0, ey-es), (cx-25.0+es, ey), (cx-25.0, ey+es), (cx-25.0-es, ey)], &EYE.hex(), None);
    b.polygon(&[(cx+25.0, ey-es), (cx+25.0+es, ey), (cx+25.0, ey+es), (cx+25.0-es, ey)], &EYE.hex(), None);

    // Beak
    let beak = vec![(cx-12.0, 72.0), (cx+12.0, 72.0), (cx, 92.0)];
    solid_body(&mut b, &beak, BEAK, false);

    b.save_png("body_idle.png", dir);
}

fn main() {
    let base = base_dir();
    for stage in ["egg", "cub", "young", "adult", "elder"] {
        std::fs::create_dir_all(base.join(stage)).ok();
    }
    println!("Generating Pylum SVG sprites\n");
    println!("=== Egg ==="); gen_egg(&base.join("egg"));
    println!("\n=== Cub ==="); gen_cub(&base.join("cub"));
    println!("\n=== Young ==="); gen_cub(&base.join("young")); // reuse cub for now
    println!("\n=== Adult ==="); gen_adult(&base.join("adult"));
    println!("\n=== Elder ==="); gen_adult(&base.join("elder"));
    println!("\nDone!");
}
