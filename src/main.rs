#![crate_type = "staticlib"]
#![feature(plugin, start, core_intrinsics)]
#![no_std]
#![plugin(macro_platformtree)]

extern crate zinc;

use zinc::hal::spi::Spi;
// use zinc::hal::cortex_m4::fpu;
use zinc::hal::timer::Timer;
use zinc::drivers::chario::CharIO;
use zinc::hal::tiva_c;

const GAMMA_MAP: [u8; 256] = [
	0, 0, 0, 0, 0, 1, 1, 1, 1, 1,
	1, 1, 1, 1, 2, 2, 2, 2, 2, 2,
	2, 2, 2, 3, 3, 3, 3, 3, 3, 3,
	3, 4, 4, 4, 4, 4, 4, 5, 5, 5,
	5, 5, 6, 6, 6, 6, 6, 7, 7, 7,
	7, 8, 8, 8, 8, 9, 9, 9, 10, 10,
	10, 10, 11, 11, 11, 12, 12, 12, 13, 13,
	13, 14, 14, 15, 15, 15, 16, 16, 17, 17,
	17, 18, 18, 19, 19, 20, 20, 21, 21, 22,
	22, 23, 23, 24, 24, 25, 25, 26, 26, 27,
	28, 28, 29, 29, 30, 31, 31, 32, 32, 33,
	34, 34, 35, 36, 37, 37, 38, 39, 39, 40,
	41, 42, 43, 43, 44, 45, 46, 47, 47, 48,
	49, 50, 51, 52, 53, 54, 54, 55, 56, 57,
	58, 59, 60, 61, 62, 63, 64, 65, 66, 67,
	68, 70, 71, 72, 73, 74, 75, 76, 77, 79,
	80, 81, 82, 83, 85, 86, 87, 88, 90, 91,
	92, 94, 95, 96, 98, 99, 100, 102, 103, 105,
	106, 108, 109, 110, 112, 113, 115, 116, 118, 120,
	121, 123, 124, 126, 128, 129, 131, 132, 134, 136,
	138, 139, 141, 143, 145, 146, 148, 150, 152, 154,
	155, 157, 159, 161, 163, 165, 167, 169, 171, 173,
	175, 177, 179, 181, 183, 185, 187, 189, 191, 193,
	196, 198, 200, 202, 204, 207, 209, 211, 214, 216,
	218, 220, 223, 225, 228, 230, 232, 235, 237, 240,
	242, 245, 247, 250, 252, 255
];

const SINE_TABLE: [u8; 256] = [
	0x80, 0x83, 0x86, 0x89, 0x8C, 0x90, 0x93, 0x96,
	0x99, 0x9C, 0x9F, 0xA2, 0xA5, 0xA8, 0xAB, 0xAE,
	0xB1, 0xB3, 0xB6, 0xB9, 0xBC, 0xBF, 0xC1, 0xC4,
	0xC7, 0xC9, 0xCC, 0xCE, 0xD1, 0xD3, 0xD5, 0xD8,
	0xDA, 0xDC, 0xDE, 0xE0, 0xE2, 0xE4, 0xE6, 0xE8,
	0xEA, 0xEB, 0xED, 0xEF, 0xF0, 0xF1, 0xF3, 0xF4,
	0xF5, 0xF6, 0xF8, 0xF9, 0xFA, 0xFA, 0xFB, 0xFC,
	0xFD, 0xFD, 0xFE, 0xFE, 0xFE, 0xFF, 0xFF, 0xFF,
	0xFF, 0xFF, 0xFF, 0xFF, 0xFE, 0xFE, 0xFE, 0xFD,
	0xFD, 0xFC, 0xFB, 0xFA, 0xFA, 0xF9, 0xF8, 0xF6,
	0xF5, 0xF4, 0xF3, 0xF1, 0xF0, 0xEF, 0xED, 0xEB,
	0xEA, 0xE8, 0xE6, 0xE4, 0xE2, 0xE0, 0xDE, 0xDC,
	0xDA, 0xD8, 0xD5, 0xD3, 0xD1, 0xCE, 0xCC, 0xC9,
	0xC7, 0xC4, 0xC1, 0xBF, 0xBC, 0xB9, 0xB6, 0xB3,
	0xB1, 0xAE, 0xAB, 0xA8, 0xA5, 0xA2, 0x9F, 0x9C,
	0x99, 0x96, 0x93, 0x90, 0x8C, 0x89, 0x86, 0x83,
	0x80, 0x7D, 0x7A, 0x77, 0x74, 0x70, 0x6D, 0x6A,
	0x67, 0x64, 0x61, 0x5E, 0x5B, 0x58, 0x55, 0x52,
	0x4F, 0x4D, 0x4A, 0x47, 0x44, 0x41, 0x3F, 0x3C,
	0x39, 0x37, 0x34, 0x32, 0x2F, 0x2D, 0x2B, 0x28,
	0x26, 0x24, 0x22, 0x20, 0x1E, 0x1C, 0x1A, 0x18,
	0x16, 0x15, 0x13, 0x11, 0x10, 0x0F, 0x0D, 0x0C,
	0x0B, 0x0A, 0x08, 0x07, 0x06, 0x06, 0x05, 0x04,
	0x03, 0x03, 0x02, 0x02, 0x02, 0x01, 0x01, 0x01,
	0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x03,
	0x03, 0x04, 0x05, 0x06, 0x06, 0x07, 0x08, 0x0A,
	0x0B, 0x0C, 0x0D, 0x0F, 0x10, 0x11, 0x13, 0x15,
	0x16, 0x18, 0x1A, 0x1C, 0x1E, 0x20, 0x22, 0x24,
	0x26, 0x28, 0x2B, 0x2D, 0x2F, 0x32, 0x34, 0x37,
	0x39, 0x3C, 0x3F, 0x41, 0x44, 0x47, 0x4A, 0x4D,
	0x4F, 0x52, 0x55, 0x58, 0x5B, 0x5E, 0x61, 0x64,
	0x67, 0x6A, 0x6D, 0x70, 0x74, 0x77, 0x7A, 0x7D
];

