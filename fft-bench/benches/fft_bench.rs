use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fft::Fft;
use num::Complex;
use rand::distributions::Standard;
use rand::Rng;

fn pow2_f32(c: &mut Criterion) {
    let mut group = c.benchmark_group("FFT, f32, powers of 2");
    for size in (8..12).map(|x| 2usize.pow(x)) {
        let input = rand::thread_rng()
            .sample_iter(&Standard)
            .zip(rand::thread_rng().sample_iter(&Standard))
            .take(size)
            .map(|(x, y)| Complex::new(x, y))
            .collect::<Vec<_>>();

        // My FFT
        let mut my_fft = fft::Fft32::new(size);
        group.bench_with_input(BenchmarkId::new("Mine", size), &input, |b, i| {
            let mut input = Vec::new();
            input.extend_from_slice(i);
            b.iter(|| my_fft.fft_in_place(input.as_mut()))
        });

        // RustFFT
        let rustfft = rustfft::FFTplanner::<f32>::new(false).plan_fft(size);
        group.bench_with_input(BenchmarkId::new("RustFFT", size), &input, |b, i| {
            let mut input = Vec::new();
            input.extend_from_slice(i);
            let mut output = vec![Complex::default(); input.len()];
            b.iter(|| rustfft.process(input.as_mut(), output.as_mut()))
        });

        use mkl_fft::plan::C2CPlan;
        let mut mkl_fft = mkl_fft::plan::C2CPlan32::aligned(
            &[size],
            mkl_fft::types::Sign::Forward,
            mkl_fft::types::Flag::Measure,
        )
        .unwrap();
        group.bench_with_input(BenchmarkId::new("Intel MKL", size), &input, |b, i| {
            let mut input = mkl_fft::array::AlignedVec::new(size);
            for (i, o) in i.iter().zip(input.as_slice_mut().iter_mut()) {
                *o = mkl_fft::types::c32::new(i.re, i.im);
            }
            let mut output = mkl_fft::array::AlignedVec::new(size);
            b.iter(|| {
                mkl_fft
                    .c2c(input.as_slice_mut(), output.as_slice_mut())
                    .unwrap()
            })
        });
    }
    group.finish();
}

criterion_group!(benches, pow2_f32);
criterion_main!(benches);
