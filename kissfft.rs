extern mod extra;

use extra::complex;
use extra::time;
use std::ptr;
use std::libc::{c_int, size_t};
use std::vec;
use std::rand;
use std::comm;
use std::cast;
use std::task;

extern {
	fn kiss_fft_alloc(nfft: c_int, inverse_fft: c_int, mem: *u8, lenmem: *size_t) -> *mut ~[u8];
	fn kiss_fft(cfg: *mut ~[u8], fin: *complex::Cmplx<f32>, mut fout: *mut complex::Cmplx<f32>);
	fn kiss_fft_cleanup();
}

fn kissFFT(in: ~[complex::Cmplx<f32>]) -> ~[complex::Cmplx<f32>] {
	let len = in.len();
	let mut fout: ~[complex::Cmplx<f32>] = ~[];
	fout.reserve(len);
	unsafe {
		vec::raw::set_len(&mut fout, len);
		let kiss_fft_cfg: *mut ~[u8] = kiss_fft_alloc(len as i32, 0, ptr::null(), ptr::null());
		kiss_fft(kiss_fft_cfg, vec::raw::to_ptr(in), vec::raw::to_mut_ptr(fout));
		kiss_fft_cleanup();
	}
	return fout;
}

fn buildFFTBlock(blockSize: u64, fwd: bool) -> (comm::Port<~[complex::Cmplx<f32>]>, comm::Chan<~[complex::Cmplx<f32>]>) { 
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
			println(fmt!("%?", size));
			// pass pointer to buffer - *mut u8
			kiss_fft_alloc(blockSize as i32, fwd as i32, vec::raw::to_ptr(kissFFTState), &size);
		}
		loop {
			let mut in = pin.recv();
			if (in.len() == 0) { break};
			//out.reserve(blockSize as uint);
			unsafe {
				kiss_fft(cast::transmute(vec::raw::to_mut_ptr(kissFFTState)), vec::raw::to_ptr(in), vec::raw::to_mut_ptr(in));
			}
			cout.send(in);
		}
		unsafe { kiss_fft_cleanup(); }
	}
	return (pout, cin);
}

// impl Drop for kiss_fft_cfg -> kiss_fft_cleanup()
fn main () {
	let (p, c) = buildFFTBlock(4096, true);
		let mut fin: ~[complex::Cmplx<f32>] = ~[];
		for 4096.times {
			let r: f32 = rand::random();
			let i: f32 = rand::random();
			fin.push(complex::Cmplx {re: r, im: i});
		}
	for 1000.times {
		let b: u64 = time::precise_time_ns();
		c.send(fin.clone());
		let d = p.recv();
		//let d = kissFFT(fin);
		let a: u64 = time::precise_time_ns();	
		print(fmt!("%? ", (a-b)/1000));
	}
	c.send(~[]);
}

