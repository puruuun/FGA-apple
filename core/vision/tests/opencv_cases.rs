#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use vision::{CvPattern, GrayImage, ScoreMap};

const TOLERANCE: f64 = 0.01;

const FLAT_BESIDE_TEXTURE: [[f64; 14]; 6] = [
    [
        0.0,
        0.0,
        0.0,
        0.0,
        -0.1782856583595276,
        0.09892870485782623,
        0.2572915852069855,
        0.21187911927700043,
        -0.030991623178124428,
        0.007454918697476387,
        0.033730313181877136,
        0.004519108682870865,
        0.30158376693725586,
        -0.15747246146202087,
    ],
    [
        0.0,
        0.0,
        0.0,
        0.0,
        -0.14876797795295715,
        0.17123062908649445,
        -0.08441118150949478,
        -0.036742158234119415,
        0.3209978938102722,
        0.11932360380887985,
        0.05024413391947746,
        0.11808639764785767,
        -0.016844164580106735,
        -0.0910797193646431,
    ],
    [
        0.0,
        0.0,
        0.0,
        0.0,
        0.08274655044078827,
        -0.17322520911693573,
        0.2612544596195221,
        0.12502728402614594,
        0.07448668032884598,
        0.012404058128595352,
        -0.16930752992630005,
        -0.006569625809788704,
        0.04875073581933975,
        0.12537744641304016,
    ],
    [
        0.0,
        0.0,
        0.0,
        0.0,
        0.04092642664909363,
        0.07939982414245605,
        0.16075332462787628,
        0.0828227624297142,
        0.05183935537934303,
        0.1866106539964676,
        0.054034192115068436,
        0.011593527160584927,
        -0.031634002923965454,
        0.03340383991599083,
    ],
    [
        0.0,
        0.0,
        0.0,
        0.0,
        -0.00019465648801997304,
        -0.031803615391254425,
        0.03222721442580223,
        0.1926259547472,
        0.05899786204099655,
        0.1926330029964447,
        0.025071803480386734,
        -0.03204994648694992,
        0.36893007159233093,
        -0.12923210859298706,
    ],
    [
        0.0,
        0.0,
        0.0,
        0.0,
        -0.18033087253570557,
        0.19663245975971222,
        0.19883425533771515,
        0.09608309715986252,
        0.18440870940685272,
        0.049598366022109985,
        0.11255775392055511,
        -0.1646054983139038,
        -0.15750789642333984,
        -0.11042826622724533,
    ],
];
const PLAIN: [[f64; 6]; 5] = [
    [
        -0.127741277217865,
        -0.32282546162605286,
        -0.0330592542886734,
        0.15459726750850677,
        -0.0977039709687233,
        -0.2759527862071991,
    ],
    [
        -0.1407673954963684,
        -0.10218919068574905,
        -0.3058985471725464,
        0.4846629500389099,
        0.08578632771968842,
        -0.1653517186641693,
    ],
    [
        0.14937885105609894,
        -0.08174554258584976,
        0.4147212505340576,
        -0.10588593035936356,
        -0.31537947058677673,
        0.3108377754688263,
    ],
    [
        -0.15980735421180725,
        -0.09894365817308426,
        0.09316597133874893,
        -0.3053090274333954,
        -0.12056984752416611,
        0.1544141322374344,
    ],
    [
        -0.27117475867271423,
        -0.1247788667678833,
        -0.04622180387377739,
        0.15253272652626038,
        -0.3526313304901123,
        0.06709345430135727,
    ],
];

const MASKED: [[f64; 6]; 5] = [
    [
        -0.12364422529935837,
        -0.3241882026195526,
        -0.0357690155506134,
        0.157699853181839,
        -0.09345695376396179,
        -0.29622340202331543,
    ],
    [
        -0.14548149704933167,
        -0.11720749735832214,
        -0.3293522298336029,
        0.5104631781578064,
        0.10812138766050339,
        -0.16725416481494904,
    ],
    [
        0.15366989374160767,
        -0.07369134575128555,
        0.4148745834827423,
        -0.10309841483831406,
        -0.3191821277141571,
        0.30895760655403137,
    ],
    [
        -0.15761502087116241,
        -0.10570356249809265,
        0.10163380205631256,
        -0.30533045530319214,
        -0.1313697248697281,
        0.1511438637971878,
    ],
    [
        -0.2771589457988739,
        -0.15007424354553223,
        -0.06215387210249901,
        0.1483418196439743,
        -0.3609708249568939,
        0.060021497309207916,
    ],
];

