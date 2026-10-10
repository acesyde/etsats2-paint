use super::*;

/// The blur of one opaque pixel in a `side` × `side` buffer, as fractions
/// of the pixel's coverage.
fn impulse(side: usize, sigma: f64) -> Vec<f64> {
    let mut data = vec![0u16; side * side];
    let c = side / 2;
    data[c * side + c] = 65535;
    blur(&mut data, side, side, box_radii(sigma));
    data.iter().map(|v| f64::from(*v) / 65535.0).collect()
}

#[test]
fn box_radii_add_up_to_the_gaussian() {
    assert_eq!(box_radii(0.0), [0, 0, 0]);
    for sigma in [1.0, 4.0, 10.0, 100.0] {
        let variance: f64 = box_radii(sigma)
            .iter()
            .map(|r| {
                let w = (2 * r + 1) as f64;
                (w * w - 1.0) / 12.0
            })
            .sum();
        assert!(
            (variance.sqrt() - sigma).abs() < 0.5,
            "sigma {sigma}: {}",
            variance.sqrt()
        );
    }
}

#[test]
fn the_blur_of_a_pixel_is_close_to_a_gaussian() {
    // An opaque pixel blurred over σ = 4 px, compared with a Gaussian of
    // the same σ, both scaled so the Gaussian's peak is 255.
    let (side, sigma) = (61, 4.0);
    let out = impulse(side, sigma);
    let total: f64 = out.iter().sum();
    assert!((total - 1.0).abs() < 0.01, "coverage kept: {total}");
    let peak = 1.0 / (2.0 * std::f64::consts::PI * sigma * sigma);
    let c = (side / 2) as f64;
    let mut worst = 0.0f64;
    for y in 0..side {
        for x in 0..side {
            let d2 = (x as f64 - c).powi(2) + (y as f64 - c).powi(2);
            let gauss = peak * (-d2 / (2.0 * sigma * sigma)).exp();
            worst = worst.max((out[y * side + x] - gauss).abs() / peak);
        }
    }
    // Three boxes stay within 5% of the Gaussian's peak.
    assert!(worst < 0.05, "worst difference {worst} of the peak");
    // As drawn: the shadow of one opaque pixel is within 2 levels of the
    // Gaussian everywhere.
    let mut worst_drawn = 0.0f64;
    for y in 0..side {
        for x in 0..side {
            let d2 = (x as f64 - c).powi(2) + (y as f64 - c).powi(2);
            let gauss = 255.0 * peak * (-d2 / (2.0 * sigma * sigma)).exp();
            worst_drawn = worst_drawn.max((255.0 * out[y * side + x] - gauss).abs());
        }
    }
    assert!(worst_drawn <= 2.0, "{worst_drawn}");
}

#[test]
fn blur_zero_changes_nothing() {
    let mut data: Vec<u16> = (0..64).map(|i| (i * 1000) as u16).collect();
    let before = data.clone();
    blur(&mut data, 8, 8, box_radii(0.0));
    assert_eq!(data, before);
}

#[test]
fn tint_is_premultiplied() {
    let tint = Tint::new(Rgba::rgb(200, 100, 0), 0.5);
    assert_eq!(tint.pixel(65535), [100, 50, 0, 128]);
    assert_eq!(tint.pixel(0), [0, 0, 0, 0]);
}

#[test]
fn the_blur_of_a_line_is_close_to_a_gaussian() {
    // An opaque vertical line: across it, a Gaussian whose peak is about
    // 25 levels, within 2 levels.
    let (w, h, sigma) = (81, 81, 4.0);
    let mut data = vec![0u16; w * h];
    for y in 0..h {
        data[y * w + w / 2] = 65535;
    }
    blur(&mut data, w, h, box_radii(sigma));
    let row = &data[(h / 2) * w..(h / 2 + 1) * w];
    let peak = 255.0 / ((2.0 * std::f64::consts::PI).sqrt() * sigma);
    for (x, v) in row.iter().enumerate() {
        let d = x as f64 - (w / 2) as f64;
        let gauss = peak * (-d * d / (2.0 * sigma * sigma)).exp();
        let level = f64::from(*v) / 257.0;
        assert!((level - gauss).abs() <= 2.0, "x {x}: {level} vs {gauss}");
    }
}

#[test]
fn layers_and_caches_go_to_worker_threads() {
    fn send<T: Send>() {}
    send::<DrawCache>();
    send::<ShadowLayer>();
}
