use crate::gray_image::GrayImage;
use crate::score_map::ScoreMap;

const CHUNK: usize = 16384;

fn dot(image: &[u8], kernel: &[u8]) -> u64 {
    image
        .chunks(CHUNK)
        .zip(kernel.chunks(CHUNK))
        .map(|(a, b)| {
            let sum: u32 = a
                .iter()
                .zip(b)
                .map(|(&p, &q)| u32::from(p) * u32::from(q))
                .sum();
            u64::from(sum)
        })
        .sum()
}

fn result_size(image: &GrayImage, templ: &GrayImage) -> (usize, usize) {
    (
        image.width() - templ.width() + 1,
        image.height() - templ.height() + 1,
    )
}

const PARALLEL_WORK: usize = 1 << 22;

fn by_rows<F>(columns: usize, rows: usize, work_per_row: usize, fill: F) -> Vec<u64>
where
    F: Fn(usize, &mut [u64]) + Sync,
{
    let mut out = vec![0u64; columns * rows];
    let columns = columns.max(1);
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let work = work_per_row.saturating_mul(rows);
    if threads < 2 || rows < 2 || work < PARALLEL_WORK {
        for (y, out_row) in out.chunks_mut(columns).enumerate() {
            fill(y, out_row);
        }
        return out;
    }
    let band = rows.div_ceil(threads.min(rows));
    std::thread::scope(|scope| {
        for (index, chunk) in out.chunks_mut(band * columns).enumerate() {
            let fill = &fill;
            scope.spawn(move || {
                for (offset, out_row) in chunk.chunks_mut(columns).enumerate() {
                    fill(index * band + offset, out_row);
                }
            });
        }
    });
    out
}

fn cross_corr(
    image: &[u8],
    image_width: usize,
    kernel: &[u8],
    width: usize,
    columns: usize,
    rows: usize,
) -> Vec<u64> {
    by_rows(columns, rows, columns * kernel.len(), |y, out_row| {
        for (r, kernel_row) in kernel.chunks(width.max(1)).enumerate() {
            let start = (y + r) * image_width;
            let image_row = image.get(start..start + image_width).unwrap_or(&[]);
            for (x, cell) in out_row.iter_mut().enumerate() {
                if let Some(window) = image_row.get(x..x + width) {
                    *cell += dot(window, kernel_row);
                }
            }
        }
    })
}

fn kept_runs(keep: &[u8], width: usize) -> Vec<(usize, usize, usize)> {
    let mut runs = Vec::new();
    for (r, row) in keep.chunks(width.max(1)).enumerate() {
        let mut start = None;
        for (c, &k) in row.iter().chain(std::iter::once(&0)).enumerate() {
            match (k, start) {
                (1, None) => start = Some(c),
                (0, Some(s)) => {
                    runs.push((r, s, c));
                    start = None;
                }
                _ => {}
            }
        }
    }
    runs
}

fn run_sums(
    prefix: &[u64],
    stride: usize,
    runs: &[(usize, usize, usize)],
    columns: usize,
    rows: usize,
) -> Vec<u64> {
    by_rows(columns, rows, columns * runs.len(), |y, out_row| {
        for &(r, start, end) in runs {
            let base = (y + r) * stride;
            let Some(row) = prefix.get(base..base + stride) else {
                continue;
            };
            for (cell, (high, low)) in out_row.iter_mut().zip(
                row.get(end..)
                    .unwrap_or(&[])
                    .iter()
                    .zip(row.get(start..).unwrap_or(&[])),
            ) {
                *cell += high - low;
            }
        }
    })
}

fn row_prefix(image: &GrayImage, value: impl Fn(u8) -> u64) -> Vec<u64> {
    let stride = image.width() + 1;
    let mut prefix = Vec::with_capacity(stride * image.height());
    for y in 0..image.height() {
        let mut total = 0u64;
        prefix.push(0);
        for &p in image.row(y) {
            total += value(p);
            prefix.push(total);
        }
    }
    prefix
}

fn window_sums(image: &GrayImage, width: usize, height: usize) -> (Vec<u64>, Vec<u64>) {
    let stride = image.width() + 1;
    let mut sum = vec![0u64; stride * (image.height() + 1)];
    let mut square = vec![0u64; stride * (image.height() + 1)];
    for y in 0..image.height() {
        let mut row_sum = 0u64;
        let mut row_square = 0u64;
        for (x, &p) in image.row(y).iter().enumerate() {
            row_sum += u64::from(p);
            row_square += u64::from(p) * u64::from(p);
            let at = (y + 1) * stride + x + 1;
            sum[at] = sum[at - stride] + row_sum;
            square[at] = square[at - stride] + row_square;
        }
    }
    let columns = image.width() + 1 - width;
    let rows = image.height() + 1 - height;
    let window = |table: &[u64], x: usize, y: usize| {
        table[(y + height) * stride + x + width] + table[y * stride + x]
            - table[y * stride + x + width]
            - table[(y + height) * stride + x]
    };
    let mut sums = Vec::with_capacity(columns * rows);
    let mut squares = Vec::with_capacity(columns * rows);
    for y in 0..rows {
        for x in 0..columns {
            sums.push(window(&sum, x, y));
            squares.push(window(&square, x, y));
        }
    }
    (sums, squares)
}

