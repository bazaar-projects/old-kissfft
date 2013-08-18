extern mod extra;

use extra::complex;
use std::ptr;
use std::libc::{c_int, size_t};
use std::vec;
use std::comm;
use std::cast;
use std::task;

extern {
	fn kiss_fft_alloc(nfft: c_int, inverse_fft: c_int, mem: *u8, lenmem: *size_t) -> *mut ~[u8];
	fn kiss_fft(cfg: *mut ~[u8], fin: *complex::Cmplx<f32>, mut fout: *mut complex::Cmplx<f32>);
	fn kiss_fft_cleanup();
}

pub fn FFT(din: ~[complex::Cmplx<f32>]) -> ~[complex::Cmplx<f32>] {
	let len = din.len();
	let mut fout: ~[complex::Cmplx<f32>] = ~[];
	fout.reserve(len);
	unsafe {
		vec::raw::set_len(&mut fout, len);
		let kiss_fft_cfg: *mut ~[u8] = kiss_fft_alloc(len as i32, 0, ptr::null(), ptr::null());
		kiss_fft(kiss_fft_cfg, vec::raw::to_ptr(din), vec::raw::to_mut_ptr(fout));
		kiss_fft_cleanup();
	}
	return fout.iter().map(|&x| x.scale(1f32/(len as f32))).collect();
}

pub fn iFFT(din: ~[complex::Cmplx<f32>]) -> ~[complex::Cmplx<f32>] {
	let len = din.len();
	let mut fout: ~[complex::Cmplx<f32>] = ~[];
	fout.reserve(len);
	unsafe {
		vec::raw::set_len(&mut fout, len);
		let kiss_fft_cfg: *mut ~[u8] = kiss_fft_alloc(len as i32, 1, ptr::null(), ptr::null());
		kiss_fft(kiss_fft_cfg, vec::raw::to_ptr(din), vec::raw::to_mut_ptr(fout));
		kiss_fft_cleanup();
	}
	return fout.iter().map(|&x| x.scale(len as f32)).collect();
}

pub fn buildFFTBlock(blockSize: u64, fwd: bool) -> (comm::Port<~[complex::Cmplx<f32>]>, comm::Chan<~[complex::Cmplx<f32>]>) {
	let (pin, cin): (comm::Port<~[complex::Complex32]>, comm::Chan<~[complex::Complex32]>) = comm::stream();
	let (pout, cout): (comm::Port<~[complex::Complex32]>, comm::Chan<~[complex::Complex32]>) = comm::stream();
	do task::spawn_sched(task::SingleThreaded) {
		let mut kissFFTState: ~[u8] = ~[];
		unsafe {
			let size = 0;
			// get neecessary buffer size by reference &size;
			kiss_fft_alloc(blockSize as i32, fwd as i32, ptr::null(), &size);
			// reserve buffer
			kissFFTState.reserve(size as uint);
			vec::raw::set_len(&mut kissFFTState, size as uint);
			// pass pointer to buffer - *mut u8
			kiss_fft_alloc(blockSize as i32, fwd as i32, vec::raw::to_ptr(kissFFTState), &size);
		}
		loop {
			let mut din = pin.recv();
			if (din.len() == 0) { break};
			assert_eq!(din.len(), blockSize as uint);
			//out.reserve(blockSize as uint);
			unsafe {
				kiss_fft(cast::transmute(vec::raw::to_mut_ptr(kissFFTState)), vec::raw::to_ptr(din), vec::raw::to_mut_ptr(din));
			}
			cout.send(din.iter().map(|&x| x.scale(1.0/(blockSize as f32))).collect());
		}
		unsafe { kiss_fft_cleanup(); }
	}
	return (pout, cin);
}