const FLAT_LEFT_SANITIZED: [[f64; 12]; 6] = [
    [
        0.1145968958735466,
        0.2570686936378479,
        0.0,
        0.28184640407562256,
        0.05265262722969055,
        0.04757816717028618,
        -0.0637771338224411,
        0.18076887726783752,
        -0.07195843011140823,
        0.07285048812627792,
        -0.4859433174133301,
        -0.32238176465034485,
    ],
    [
        -0.08362475782632828,
        0.25087428092956543,
        -0.15486066043376923,
        0.17344394326210022,
        -0.2849436402320862,
        0.03404376655817032,
        -0.2069484293460846,
        -0.20732587575912476,
        -0.5066310167312622,
        -0.22398853302001953,
        0.11503676325082779,
        0.3269287645816803,
    ],
    [
        -0.17344394326210022,
        -0.22299936413764954,
        0.03406934812664986,
        0.0,
        0.09291639924049377,
        0.18513567745685577,
        0.24654215574264526,
        0.522977888584137,
        0.004750144202262163,
        0.2849981486797333,
        -0.05525058135390282,
        -0.01568540558218956,
    ],
    [
        0.3437906801700592,
        0.17344394326210022,
        0.34069347381591797,
        -0.01238885335624218,
        -0.1053052544593811,
        0.1956879049539566,
        -0.013821134343743324,
        -0.1554533690214157,
        -0.27622365951538086,
        -0.20657961070537567,
        -0.19384300708770752,
        -0.18544206023216248,
    ],
    [
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        -0.18101657927036285,
        -0.26955732703208923,
        -0.4256092607975006,
        -0.08322639018297195,
        0.16268602013587952,
        0.4409657418727875,
        0.5042142271995544,
    ],
    [
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        -0.037762127816677094,
        -0.013391576707363129,
        0.09536336362361908,
        -0.024473117664456367,
        -0.15870769321918488,
        0.13728849589824677,
        -0.19549252092838287,
    ],
];

fn noise(seed: u32, width: usize, height: usize) -> Vec<u8> {
    let mut state = seed;
    (0..width * height)
        .map(|_| {
            state = state.wrapping_mul(1103515245).wrapping_add(12345) & 0x7fff_ffff;
            (state >> 16) as u8
        })
        .collect()
}

fn gray(width: usize, height: usize, pixels: Vec<u8>) -> GrayImage {
    GrayImage::new(width, height, pixels).unwrap()
}

fn masked(templ: GrayImage, hidden: &[(usize, usize)]) -> CvPattern {
    let mut mask = vec![255; templ.width() * templ.height()];
    for &(x, y) in hidden {
        mask[y * templ.width() + x] = 0;
    }
    let mask = gray(templ.width(), templ.height(), mask);
    CvPattern::with_mask(templ, mask).unwrap()
}

fn map(image: &CvPattern, templ: &CvPattern) -> ScoreMap {
    image.match_template(templ).unwrap().unwrap()
}

fn assert_map<const W: usize, const H: usize>(actual: &ScoreMap, expected: &[[f64; W]; H]) {
    assert_eq!((actual.width(), actual.height()), (W, H));
    for (y, row) in expected.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            let got = f64::from(actual.get(x, y).unwrap());
            assert!(
                (got - value).abs() <= TOLERANCE,
                "({x}, {y}): {got} expected {value}"
            );
        }
    }
}

fn assert_all(actual: &ScoreMap, value: f32) {
    assert!(
        actual.values().iter().all(|&v| v == value),
        "{:?}",
        actual.values()
    );
}

#[test]
fn unmasked_map_matches_opencv() {
    let image = CvPattern::new(gray(9, 7, noise(3, 9, 7)));
    let templ = CvPattern::new(gray(4, 3, noise(4, 4, 3)));
    assert_map(&map(&image, &templ), &PLAIN);
}

#[test]
fn masked_map_matches_opencv() {
    let image = CvPattern::new(gray(9, 7, noise(3, 9, 7)));
    let templ = masked(gray(4, 3, noise(4, 4, 3)), &[(1, 0)]);
    assert_map(&map(&image, &templ), &MASKED);
}

#[test]
fn every_nonzero_mask_value_counts_as_kept() {
    let image = CvPattern::new(gray(9, 7, noise(3, 9, 7)));
    let mask = gray(4, 3, vec![1, 0, 128, 255, 2, 64, 200, 1, 255, 3, 7, 90]);
    let templ = CvPattern::with_mask(gray(4, 3, noise(4, 4, 3)), mask).unwrap();
    assert_map(&map(&image, &templ), &MASKED);
}

