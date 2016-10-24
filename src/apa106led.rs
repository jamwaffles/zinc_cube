#[derive(Copy, Clone)]
pub struct Apa106Led {
	pub red: u8,
	pub green: u8,
	pub blue: u8,
}

pub const WARM_WHITE: Apa106Led = Apa106Led {
	red: 255,
	green: 183,
	blue: 76,
};