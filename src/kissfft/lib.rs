extern mod extra;

use extra::complex;
use std::ptr;
use std::libc::{c_int, size_t};
use std::vec;
use std::comm;
use std::cast;
use std::task;

#[link_args = "-lkissfft"] extern {}

extern "C" {
	fn kiss_fft_alloc(nfft: c_int, inverse_fft: c_int, mem: *u8, lenmem: *size_t) -> *mut ~[u8];
	fn kiss_fft(cfg: *mut ~[u8], fin: *complex::Cmplx<f64>, mut fout: *mut complex::Cmplx<f64>);
	fn kiss_fft_cleanup();
}

fn kissFFTWorker(cfg: *mut ~[u8], fin: *complex::Cmplx<f64>, fout: *mut complex::Cmplx<f64>){
	unsafe {
	kiss_fft(cfg, fin, fout);
	}
}

pub fn kissFFT(din: ~[complex::Cmplx<f64>]) -> ~[complex::Cmplx<f64>] {
	let len = din.len();
	let mut fout: ~[complex::Cmplx<f64>] = ~[];
	fout.reserve(len);
	unsafe {
		vec::raw::set_len(&mut fout, len);
		let kiss_fft_cfg: *mut ~[u8] = kiss_fft_alloc(len as i32, 0, ptr::null(), ptr::null());
		kissFFTWorker(kiss_fft_cfg, vec::raw::to_ptr(din), vec::raw::to_mut_ptr(fout));
		kiss_fft_cleanup();
	}
	return fout;
}
pub fn kissiFFT(din: ~[complex::Cmplx<f64>]) -> ~[complex::Cmplx<f64>] {
	let len = din.len();
	let mut fout: ~[complex::Cmplx<f64>] = ~[];
	fout.reserve(len);
	unsafe {
		vec::raw::set_len(&mut fout, len);
		let kiss_fft_cfg: *mut ~[u8] = kiss_fft_alloc(len as i32, 1, ptr::null(), ptr::null());
		kissFFTWorker(kiss_fft_cfg, vec::raw::to_ptr(din), vec::raw::to_mut_ptr(fout));
		kiss_fft_cleanup();
	}
	return fout;
}

pub fn buildFFTBlock(blockSize: u64, fwd: bool) -> (comm::Port<~[complex::Cmplx<f64>]>, comm::Chan<~[complex::Cmplx<f64>]>) {
	let (pin, cin): (comm::Port<~[complex::Complex64]>, comm::Chan<~[complex::Complex64]>) = comm::stream();
	let (pout, cout): (comm::Port<~[complex::Complex64]>, comm::Chan<~[complex::Complex64]>) = comm::stream();
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
		'fft : loop {
			let mut din = pin.recv();
			if (din.len() == 0) { break};
			assert_eq!(din.len(), blockSize as uint);
			unsafe {
			kissFFTWorker(cast::transmute(vec::raw::to_mut_ptr(kissFFTState)), vec::raw::to_ptr(din), vec::raw::to_mut_ptr(din));
			}
			cout.send(din.iter().map(|&x: &complex::Complex64| x.scale(1.0/(blockSize as f64))).collect());
		}
		unsafe { kiss_fft_cleanup(); }
	}
	return (pout, cin);
}
