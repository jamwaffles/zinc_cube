#![crate_type = "staticlib"]
#![feature(plugin, start, core_intrinsics)]
#![no_std]
#![plugin(macro_platformtree)]

extern crate zinc;

use zinc::hal::spi::Spi;
use zinc::hal::timer::Timer;
use zinc::drivers::chario::CharIO;
use zinc::hal::tiva_c;

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

	// SPI transmits MSB first
	for pos in 0..8 {
		bytes[7 - pos as usize] = if bit_is_set(input.red, pos as u8) { ON_BYTE } else { OFF_BYTE };

		bytes[8 + (7 - pos as usize)] = if bit_is_set(input.green, pos as u8) { ON_BYTE } else { OFF_BYTE };

		bytes[16 + (7 - pos as usize)] = if bit_is_set(input.blue, pos as u8) { ON_BYTE } else { OFF_BYTE };
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