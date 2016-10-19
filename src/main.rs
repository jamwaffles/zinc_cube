#![crate_type = "staticlib"]
#![feature(plugin, start, core_intrinsics)]
#![no_std]
#![plugin(macro_platformtree)]

extern crate zinc;

use zinc::hal::spi::Spi;
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

fn run(args: &pt::run_args) {
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

		cube.fill(Apa106Led { red: counter as u8, green: 0, blue: 0 });

		cube.flush();

		counter += inc;

		if counter > 250 || counter == 0 {
			inc *= -1;
		}
	}
}