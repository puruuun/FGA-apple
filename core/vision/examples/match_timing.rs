use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::{Duration, Instant};

use vision::{CvPattern, GrayImage};

fn read_gray(path: &Path) -> Result<GrayImage, Box<dyn std::error::Error>> {
    let mut reader = png::Decoder::new(BufReader::new(File::open(path)?)).read_info()?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("no buffer size")?];
    let info = reader.next_frame(&mut buffer)?;
    buffer.truncate(info.buffer_size());
    Ok(GrayImage::new(
        info.width as usize,
        info.height as usize,
        buffer,
    )?)
}

fn median(
    frame: &CvPattern,
    template: &CvPattern,
    runs: usize,
) -> Result<Duration, vision::VisionError> {
    let mut times = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        let found = frame.find(template, 0.8)?;
        times.push(start.elapsed());
        std::hint::black_box(found);
    }
    times.sort();
    Ok(times.get(runs / 2).copied().unwrap_or_default())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/synthetic/frames/frame_00.png");
    let frame = CvPattern::new(read_gray(&path)?);
    let gray = frame.crop(400, 200, 64, 64)?.image().clone();
    let mask_pixels = (0..64 * 64)
        .map(|i| if (i / 64 + i % 64) % 5 == 0 { 0 } else { 255 })
        .collect();
    let mask = GrayImage::new(64, 64, mask_pixels)?;
    let plain = CvPattern::new(gray.clone());
    let masked = CvPattern::with_mask(gray, mask)?;
    let region = frame.crop(370, 180, 300, 120)?;

    for (name, image, runs) in [("300x120", &region, 51), ("1565x720", &frame, 3)] {
        println!("{name} unmasked: {:?}", median(image, &plain, runs)?);
        println!("{name} masked: {:?}", median(image, &masked, runs)?);
    }
    Ok(())
}
