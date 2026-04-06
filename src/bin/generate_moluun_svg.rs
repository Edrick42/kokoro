//! Kokoro — Moluun SVG Sprite Generator
//!
//! Simple, solid colors, clean shapes. No shading layers.
//!
//! Usage: cargo run --bin generate_moluun_svg

mod svg_common;

use svg_common::*;
use std::path::{Path, PathBuf};

const BODY: Rgb = Rgb(156, 232, 252);
const BELLY: Rgb = Rgb(195, 240, 250);
const EAR: Rgb = Rgb(130, 210, 240);
const EYE: Rgb = Rgb(30, 30, 40);

fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("moluun")
}

// === EGG ===
fn gen_egg(dir: &Path) {
    let mut b = SpriteBuilder::new(200, 260);
    let cx = 100.0;
    let cy = 140.0;

    let egg = vec![
        (cx, cy - 100.0), (cx + 30.0, cy - 90.0), (cx + 55.0, cy - 55.0),
        (cx + 60.0, cy), (cx + 55.0, cy + 50.0), (cx + 35.0, cy + 85.0),
        (cx, cy + 95.0),
        (cx - 35.0, cy + 85.0), (cx - 55.0, cy + 50.0), (cx - 60.0, cy),
        (cx - 55.0, cy - 55.0), (cx - 30.0, cy - 90.0),
    ];
    solid_body(&mut b, &egg, Rgb(200, 220, 230), false);

    // Texture — a few simple spots hinting at the species
    b.ellipse(cx - 18.0, cy - 25.0, 15.0, 11.0, &Rgb(180, 210, 225).hex(), None);
    b.ellipse(cx + 15.0, cy + 20.0, 12.0, 9.0, &Rgb(180, 210, 225).hex(), None);
    b.ellipse(cx - 8.0, cy + 50.0, 10.0, 7.0, &Rgb(185, 215, 228).hex(), None);

    b.save_png("body_idle.png", dir);
}

// === CUB ===
fn gen_cub(dir: &Path) {
    let mut b = SpriteBuilder::new(300, 320);
    let cx = 150.0;
    let cy = 145.0;

    // Ears behind body
    let ear_l = vec![(cx - 60.0, cy - 50.0), (cx - 38.0, cy - 100.0), (cx - 15.0, cy - 55.0)];
    let ear_r = vec![(cx + 15.0, cy - 55.0), (cx + 38.0, cy - 100.0), (cx + 60.0, cy - 50.0)];
    solid_body(&mut b, &ear_l, EAR, false);
    solid_body(&mut b, &ear_r, EAR, false);

    // Body
    let body = vec![
        (cx, cy - 88.0),
        (cx + 55.0, cy - 75.0), (cx + 85.0, cy - 20.0),
        (cx + 82.0, cy + 40.0), (cx + 50.0, cy + 80.0),
        (cx, cy + 88.0),
        (cx - 50.0, cy + 80.0), (cx - 82.0, cy + 40.0),
        (cx - 85.0, cy - 20.0), (cx - 55.0, cy - 75.0),
    ];
    solid_body(&mut b, &body, BODY, false);

    // Belly
    b.ellipse(cx, cy + 10.0, 50.0, 42.0, &BELLY.hex(), None);

    // Feet
    let fl = vec![(cx - 42.0, cy + 82.0), (cx - 10.0, cy + 82.0), (cx - 26.0, cy + 112.0)];
    let fr = vec![(cx + 10.0, cy + 82.0), (cx + 42.0, cy + 82.0), (cx + 26.0, cy + 112.0)];
    solid_body(&mut b, &fl, BODY, false);
    solid_body(&mut b, &fr, BODY, false);

    // Arms
    let al = vec![(cx - 80.0, cy - 8.0), (cx - 105.0, cy + 18.0), (cx - 80.0, cy + 42.0)];
    let ar = vec![(cx + 80.0, cy - 8.0), (cx + 105.0, cy + 18.0), (cx + 80.0, cy + 42.0)];
    solid_body(&mut b, &al, BODY, false);
    solid_body(&mut b, &ar, BODY, false);

    // Eyes
    let es = 15.0;
    let ey = cy - 12.0;
    b.polygon(&[(cx-30.0, ey-es), (cx-30.0+es, ey), (cx-30.0, ey+es), (cx-30.0-es, ey)], &EYE.hex(), None);
    b.polygon(&[(cx+30.0, ey-es), (cx+30.0+es, ey), (cx+30.0, ey+es), (cx+30.0-es, ey)], &EYE.hex(), None);

    // Mouth
    b.ellipse(cx, cy + 22.0, 7.0, 4.0, &EYE.hex(), None);

    b.save_png("body_idle.png", dir);
}

