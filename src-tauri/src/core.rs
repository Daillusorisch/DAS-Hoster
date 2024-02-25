use rustfft::{num_complex::Complex, Fft, FftNum, FftPlanner};
use std::sync::Arc;

/// ## panic
/// panic when scratch couldn't match `process_with_scratch`
pub fn fft_warpper<T: FftNum>(
    target: &[T],
    fft: Arc<dyn Fft<T>>,
    scratch: Option<&mut [Complex<T>]>,
) -> Vec<Complex<T>> {
    let mut target_complex = target
        .iter()
        .map(|x| Complex::<T>::from(*x))
        .collect::<Vec<_>>();
    match scratch {
        Some(scratch) => fft.process_with_scratch(&mut target_complex, scratch),
        None => fft.process(&mut target_complex),
    };
    target_complex
}

pub fn fft<T: FftNum>(target: &[T]) -> Vec<T> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(target.len());
    fft_warpper(target, fft, None).get_re_part()
}

pub fn deconvolve1d(sequence: &[f64], kernel: &[f64]) -> Vec<f64> {
    conv_internel(sequence, kernel, ConvDirection::DeConv)
}

#[allow(dead_code)]
pub fn convolve1d<T: FftNum>(sequence: &[T], kernel: &[T]) -> Vec<T> {
    conv_internel(sequence, kernel, ConvDirection::Conv)
}

fn conv_internel<T: FftNum>(sequence: &[T], kernel: &[T], direction: ConvDirection) -> Vec<T> {
    let len = sequence.len() + kernel.len() - 1;
    let sequence_padded = sequence.pad(len);
    let kernel_padded = kernel.pad(len);

    let mut planner = FftPlanner::<T>::new();
    let fft = planner.plan_fft_forward(len);
    let ifft = planner.plan_fft_inverse(len);
    let mut scratch = vec![
        Complex {
            re: T::zero(),
            im: T::zero(),
        };
        len * 2
    ];

    let sequence_complex = fft_warpper(
        sequence_padded.as_slice(),
        Arc::clone(&fft),
        Some(&mut scratch),
    );

    let kernel_complex = fft_warpper(
        kernel_padded.as_slice(),
        Arc::clone(&fft),
        Some(&mut scratch),
    );

    let zip = sequence_complex.iter().zip(kernel_complex.iter());
    let mut result = match direction {
        ConvDirection::Conv => zip.map(|(s, k)| s * k).collect::<Vec<_>>(),
        ConvDirection::DeConv => zip.map(|(s, k)| s / k).collect::<Vec<_>>(),
    };
    ifft.process_with_scratch(result.as_mut_slice(), scratch.as_mut_slice());
    result.get_re_part().noramlize()
}

enum ConvDirection {
    Conv,
    DeConv,
}

trait Pad<T: FftNum> {
    fn pad(self, len: usize) -> Vec<T>;
}

impl<T: FftNum> Pad<T> for Vec<T> {
    fn pad(self, len: usize) -> Vec<T> {
        let mut padded = self;
        while padded.len() < len {
            padded.push(T::zero());
        }
        padded
    }
}

impl<T: FftNum> Pad<T> for &[T] {
    fn pad(self, len: usize) -> Vec<T> {
        let mut padded = self.to_vec();
        while padded.len() < len {
            padded.push(T::zero());
        }
        padded
    }
}

pub trait GetRePart<T: FftNum> {
    fn get_re_part(&self) -> Vec<T>;
}

impl<T: FftNum> GetRePart<T> for Vec<Complex<T>> {
    fn get_re_part(&self) -> Vec<T> {
        self.iter().map(|c| c.re).collect()
    }
}

trait Noramlize<T: FftNum> {
    fn noramlize(self) -> Self;
}

impl<T: FftNum> Noramlize<T> for Vec<T> {
    fn noramlize(self) -> Self {
        self.iter()
            .map(|&x| x / T::from_usize(self.len()).unwrap())
            .collect()
    }
}

mod tests {
    #[test]
    fn test_deconvole1d() {
        use super::{convolve1d, deconvolve1d};
        let sequence_orgin = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 20.0, 50.0,
        ];
        let kernel = vec![1.0, 2.0, 3.0];
        let sequence = convolve1d(sequence_orgin.as_slice(), kernel.as_slice());
        println!("{:?}", sequence);
        let result = deconvolve1d(sequence.as_slice(), kernel.as_slice());
        assert_eq!(result[0..12], sequence_orgin);
    }
}