#[test]
fn masked_windows_that_opencv_turns_into_nan_or_infinity_become_zero() {
    let mut pixels = noise(18, 16, 10);
    for row in pixels.chunks_mut(16) {
        row[..9].fill(37);
    }
    pixels[3 * 16 + 4] = 36;
    let image = CvPattern::new(gray(16, 10, pixels));
    let templ = masked(gray(5, 5, noise(1018, 5, 5)), &[(1, 1), (2, 3)]);
    let actual = map(&image, &templ);
    assert_map(&actual, &FLAT_LEFT_SANITIZED);
    for (x, y) in [
        (2, 0),
        (3, 2),
        (0, 4),
        (1, 4),
        (2, 4),
        (3, 4),
        (4, 4),
        (0, 5),
        (1, 5),
        (2, 5),
        (3, 5),
        (4, 5),
    ] {
        assert_eq!(actual.get(x, y), Some(0.0), "({x}, {y})");
    }
}

#[test]
fn flat_inputs_match_opencv() {
    let image = gray(9, 7, noise(3, 9, 7));
    let templ = gray(4, 3, noise(4, 4, 3));
    let flat_image = gray(9, 7, vec![90; 63]);
    let flat_templ = gray(4, 3, vec![200; 12]);
    let cases = [
        (&flat_image, &templ, 0.0, 0.0),
        (&image, &flat_templ, 1.0, 0.0),
        (&flat_image, &flat_templ, 1.0, 0.0),
    ];
    for (image, templ, plain, with_mask) in cases {
        let image = CvPattern::new(image.clone());
        assert_all(&map(&image, &CvPattern::new(templ.clone())), plain);
        assert_all(&map(&image, &masked(templ.clone(), &[(1, 0)])), with_mask);
    }
    let hidden: Vec<(usize, usize)> = (0..3).flat_map(|y| (0..4).map(move |x| (x, y))).collect();
    let image = CvPattern::new(image);
    assert_all(&map(&image, &masked(templ, &hidden)), 0.0);
}

fn several() -> (CvPattern, GrayImage) {
    let templ = noise(7, 6, 6);
    let mut pixels = noise(11, 40, 30);
    for (x, y) in [(3, 4), (25, 2), (14, 20)] {
        for r in 0..6 {
            let start = (y + r) * 40 + x;
            pixels[start..start + 6].copy_from_slice(&templ[r * 6..r * 6 + 6]);
        }
    }
    pixels[22 * 40 + 16] = pixels[22 * 40 + 16].wrapping_add(128);
    (CvPattern::new(gray(40, 30, pixels)), gray(6, 6, templ))
}

fn assert_found(image: &CvPattern, templ: &CvPattern, expected: &[(i32, i32, f64)]) {
    let actual: Vec<(i32, i32, f64)> = image
        .find_matches(templ, 0.8)
        .unwrap()
        .take(10)
        .map(|m| (m.x, m.y, m.score))
        .collect();
    assert_eq!(actual.len(), expected.len(), "{actual:?}");
    for (a, e) in actual.iter().zip(expected) {
        assert_eq!((a.0, a.1), (e.0, e.1), "{actual:?}");
        assert!((a.2 - e.2).abs() <= TOLERANCE, "{actual:?}");
    }
}

#[test]
fn several_separated_matches_come_out_in_opencv_order() {
    let (image, templ) = several();
    assert_found(
        &image,
        &CvPattern::new(templ.clone()),
        &[(25, 2, 1.0), (3, 4, 1.0), (14, 20, 0.9585978984832764)],
    );
    assert_found(
        &image,
        &masked(templ, &[(0, 0), (5, 5), (3, 2)]),
        &[(25, 2, 1.0), (3, 4, 1.0), (14, 20, 0.955262303352356)],
    );
    let found = image
        .find(&CvPattern::new(several().1), 0.8)
        .unwrap()
        .unwrap();
    assert_eq!((found.x, found.y, found.width, found.height), (25, 2, 6, 6));
    assert_eq!(image.exists(&CvPattern::new(several().1), 0.97), Ok(true));
    assert_eq!(image.exists(&CvPattern::new(several().1), 1.01), Ok(false));
}

fn near_flat(seed: u32, width: usize, height: usize, base: u8, every: u8) -> GrayImage {
    let pixels = noise(seed, width, height)
        .into_iter()
        .map(|z| if z % every == 0 { base + 1 } else { base })
        .collect();
    gray(width, height, pixels)
}

