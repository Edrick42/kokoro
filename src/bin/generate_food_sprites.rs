//! Kokoro — Food Item Sprite Generator
//!
//! Generates 48x48 icon sprites for each food type.
//! Low-poly collage style matching the creature visual language.
//!
//! Usage: cargo run --bin generate_food_sprites

#[allow(dead_code)]
mod sprite_common;

use image::Rgba;
use sprite_common::*;
use std::path::{Path, PathBuf};

fn out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sprites")
        .join("shared")
        .join("food")
}

const SZ: u32 = 48;

// --- Verdance Berry: round green berry with leaf ---
fn gen_verdance_berry(dir: &Path) {
    let mut img = new_canvas(SZ, SZ);
    let (cx, cy) = (24.0, 28.0);

    // Berry body
    let body = vec![
        (cx, cy - 14.0), (cx + 10.0, cy - 10.0), (cx + 14.0, cy),
        (cx + 10.0, cy + 10.0), (cx, cy + 14.0),
        (cx - 10.0, cy + 10.0), (cx - 14.0, cy),
        (cx - 10.0, cy - 10.0),
    ];
    smooth_shade(&mut img, &body, (cx, cy), Rgba([80, 180, 70, 255]), (-0.3, -1.0));

    // Small leaf on top
    let leaf = [(cx - 3.0, cy - 14.0), (cx + 5.0, cy - 18.0), (cx + 2.0, cy - 12.0)];
    let leaf_shape = polygon_pixels(&leaf);
    px_set(&mut img, &leaf_shape, Rgba([50, 140, 50, 255]));

    // Stem
    put(&mut img, cx as i32, (cy - 14.0) as i32, Rgba([80, 60, 40, 255]));
    put(&mut img, cx as i32, (cy - 15.0) as i32, Rgba([80, 60, 40, 255]));

    let all: Shape = polygon_pixels(&body).union(&leaf_shape).copied().collect();
    flood_outline(&mut img, &all, Rgba([30, 50, 25, 255]));
    save_raw(&img, "verdance_berry.png", dir);
}

// --- Lattice Fruit: golden oval fruit ---
fn gen_lattice_fruit(dir: &Path) {
    let mut img = new_canvas(SZ, SZ);
    let (cx, cy) = (24.0, 26.0);

    let body = vec![
        (cx, cy - 12.0), (cx + 8.0, cy - 10.0), (cx + 12.0, cy - 2.0),
        (cx + 10.0, cy + 8.0), (cx, cy + 12.0),
        (cx - 10.0, cy + 8.0), (cx - 12.0, cy - 2.0),
        (cx - 8.0, cy - 10.0),
    ];
    smooth_shade(&mut img, &body, (cx, cy), Rgba([230, 180, 60, 255]), (-0.3, -1.0));

    // Leaf
    let leaf = [(cx - 2.0, cy - 12.0), (cx + 6.0, cy - 16.0), (cx + 3.0, cy - 10.0)];
    px_set(&mut img, &polygon_pixels(&leaf), Rgba([60, 150, 50, 255]));

    let all: Shape = polygon_pixels(&body).union(&polygon_pixels(&leaf)).copied().collect();
    flood_outline(&mut img, &all, Rgba([60, 45, 20, 255]));
    save_raw(&img, "lattice_fruit.png", dir);
}

// --- Thermal Seed: angular brown seed with heat marks ---
fn gen_thermal_seed(dir: &Path) {
    let mut img = new_canvas(SZ, SZ);
    let (cx, cy) = (24.0, 24.0);

    // Seed shape — angular teardrop
    let body = vec![
        (cx, cy - 14.0), (cx + 10.0, cy - 4.0), (cx + 8.0, cy + 8.0),
        (cx, cy + 14.0), (cx - 8.0, cy + 8.0), (cx - 10.0, cy - 4.0),
    ];
    smooth_shade(&mut img, &body, (cx, cy), Rgba([200, 170, 90, 255]), (-0.3, -1.0));

    // Heat line marks
    let body_shape = polygon_pixels(&body);
    for &y in &[(cy - 4.0) as i32, (cy + 2.0) as i32, (cy + 8.0) as i32] {
        for x in (cx as i32 - 5)..=(cx as i32 + 5) {
            if body_shape.contains(&(x, y)) {
                put(&mut img, x, y, Rgba([180, 140, 60, 255]));
            }
        }
    }

    flood_outline(&mut img, &body_shape, Rgba([60, 45, 25, 255]));
    save_raw(&img, "thermal_seed.png", dir);
}

// --- Cave Crustacean: red-brown shell with claws ---
fn gen_cave_crustacean(dir: &Path) {
    let mut img = new_canvas(SZ, SZ);
    let (cx, cy) = (24.0, 26.0);

    // Body shell
    let body = vec![
        (cx, cy - 10.0), (cx + 12.0, cy - 6.0), (cx + 14.0, cy + 4.0),
        (cx + 10.0, cy + 10.0), (cx, cy + 12.0),
        (cx - 10.0, cy + 10.0), (cx - 14.0, cy + 4.0),
        (cx - 12.0, cy - 6.0),
    ];
    smooth_shade(&mut img, &body, (cx, cy), Rgba([190, 80, 60, 255]), (-0.3, -1.0));

    // Small claws
    let claw_l = [(cx - 14.0, cy - 2.0), (cx - 18.0, cy - 6.0), (cx - 16.0, cy + 2.0)];
    let claw_r = [(cx + 14.0, cy - 2.0), (cx + 18.0, cy - 6.0), (cx + 16.0, cy + 2.0)];
    px_set(&mut img, &polygon_pixels(&claw_l), Rgba([170, 70, 50, 255]));
    px_set(&mut img, &polygon_pixels(&claw_r), Rgba([170, 70, 50, 255]));

    // Eyes
    put(&mut img, (cx - 4.0) as i32, (cy - 8.0) as i32, Rgba([20, 20, 20, 255]));
    put(&mut img, (cx + 4.0) as i32, (cy - 8.0) as i32, Rgba([20, 20, 20, 255]));

    let all: Shape = polygon_pixels(&body)
        .union(&polygon_pixels(&claw_l))
        .chain(&polygon_pixels(&claw_r))
        .copied().collect();
    flood_outline(&mut img, &all, Rgba([50, 25, 20, 255]));
    save_raw(&img, "cave_crustacean.png", dir);
}

