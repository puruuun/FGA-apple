use std::fs;
use std::path::{Path, PathBuf};

use automata::{
    GameAreaManager, Location, RealScale, RealTransformer, Region, Scale, Size, Transformer,
};
use scripts::fgo_game_area_manager::{FgoGameAreaManager, is_wide};
use serde_json::Value;

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/golden/locations")
}

fn golden_files() -> Result<Vec<PathBuf>, String> {
    let dir = golden_dir();
    let entries = fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| format!("cannot list {}: {e}", dir.display()))?
            .path();
        if path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!("no golden files in {}", dir.display()));
    }
    Ok(files)
}

fn int(value: &Value, path: &str) -> Result<i32, String> {
    value
        .as_i64()
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| format!("{path}: not an int: {value}"))
}

fn field<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a Value, String> {
    value
        .get(name)
        .ok_or_else(|| format!("{path}: missing {name}"))
}

fn expected_region(value: &Value, path: &str) -> Result<[i32; 4], String> {
    Ok([
        int(field(value, "x", path)?, &format!("{path}.x"))?,
        int(field(value, "y", path)?, &format!("{path}.y"))?,
        int(field(value, "width", path)?, &format!("{path}.width"))?,
        int(field(value, "height", path)?, &format!("{path}.height"))?,
    ])
}

fn parts(region: Region) -> [i32; 4] {
    [region.x(), region.y(), region.width(), region.height()]
}

fn float(value: &Value, path: &str) -> Result<Option<f64>, String> {
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_f64()
        .map(Some)
        .ok_or_else(|| format!("{path}: not a number: {value}"))
}

#[derive(Default)]
struct Counts {
    files: usize,
    entries: usize,
    locations: usize,
    regions: usize,
    numbers: usize,
}

fn check_file(path: &Path, counts: &mut Counts, failures: &mut Vec<String>) -> Result<(), String> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let text = fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("{name}: {e}"))?;

    let screen = field(&doc, "screen", &name)?;
    let screen_size = Size::new(
        int(
            field(screen, "width", &name)?,
            &format!("{name}:screen.width"),
        )?,
        int(
            field(screen, "height", &name)?,
            &format!("{name}:screen.height"),
        )?,
    )
    .map_err(|e| format!("{name}: screen: {e}"))?;

    let manager = FgoGameAreaManager::new(screen_size, Location::default)
        .map_err(|e| format!("{name}: game area manager: {e}"))?;
    let scale = RealScale::new(&manager);
    let transformer = RealTransformer::new(&manager, &scale);

    let mut fail = |what: &str, expected: String, actual: String| {
        failures.push(format!("{name}: {what}: expected {expected}, got {actual}"));
    };

    let wide = field(&doc, "isWide", &name)?
        .as_bool()
        .ok_or_else(|| format!("{name}: isWide is not a bool"))?;
    if wide != is_wide(screen_size) {
        fail("isWide", wide.to_string(), is_wide(screen_size).to_string());
    }

    let expected_area =
        expected_region(field(&doc, "gameArea", &name)?, &format!("{name}:gameArea"))?;
    if expected_area != parts(manager.game_area()) {
        fail(
            "gameArea",
            format!("{expected_area:?}"),
            format!("{:?}", parts(manager.game_area())),
        );
    }

    let scales = [
        ("scriptToScreen", Some(scale.script_to_screen())),
        ("screenToImage", scale.screen_to_image()),
        ("scriptToImage", Some(scale.script_to_image())),
    ];
    for (key, actual) in scales {
        let expected = float(field(&doc, key, &name)?, &format!("{name}:{key}"))?;
        counts.numbers += 1;
        if expected != actual {
            fail(key, format!("{expected:?}"), format!("{actual:?}"));
        }
    }

    let values = field(&doc, "values", &name)?
        .as_object()
        .ok_or_else(|| format!("{name}: values is not an object"))?;
    for (key, entry) in values {
        counts.entries += 1;
        let Some(script) = entry.get("script") else {
            continue;
        };
        if entry.get("error").is_some() {
            continue;
        }
        let at = format!("{name}:{key}");
        let expected_screen = field(entry, "screen", &at)?;

        if script.get("width").is_some() {
            counts.regions += 1;
            let script_region = expected_region(script, &format!("{at}.script"))?;
            let region = Region::new(
                script_region[0],
                script_region[1],
                script_region[2],
                script_region[3],
            )
            .map_err(|e| format!("{at}.script: {e}"))?;

            let expected = expected_region(expected_screen, &format!("{at}.screen"))?;
            match transformer.to_screen_region(region) {
                Ok(actual) if parts(actual) == expected => {}
                Ok(actual) => fail(
                    &format!("{key} script {script_region:?} to screen"),
                    format!("{expected:?}"),
                    format!("{:?}", parts(actual)),
                ),
                Err(e) => fail(
                    &format!("{key} script {script_region:?} to screen"),
                    format!("{expected:?}"),
                    e.to_string(),
                ),
            }

            if let Some(image) = entry.get("image") {
                let expected = expected_region(image, &format!("{at}.image"))?;
                match transformer.to_image(region) {
                    Ok(actual) if parts(actual) == expected => {}
                    Ok(actual) => fail(
                        &format!("{key} script {script_region:?} to image"),
                        format!("{expected:?}"),
                        format!("{:?}", parts(actual)),
                    ),
                    Err(e) => fail(
                        &format!("{key} script {script_region:?} to image"),
                        format!("{expected:?}"),
                        e.to_string(),
                    ),
                }
            }
        } else {
            counts.locations += 1;
            let location = Location::new(
                int(field(script, "x", &at)?, &format!("{at}.script.x"))?,
                int(field(script, "y", &at)?, &format!("{at}.script.y"))?,
            );
            let expected = Location::new(
                int(field(expected_screen, "x", &at)?, &format!("{at}.screen.x"))?,
                int(field(expected_screen, "y", &at)?, &format!("{at}.screen.y"))?,
            );
            match transformer.to_screen_location(location) {
                Ok(actual) if actual == expected => {}
                Ok(actual) => fail(
                    &format!("{key} script {location:?} to screen"),
                    format!("{expected:?}"),
                    format!("{actual:?}"),
                ),
                Err(e) => fail(
                    &format!("{key} script {location:?} to screen"),
                    format!("{expected:?}"),
                    e.to_string(),
                ),
            }
        }
    }

    counts.files += 1;
    Ok(())
}

#[test]
fn transforms_match_the_kotlin_golden_files() {
    let files = golden_files().expect("golden files");
    let mut counts = Counts::default();
    let mut failures = Vec::new();
    for file in &files {
        if let Err(message) = check_file(file, &mut counts, &mut failures) {
            failures.push(message);
        }
    }
    println!(
        "golden files: {}, entries: {}, locations: {}, regions: {}, scale numbers: {}",
        counts.files, counts.entries, counts.locations, counts.regions, counts.numbers
    );
    assert!(
        failures.is_empty(),
        "{} mismatches, first 20:\n{}",
        failures.len(),
        failures
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        counts.locations > 0 && counts.regions > 0,
        "no values were checked"
    );
}