fn pearson(image: &GrayImage, templ: &GrayImage, x: usize, y: usize) -> f64 {
    let n = templ.pixels().len() as i128;
    let (mut si, mut st, mut sii, mut stt, mut sit) = (0i128, 0i128, 0i128, 0i128, 0i128);
    for ty in 0..templ.height() {
        for tx in 0..templ.width() {
            let i = i128::from(image.pixels()[(y + ty) * image.width() + x + tx]);
            let t = i128::from(templ.pixels()[ty * templ.width() + tx]);
            si += i;
            st += t;
            sii += i * i;
            stt += t * t;
            sit += i * t;
        }
    }
    let image_spread = n * sii - si * si;
    if image_spread == 0 {
        return 0.0;
    }
    let templ_spread = n * stt - st * st;
    (n * sit - si * st) as f64 / ((image_spread as f64).sqrt() * (templ_spread as f64).sqrt())
}

#[test]
fn near_flat_unmasked_windows_score_the_exact_correlation() {
    let image = near_flat(11, 60, 30, 200, 23);
    let templ = near_flat(61, 12, 12, 90, 5);
    let actual = map(
        &CvPattern::new(image.clone()),
        &CvPattern::new(templ.clone()),
    );
    let mut largest: f64 = 0.0;
    for y in 0..actual.height() {
        for x in 0..actual.width() {
            let expected = pearson(&image, &templ, x, y);
            let got = f64::from(actual.get(x, y).unwrap());
            assert!(
                (got - expected).abs() <= 1e-6,
                "({x}, {y}): {got} expected {expected}"
            );
            largest = largest.max(expected.abs());
        }
    }
    assert!(largest > 0.1);
}

#[test]
fn constant_templates_over_constant_images_match_opencv() {
    let side = 40;
    let cases: [(usize, usize, u8, u8, f32); 13] = [
        (7, 7, 200, 200, 0.0),
        (7, 7, 200, 0, 0.0),
        (7, 7, 200, 255, 0.0),
        (7, 7, 254, 254, 0.0),
        (23, 11, 254, 254, 0.0),
        (23, 11, 254, 255, 0.0),
        (11, 23, 2, 2, 0.0),
        (23, 11, 2, 2, 0.0),
        (7, 7, 13, 255, 0.0),
        (7, 7, 1, 1, 1.0),
        (30, 30, 1, 1, 1.0),
        (1, 1, 1, 1, 1.0),
        (12, 12, 77, 0, 1.0),
    ];
    for (width, height, templ_value, image_value, expected) in cases {
        let image = CvPattern::new(gray(side, side, vec![image_value; side * side]));
        let templ = CvPattern::new(gray(width, height, vec![templ_value; width * height]));
        let actual = map(&image, &templ);
        assert_eq!(
            (actual.width(), actual.height()),
            (side + 1 - width, side + 1 - height)
        );
        assert!(
            actual
                .values()
                .iter()
                .all(|&v| (f64::from(v) - f64::from(expected)).abs() <= TOLERANCE),
            "{width}x{height} of {templ_value} over {image_value}: {:?}",
            actual.max_loc()
        );
    }
}

#[test]
fn flat_windows_beside_texture_score_zero_as_in_opencv() {
    let mut pixels = noise(21, 20, 12);
    for row in pixels.chunks_mut(20) {
        row[..10].fill(200);
    }
    let image = CvPattern::new(gray(20, 12, pixels));
    let templ = CvPattern::new(gray(7, 7, noise(22, 7, 7)));
    let actual = map(&image, &templ);
    assert_map(&actual, &FLAT_BESIDE_TEXTURE);
    for y in 0..actual.height() {
        for x in 0..4 {
            assert_eq!(actual.get(x, y), Some(0.0), "({x}, {y})");
        }
    }
}

#[test]
fn a_constant_template_scores_zero_over_flat_and_textured_windows_as_in_opencv() {
    let mut pixels = noise(21, 40, 30);
    for row in pixels.chunks_mut(40) {
        row[..20].fill(200);
    }
    let image = CvPattern::new(gray(40, 30, pixels));
    let templ = CvPattern::new(gray(7, 7, vec![200; 49]));
    let actual = map(&image, &templ);
    assert_eq!((actual.width(), actual.height()), (34, 24));
    assert!(
        actual
            .values()
            .iter()
            .all(|&v| f64::from(v).abs() <= TOLERANCE),
        "{:?}",
        actual.max_loc()
    );
}