// --- Biolum Plankton: cyan glowing cluster ---
fn gen_biolum_plankton(dir: &Path) {
    let mut img = new_canvas(SZ, SZ);

    // Multiple small glowing dots
    for &(cx, cy, r) in &[
        (24.0, 24.0, 8.0), (16.0, 18.0, 5.0), (32.0, 20.0, 5.0),
        (20.0, 32.0, 4.0), (30.0, 30.0, 4.0), (14.0, 28.0, 3.0),
    ] {
        let verts: Vec<(f32, f32)> = (0..6).map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 6.0;
            (cx + a.cos() * r, cy + a.sin() * r)
        }).collect();
        smooth_shade(&mut img, &verts, (cx, cy), Rgba([60, 190, 200, 255]), (-0.3, -1.0));
        flood_outline(&mut img, &polygon_pixels(&verts), Rgba([20, 60, 70, 255]));
    }

    save_raw(&img, "biolum_plankton.png", dir);
}

// --- Root Tuber: brown rounded root ---
fn gen_root_tuber(dir: &Path) {
    let mut img = new_canvas(SZ, SZ);
    let (cx, cy) = (24.0, 26.0);

    let body = vec![
        (cx - 4.0, cy - 14.0), (cx + 4.0, cy - 14.0),
        (cx + 10.0, cy - 4.0), (cx + 12.0, cy + 6.0),
        (cx + 8.0, cy + 14.0), (cx - 8.0, cy + 14.0),
        (cx - 12.0, cy + 6.0), (cx - 10.0, cy - 4.0),
    ];
    smooth_shade(&mut img, &body, (cx, cy), Rgba([160, 120, 70, 255]), (-0.3, -1.0));

    // Root lines
    let body_shape = polygon_pixels(&body);
    for &(rx, ry, rr) in &[(cx + 8.0, cy + 10.0, 4.0), (cx - 6.0, cy + 12.0, 3.0)] {
        let root = ellipse_pixels(rx, ry, rr, 2.0);
        for &(x, y) in &root {
            if !body_shape.contains(&(x, y)) {
                put(&mut img, x, y, Rgba([140, 100, 55, 255]));
            }
        }
    }

    flood_outline(&mut img, &body_shape, Rgba([50, 35, 20, 255]));
    save_raw(&img, "root_tuber.png", dir);
}

// --- Spore Moss: green cluster with sparkles ---
fn gen_spore_moss(dir: &Path) {
    let mut img = new_canvas(SZ, SZ);
    let (cx, cy) = (24.0, 26.0);

    // Moss mound
    let body = vec![
        (cx - 14.0, cy + 8.0), (cx - 12.0, cy - 4.0),
        (cx - 4.0, cy - 10.0), (cx + 4.0, cy - 10.0),
        (cx + 12.0, cy - 4.0), (cx + 14.0, cy + 8.0),
        (cx, cy + 12.0),
    ];
    smooth_shade(&mut img, &body, (cx, cy), Rgba([80, 140, 80, 255]), (-0.3, -1.0));

    // Spore sparkles
    for &(sx, sy) in &[(cx - 4.0, cy - 8.0), (cx + 6.0, cy - 6.0), (cx, cy - 4.0)] {
        let spore = diamond_pixels(sx, sy, 2.0, 2.0);
        px_set(&mut img, &spore, Rgba([200, 220, 100, 255]));
    }

    flood_outline(&mut img, &polygon_pixels(&body), Rgba([30, 50, 30, 255]));
    save_raw(&img, "spore_moss.png", dir);
}

// --- Crystal Water: blue crystal droplet ---
fn gen_crystal_water(dir: &Path) {
    let mut img = new_canvas(SZ, SZ);
    let (cx, cy) = (24.0, 24.0);

    // Droplet shape
    let body = vec![
        (cx, cy - 14.0),
        (cx + 10.0, cy + 2.0), (cx + 8.0, cy + 10.0),
        (cx, cy + 14.0),
        (cx - 8.0, cy + 10.0), (cx - 10.0, cy + 2.0),
    ];
    smooth_shade(&mut img, &body, (cx, cy), Rgba([100, 160, 220, 255]), (-0.3, -1.0));

    // Sparkle highlight
    let sparkle = diamond_pixels(cx - 3.0, cy - 4.0, 2.0, 2.0);
    px_set(&mut img, &sparkle, Rgba([200, 230, 255, 255]));

    flood_outline(&mut img, &polygon_pixels(&body), Rgba([30, 50, 80, 255]));
    save_raw(&img, "crystal_water.png", dir);
}

fn main() {
    let dir = out_dir();
    std::fs::create_dir_all(&dir).ok();
    println!("Generating food sprites -> {}\n", dir.display());

    gen_verdance_berry(&dir);
    gen_lattice_fruit(&dir);
    gen_thermal_seed(&dir);
    gen_cave_crustacean(&dir);
    gen_biolum_plankton(&dir);
    gen_root_tuber(&dir);
    gen_spore_moss(&dir);
    gen_crystal_water(&dir);

    println!("\nDone! 8 food sprites saved.");
}
