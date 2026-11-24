#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use vision::{CvPattern, GrayImage, TemplateMatch};

const SIMILARITY: f64 = 0.8;
const FURTHER_MATCHES: usize = 3;
const NO_THRESHOLD: f64 = -2.0;
const TOLERANCE: f64 = 0.01;

#[derive(Deserialize)]
struct Hit {
    score: f64,
    x: i32,
    y: i32,
}

#[derive(Deserialize)]
struct Box {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Deserialize)]
struct Case {
    best: Hit,
    further: Vec<Hit>,
    kind: String,
    masked: bool,
    region: Box,
    template: String,
    variant: String,
}

#[derive(Deserialize)]
struct FrameFile {
    cases: Vec<Case>,
    frame: String,
}

fn testdata() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata")
}

fn read_gray(path: &Path) -> GrayImage {
    let file = File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut reader = png::Decoder::new(BufReader::new(file)).read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buffer).unwrap();
    assert_eq!(
        info.color_type,
        png::ColorType::Grayscale,
        "{}",
        path.display()
    );
    assert_eq!(info.bit_depth, png::BitDepth::Eight, "{}", path.display());
    buffer.truncate(info.buffer_size());
    GrayImage::new(info.width as usize, info.height as usize, buffer).unwrap()
}

fn template(cache: &mut HashMap<(String, bool), CvPattern>, name: &str, masked: bool) -> CvPattern {
    cache
        .entry((name.to_owned(), masked))
        .or_insert_with(|| {
            let dir = testdata().join("synthetic/templates");
            let gray = read_gray(&dir.join(name));
            if masked {
                let mask_name = format!("{}.mask.png", name.trim_end_matches(".png"));
                CvPattern::with_mask(gray, read_gray(&dir.join(mask_name))).unwrap()
            } else {
                CvPattern::new(gray)
            }
        })
        .clone()
}

fn absolute(found: &TemplateMatch, region: &Box) -> (i32, i32, f64) {
    (region.x + found.x, region.y + found.y, found.score)
}

#[test]
fn every_golden_case_agrees_with_opencv() {
    let dir = testdata().join("golden/scores");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();

    let mut templates = HashMap::new();
    let mut cases = 0;
    let mut largest = (0.0_f64, String::new());
    let mut failures = Vec::new();

    for path in &files {
        let parsed: FrameFile =
            serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap();
        let frame = CvPattern::new(read_gray(
            &testdata()
                .join("synthetic/frames")
                .join(format!("{}.png", parsed.frame)),
        ));
        for case in &parsed.cases {
            cases += 1;
            let label = format!(
                "{} {} {} {} masked={}",
                parsed.frame, case.template, case.kind, case.variant, case.masked
            );
            let pattern = template(&mut templates, &case.template, case.masked);
            let region = &case.region;
            let cropped = frame
                .crop(region.x, region.y, region.width, region.height)
                .unwrap();

            let best = cropped
                .find_matches(&pattern, NO_THRESHOLD)
                .unwrap()
                .next()
                .map(|m| absolute(&m, region));
            let Some((x, y, score)) = best else {
                failures.push(format!("{label}: no best match"));
                continue;
            };
            let diff = (score - case.best.score).abs();
            if diff > largest.0 {
                largest = (diff, format!("{label}: best"));
            }
            if diff > TOLERANCE || (x, y) != (case.best.x, case.best.y) {
                failures.push(format!(
                    "{label}: best ({x}, {y}) {score} expected ({}, {}) {}",
                    case.best.x, case.best.y, case.best.score
                ));
            }

            let yielded: Vec<(i32, i32, f64)> = cropped
                .find_matches(&pattern, SIMILARITY)
                .unwrap()
                .take(FURTHER_MATCHES + 1)
                .map(|m| absolute(&m, region))
                .collect();
            let expected: Vec<&Hit> = if case.best.score >= SIMILARITY {
                std::iter::once(&case.best).chain(&case.further).collect()
            } else {
                Vec::new()
            };
            if yielded.len() != expected.len() {
                failures.push(format!(
                    "{label}: yielded {yielded:?} expected {} entries",
                    expected.len()
                ));
            }
            for (index, (found, expected)) in yielded.iter().zip(&expected).enumerate() {
                let diff = (found.2 - expected.score).abs();
                if diff > largest.0 {
                    largest = (diff, format!("{label}: yielded {index}"));
                }
                if diff > TOLERANCE || (found.0, found.1) != (expected.x, expected.y) {
                    failures.push(format!(
                        "{label}: yielded {index} {found:?} expected ({}, {}) {}",
                        expected.x, expected.y, expected.score
                    ));
                }
            }
        }
    }

    println!(
        "{cases} cases over {} files, largest score difference {} ({})",
        files.len(),
        largest.0,
        largest.1
    );
    for failure in &failures {
        println!("{failure}");
    }
    assert_eq!(files.len(), 14);
    assert_eq!(cases, 222);
    assert!(failures.is_empty(), "{} failures", failures.len());
}
