//! Kokoro — Nyxal SVG Sprite Generator
//! Usage: cargo run --bin generate_nyxal_svg

mod svg_common;
use svg_common::*;
use std::path::{Path, PathBuf};

const BODY: Rgb = Rgb(95, 60, 130);
const BELLY: Rgb = Rgb(120, 85, 150);
const MANTLE: Rgb = Rgb(85, 50, 120);
const TENTACLE: Rgb = Rgb(100, 65, 140);
const GLOW: Rgb = Rgb(40, 180, 200);
const EYE: Rgb = Rgb(15, 15, 30);

fn base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("sprites").join("nyxal")
}

fn gen_egg(dir: &Path) {
    let mut b = SpriteBuilder::new(220, 220);
    let (cx, cy) = (110.0, 110.0);
    // Roe cluster — multiple round shapes
    b.circle(cx, cy, 40.0, &Rgb(130, 100, 160).hex(), None);
    b.circle(cx-28.0, cy-22.0, 25.0, &Rgb(125, 95, 155).hex(), None);
    b.circle(cx+28.0, cy-18.0, 22.0, &Rgb(128, 98, 158).hex(), None);
    b.circle(cx-18.0, cy+28.0, 20.0, &Rgb(122, 92, 152).hex(), None);
    b.circle(cx+22.0, cy+25.0, 18.0, &Rgb(126, 96, 156).hex(), None);
    b.save_png("body_idle.png", dir);
}

fn gen_cub(dir: &Path) {
    let mut b = SpriteBuilder::new(280, 310);
    let (cx, cy) = (140.0, 130.0);

    // Tentacle stubs behind
    let tl = vec![(cx-22.0, cy+70.0), (cx-10.0, cy+70.0), (cx-14.0, cy+115.0), (cx-20.0, cy+115.0)];
    let tr = vec![(cx+10.0, cy+70.0), (cx+22.0, cy+70.0), (cx+20.0, cy+115.0), (cx+14.0, cy+115.0)];
    solid_body(&mut b, &tl, TENTACLE, false);
    solid_body(&mut b, &tr, TENTACLE, false);
    // Glow tips
    b.circle(cx-17.0, cy+112.0, 5.0, &GLOW.hex(), None);
    b.circle(cx+17.0, cy+112.0, 5.0, &GLOW.hex(), None);

    // Body
    let body = vec![
        (cx, cy-78.0), (cx+48.0, cy-65.0), (cx+75.0, cy-15.0),
        (cx+70.0, cy+35.0), (cx+42.0, cy+68.0), (cx, cy+75.0),
        (cx-42.0, cy+68.0), (cx-70.0, cy+35.0), (cx-75.0, cy-15.0),
        (cx-48.0, cy-65.0),
    ];
    solid_body(&mut b, &body, BODY, false);
    b.ellipse(cx, cy+5.0, 42.0, 35.0, &BELLY.hex(), None);

    // Eyes — glowing cyan
    let ey = cy-12.0;
    let es = 14.0;
    b.polygon(&[(cx-28.0, ey-es), (cx-28.0+es, ey), (cx-28.0, ey+es), (cx-28.0-es, ey)], &GLOW.hex(), None);
    b.polygon(&[(cx+28.0, ey-es), (cx+28.0+es, ey), (cx+28.0, ey+es), (cx+28.0-es, ey)], &GLOW.hex(), None);
    // Slit pupils
    b.polygon(&[(cx-28.0, ey-10.0), (cx-27.0, ey), (cx-28.0, ey+10.0), (cx-29.0, ey)], &EYE.hex(), None);
    b.polygon(&[(cx+28.0, ey-10.0), (cx+27.0, ey), (cx+28.0, ey+10.0), (cx+29.0, ey)], &EYE.hex(), None);

    b.save_png("body_idle.png", dir);
}

fn gen_adult(dir: &Path) {
    let mut b = SpriteBuilder::new(380, 440);
    let cx = 190.0;

    // Tentacles behind (4)
    let positions = [
        (cx-35.0, cx-28.0, 300.0),  // front left
        (cx+28.0, cx+35.0, 300.0),  // front right
        (cx-55.0, cx-48.0, 285.0),  // back left
        (cx+48.0, cx+55.0, 285.0),  // back right
    ];
    for &(x1, x2, start_y) in &positions {
        let t = vec![(x1, start_y), (x2, start_y), ((x1+x2)/2.0+2.0, start_y+120.0), ((x1+x2)/2.0-2.0, start_y+120.0)];
        solid_body(&mut b, &t, TENTACLE, false);
        b.circle((x1+x2)/2.0, start_y+115.0, 6.0, &GLOW.hex(), None);
    }

    // Mantle dome
    let mantle = vec![
        (cx, 10.0), (cx+48.0, 18.0), (cx+62.0, 48.0),
        (cx+55.0, 78.0), (cx, 88.0),
        (cx-55.0, 78.0), (cx-62.0, 48.0), (cx-48.0, 18.0),
    ];
    solid_body(&mut b, &mantle, MANTLE, false);

    // Body
    let body = vec![
        (cx+55.0, 82.0), (cx+72.0, 115.0), (cx+68.0, 160.0),
        (cx+48.0, 195.0), (cx, 208.0),
        (cx-48.0, 195.0), (cx-68.0, 160.0), (cx-72.0, 115.0),
        (cx-55.0, 82.0),
    ];
    solid_body(&mut b, &body, BODY, false);
    b.ellipse(cx, 148.0, 42.0, 35.0, &BELLY.hex(), None);

    // Eyes — glowing
    let ey = 135.0;
    let es = 12.0;
    b.polygon(&[(cx-25.0, ey-es), (cx-25.0+es, ey), (cx-25.0, ey+es), (cx-25.0-es, ey)], &GLOW.hex(), None);
    b.polygon(&[(cx+25.0, ey-es), (cx+25.0+es, ey), (cx+25.0, ey+es), (cx+25.0-es, ey)], &GLOW.hex(), None);
    b.polygon(&[(cx-25.0, ey-8.0), (cx-24.0, ey), (cx-25.0, ey+8.0), (cx-26.0, ey)], &EYE.hex(), None);
    b.polygon(&[(cx+25.0, ey-8.0), (cx+24.0, ey), (cx+25.0, ey+8.0), (cx+26.0, ey)], &EYE.hex(), None);

    b.save_png("body_idle.png", dir);
}

fn main() {
    let base = base_dir();
    for stage in ["egg", "cub", "young", "adult", "elder"] {
        std::fs::create_dir_all(base.join(stage)).ok();
    }
    println!("Generating Nyxal SVG sprites\n");
    println!("=== Egg ==="); gen_egg(&base.join("egg"));
    println!("\n=== Cub ==="); gen_cub(&base.join("cub"));
    println!("\n=== Young ==="); gen_cub(&base.join("young"));
    println!("\n=== Adult ==="); gen_adult(&base.join("adult"));
    println!("\n=== Elder ==="); gen_adult(&base.join("elder"));
    println!("\nDone!");
}
