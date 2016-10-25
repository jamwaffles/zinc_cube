#![crate_type = "staticlib"]
#![feature(plugin, start, core_intrinsics)]
#![no_std]
#![plugin(macro_platformtree)]

extern crate zinc;

// use zinc::hal::spi::Spi;
use zinc::hal::cortex_m4::fpu;
use zinc::hal::timer::Timer;
use zinc::drivers::chario::CharIO;
use zinc::hal::tiva_c;
// use core::intrinsics;

mod cube;
mod apa106led;
mod tables;
mod colour_functions;
mod embedded_rand;
mod patterns;

use colour_functions::{ christmas_wheel, fade };
use apa106led::{ Apa106Led, WARM_WHITE, OFF };
use cube::{ Cube4, Voxel };
use embedded_rand::{ rand_range };
use patterns::{ MAX_BRIGHTNESS };

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

fn clamp_to_u8(value: f32) -> u8 {
	let ret = if value > 255.0 {
		255
	} else {
		value as u8
	};

	ret
}

fn run(args: &pt::run_args) {
	fpu::enable_fpu();

	args.uart.puts("Started\r\n");

	let spi = tiva_c::spi::Spi::new(tiva_c::spi::SpiConf {
		peripheral: tiva_c::spi::SpiId::Spi0,

		frequency: 4_678_362
	});

	let mut cube = Cube4::new(&spi);

	cube.fill(Apa106Led { red: 1, green: 0, blue: 0 });

	cube.flush();
	args.timer.wait_ms(1);

	let raindrop_colour = fade(WARM_WHITE, MAX_BRIGHTNESS as f32 / 255.0);

	let mut counter = 0;

	loop {
		// Rain
		for _ in 0..4 {
			patterns::rain(&mut cube, &args.timer, raindrop_colour);
		}

		// Rainbow
		// for _ in 0..4 {
		// 	patterns::christmas_rainbow(&mut cube, &args.timer);
		// }

		// Fadey slices thing
		// for _ in 0..4 {
			patterns::animated_slices(&mut cube, &args.timer);
		// }
	}
}