platformtree!(
	tiva_c@mcu {
		// Tiva C ends up with an 80MHz clock from 16MHz external xtal and x5 PLL
		clock {
			source = "MOSC";
			xtal   = "X16_0MHz";
			pll    = true;
			div    = 5;
		}

		gpio {
			f {
				led1@1 { direction = "out"; }
				led2@2 { direction = "out"; }
			}

			a {
				uart_rx@0 {
					direction = "in";
					function  = 1;
				}

				uart_tx@1 {
					direction = "in";
					function  = 1;
				}

				spi_ck@2 {
					direction = "out";
					function  = 2;
				}

				spi_cs@3 {
					direction = "out";
					function  = 2;
				}

				spi_rx@4 {
					direction = "in";
					function  = 2;
				}

				spi_tx@5 {
					direction = "out";
					function  = 2;
				}
			}
		}

		timer {
			// The mcu contain both 16/32bit and "wide" 32/64bit timers.
			timer@w0 {
				// Prescale sysclk (here 80MHz) to 1Mhz since the wait code expects 1us granularity
				prescale = 80;
				mode = "periodic";
			}
		}

		uart {
			uart@0 {
				mode = "115200,8n1";
			}
		}
	}

	os {
		single_task {
			loop = "run";
			args {
				timer = &timer;
				spi_tx = &spi_tx;
				uart = &uart;
			}
		}
	}
);

#[derive(Copy, Clone)]
struct Apa106Led {
	red: u8,
	green: u8,
	blue: u8,
}

struct Cube4<'a> {
	spi: &'a tiva_c::spi::Spi,

	cube_frame: [Apa106Led; 64],
}

impl<'a> Cube4<'a> {
	fn new(spi: &tiva_c::spi::Spi) -> Cube4 {
		let blank_frame: [Apa106Led; 64] = [Apa106Led { red: 0, green: 0x05, blue: 0 }; 64];

		Cube4 {
			spi: spi,

			cube_frame: blank_frame
		}
	}

	pub fn fill(&mut self, fill_colour: Apa106Led) {
		self.cube_frame = [fill_colour; 64];
	}

	pub fn set_at_index(&mut self, index: usize, colour: Apa106Led) {
		self.cube_frame[index] = colour;
	}

	pub fn flush(&self) {
		for led in self.cube_frame.into_iter() {
			for byte in colour_to_raw(led).into_iter() {
				self.spi.write(*byte);
			}
		}
	}
}

fn bit_is_set(byte: u8, bit_index: u8) -> bool {
	(byte & (1 << bit_index)) != 0
}

const ON_BYTE: u8 = 0b1111_1100;
const OFF_BYTE: u8 = 0b1100_0000;

fn colour_to_raw(input: &Apa106Led) -> [u8; 24] {
	let mut bytes: [u8; 24] = [0; 24];

	// Gamma correct colours
	let gamma_corrected_input = Apa106Led {
		red: GAMMA_MAP[input.red as usize],
		green: GAMMA_MAP[input.green as usize],
		blue: GAMMA_MAP[input.blue as usize],
	};

	// SPI transmits MSB first
	for pos in 0..8 {
		bytes[7 - pos as usize] = if bit_is_set(gamma_corrected_input.red, pos as u8) { ON_BYTE } else { OFF_BYTE };

		bytes[8 + (7 - pos as usize)] = if bit_is_set(gamma_corrected_input.green, pos as u8) { ON_BYTE } else { OFF_BYTE };

		bytes[16 + (7 - pos as usize)] = if bit_is_set(gamma_corrected_input.blue, pos as u8) { ON_BYTE } else { OFF_BYTE };
	}

	bytes
}

fn wheel(wheelpos: u8) -> Apa106Led {
	let mut thingy = wheelpos;

	if thingy < 85 {
		Apa106Led { red: thingy * 3, green: 255 - thingy * 3, blue: 0 }
	} else if thingy < 170 {
		thingy -= 85;

		Apa106Led { red: 255 - thingy * 3, green: 0, blue: thingy * 3 }
	} else {
		thingy -= 170;

		Apa106Led { red: 0, green: thingy * 3, blue: 255 - thingy * 3 }
	}
}

fn run(args: &pt::run_args) {
	// fpu::enable_fpu();

	let spi = tiva_c::spi::Spi::new(tiva_c::spi::SpiConf {
		peripheral: tiva_c::spi::SpiId::Spi0,

		frequency: 4_678_362
	});

	args.uart.puts("Started\r\n");

	let mut counter: i16 = 0;
	let mut inc: i16 = 4;

	let mut cube = Cube4::new(&spi);

	cube.fill(Apa106Led { red: 0xff, green: 0, blue: 0 });

	cube.flush();

	loop {
		args.timer.wait_ms(16);

		for index in 0..64 {
			cube.set_at_index(index as usize, wheel(((index * 4) + counter as u8) & 255));
		}

		cube.flush();

		counter += 1;
	}
}