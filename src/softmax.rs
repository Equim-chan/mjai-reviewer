#[inline]
fn eq(l: f32, r: f32) -> bool {
    (l - r).abs() < f32::EPSILON
}

pub fn softmax(arr: &mut [f32], temperature: f32) {
    if arr.is_empty() {
        return;
    }

    if !eq(temperature, 1.) {
        arr.iter_mut().for_each(|x| *x /= temperature);
    }

    let max = arr
        .iter()
        .copied()
        .max_by(|l, r| l.total_cmp(r))
        .unwrap_or(f32::NEG_INFINITY);
    let sum: f32 = arr.iter().copied().map(|x| (x - max).exp()).sum();

    let offset = max + sum.ln();
    arr.iter_mut().for_each(|x| *x = (*x - offset).exp());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_softmax() {
        let mut arr = [1., 4.2, 0.6, 1.23, 4.3, 1.2, 2.5];
        softmax(&mut arr, 1.);
        let expected = [
            0.016590023,
            0.40699518,
            0.011120625,
            0.020880206,
            0.44979942,
            0.020263102,
            0.07435133,
        ];
        assert!(arr.into_iter().zip(expected).all(|(l, r)| eq(l, r)));
    }
}