pub(crate) fn ccoeff_normed(image: &GrayImage, templ: &GrayImage) -> ScoreMap {
    let (columns, rows) = result_size(image, templ);
    let area = templ.pixels().len() as f64;
    let inv_area = 1.0 / area;

    let templ_sum: u64 = templ.pixels().iter().map(|&p| u64::from(p)).sum();
    let templ_square: u64 = templ
        .pixels()
        .iter()
        .map(|&p| u64::from(p) * u64::from(p))
        .sum();
    let templ_mean = templ_sum as f64 * inv_area;
    let templ_sdv = (templ_square as f64 * inv_area - templ_mean * templ_mean)
        .max(0.0)
        .sqrt();
    let templ_variance = templ_sdv * templ_sdv;
    if templ_variance < f64::EPSILON {
        return ScoreMap::new(columns, rows, vec![1.0; columns * rows]);
    }
    let templ_norm = templ_variance.sqrt() / inv_area.sqrt();

    let correlation = cross_corr(
        image.pixels(),
        image.width(),
        templ.pixels(),
        templ.width(),
        columns,
        rows,
    );
    let (sums, squares) = window_sums(image, templ.width(), templ.height());

    let values = correlation
        .iter()
        .zip(sums.iter().zip(&squares))
        .map(|(&corr, (&sum, &square))| {
            let sum = sum as f64;
            let wnd_sum2 = square as f64;
            let wnd_mean2 = sum * sum * inv_area;
            let mut num = corr as f64 - sum * templ_mean;
            let diff2 = (wnd_sum2 - wnd_mean2).max(0.0);
            let t = if diff2 <= (10.0 * f64::from(f32::EPSILON) * wnd_sum2).min(0.5) {
                0.0
            } else {
                diff2.sqrt() * templ_norm
            };
            if num.abs() < t {
                num /= t;
            } else if num.abs() < t * 1.125 {
                num = if num > 0.0 { 1.0 } else { -1.0 };
            } else {
                num = 0.0;
            }
            num as f32
        })
        .collect();
    ScoreMap::new(columns, rows, values)
}

pub(crate) fn ccoeff_normed_masked(
    image: &GrayImage,
    templ: &GrayImage,
    mask: &GrayImage,
) -> ScoreMap {
    let (columns, rows) = result_size(image, templ);
    let keep: Vec<u8> = mask.pixels().iter().map(|&m| u8::from(m > 0)).collect();
    let kept_templ: Vec<u8> = templ
        .pixels()
        .iter()
        .zip(&keep)
        .map(|(&t, &k)| t * k)
        .collect();
    let count = i128::from(keep.iter().map(|&k| u64::from(k)).sum::<u64>());
    let templ_sum = i128::from(kept_templ.iter().map(|&t| u64::from(t)).sum::<u64>());
    let templ_square = i128::from(
        kept_templ
            .iter()
            .map(|&t| u64::from(t) * u64::from(t))
            .sum::<u64>(),
    );
    let templ_spread = (count * templ_square - templ_sum * templ_sum) as f64;

    let width = templ.width();
    let correlation = cross_corr(
        image.pixels(),
        image.width(),
        &kept_templ,
        width,
        columns,
        rows,
    );
    let runs = kept_runs(&keep, width);
    let stride = image.width() + 1;
    let sums = run_sums(&row_prefix(image, u64::from), stride, &runs, columns, rows);
    let square_sums = run_sums(
        &row_prefix(image, |p| u64::from(p) * u64::from(p)),
        stride,
        &runs,
        columns,
        rows,
    );
    let values = correlation
        .iter()
        .zip(sums.iter().zip(&square_sums))
        .map(|(&corr, (&sum, &square))| {
            let corr = i128::from(corr);
            let sum = i128::from(sum);
            let square = i128::from(square);
            let num = (count * corr - templ_sum * sum) as f64;
            let image_spread = (count * square - sum * sum) as f64;
            (num / (image_spread.sqrt() * templ_spread.sqrt())) as f32
        })
        .collect();
    ScoreMap::new(columns, rows, values)
}
