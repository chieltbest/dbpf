use binrw::binrw;

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum JobAssignment {
	#[default]
	Nothing = 0x0,
	Chef = 0x1,
	Host = 0x2,
	Server = 0x3,
	Cashier = 0x4,
	Bartender = 0x5,
	Barista = 0x6,
	DJ = 0x7,
	SellLemonade = 0x8,
	Stylist = 0x9,
	Tidy = 0xA,
	Restock = 0xB,
	Sales = 0xC,
	MakeToys = 0xD,
	ArrangeFlowers = 0xE,
	BuildRobots = 0xF,
	MakeFood = 0x10,
	Masseuse = 0x11,
	MakePottery = 0x12,
	Sewing = 0x13,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BusinessData {
	pub lot_id: u16,
	pub salary: u16,
	pub flags: u16, // TODO bitfield
	pub assignment: JobAssignment,
}