// === ADULT ===
fn gen_adult(dir: &Path) {
    let mut b = SpriteBuilder::new(380, 440);
    let cx = 190.0;

    // Ears behind head
    let ear_l = vec![(cx - 62.0, 30.0), (cx - 45.0, 6.0), (cx - 28.0, 24.0)];
    let ear_r = vec![(cx + 28.0, 24.0), (cx + 45.0, 6.0), (cx + 62.0, 30.0)];
    solid_body(&mut b, &ear_l, EAR, false);
    solid_body(&mut b, &ear_r, EAR, false);

    // Head
    let head = vec![
        (cx, 18.0), (cx + 62.0, 26.0), (cx + 82.0, 68.0),
        (cx + 72.0, 112.0), (cx, 122.0),
        (cx - 72.0, 112.0), (cx - 82.0, 68.0), (cx - 62.0, 26.0),
    ];
    solid_body(&mut b, &head, BODY, false);

    // Muzzle
    b.ellipse(cx, 92.0, 28.0, 20.0, &BELLY.hex(), None);

    // Body
    let torso = vec![
        (cx + 72.0, 118.0), (cx + 98.0, 168.0), (cx + 108.0, 228.0),
        (cx + 92.0, 288.0), (cx + 62.0, 312.0),
        (cx - 62.0, 312.0), (cx - 92.0, 288.0), (cx - 108.0, 228.0),
        (cx - 98.0, 168.0), (cx - 72.0, 118.0),
    ];
    solid_body(&mut b, &torso, BODY, false);

    // Belly
    b.ellipse(cx, 232.0, 52.0, 42.0, &BELLY.hex(), None);

    // Legs
    let ll = vec![
        (cx-62.0, 308.0), (cx-22.0, 308.0),
        (cx-20.0, 385.0), (cx-32.0, 410.0), (cx-68.0, 410.0), (cx-68.0, 385.0),
    ];
    let lr = vec![
        (cx+22.0, 308.0), (cx+62.0, 308.0),
        (cx+68.0, 385.0), (cx+68.0, 410.0), (cx+32.0, 410.0), (cx+20.0, 385.0),
    ];
    solid_body(&mut b, &ll, BODY, false);
    solid_body(&mut b, &lr, BODY, false);

    // Arms
    let al = vec![
        (cx-98.0, 142.0), (cx-72.0, 128.0),
        (cx-82.0, 242.0), (cx-98.0, 255.0), (cx-128.0, 245.0), (cx-128.0, 192.0),
    ];
    let ar = vec![
        (cx+72.0, 128.0), (cx+98.0, 142.0),
        (cx+128.0, 192.0), (cx+128.0, 245.0), (cx+98.0, 255.0), (cx+82.0, 242.0),
    ];
    solid_body(&mut b, &al, BODY, false);
    solid_body(&mut b, &ar, BODY, false);

    // Eyes
    let es = 12.0;
    let ey = 62.0;
    b.polygon(&[(cx-30.0, ey-es), (cx-30.0+es, ey), (cx-30.0, ey+es), (cx-30.0-es, ey)], &EYE.hex(), None);
    b.polygon(&[(cx+30.0, ey-es), (cx+30.0+es, ey), (cx+30.0, ey+es), (cx+30.0-es, ey)], &EYE.hex(), None);

    // Nose
    b.circle(cx - 5.0, 86.0, 3.0, &EYE.hex(), None);
    b.circle(cx + 5.0, 86.0, 3.0, &EYE.hex(), None);

    b.save_png("body_idle.png", dir);
}

fn main() {
    let base = base_dir();
    let egg = base.join("egg");
    let cub = base.join("cub");
    let adult = base.join("adult");

    for d in [&egg, &cub, &adult] {
        std::fs::create_dir_all(d).ok();
    }

    let young = base.join("young");
    let elder = base.join("elder");
    for d in [&young, &elder] { std::fs::create_dir_all(d).ok(); }

    println!("Generating Moluun SVG sprites\n");
    println!("=== Egg ==="); gen_egg(&egg);
    println!("\n=== Cub ==="); gen_cub(&cub);
    println!("\n=== Young ==="); gen_cub(&young); // reuse cub for now
    println!("\n=== Adult ==="); gen_adult(&adult);
    println!("\n=== Elder ==="); gen_adult(&elder); // reuse adult for now
    println!("\nDone!");
}
