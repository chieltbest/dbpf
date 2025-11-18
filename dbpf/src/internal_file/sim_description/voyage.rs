use binrw::binrw;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

#[bitfield]
#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageTraitFlags {
	pub robots: bool,
	pub plants: bool,
	pub lycanthropy: bool,
	pub witchiness: bool,
	unused: B12,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageData {
	pub vacation_days_left: u16,
	pub turn_ons: BonVoyageTraitFlags,
	pub turn_offs: BonVoyageTraitFlags,
	pub traits: BonVoyageTraitFlags,
}

#[bitfield]
#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageMementosFlags {
	pub go_on_island_vacation: bool,
	pub learn_island_greeting: bool,
	pub learn_hula_dance: bool,
	pub learn_hot_stone_massage: bool,
	pub learn_fire_dance: bool,
	pub learn_sea_chantey: bool,
	pub get_voodoo_dool: bool,
	pub go_on_mountain_vacation: bool,
	pub learn_mountain_greeting: bool,
	pub learn_slap_dance: bool,
	pub learn_deep_tissue_massage: bool,
	pub befriend_bigfoot: bool,
	pub go_on_far_east_vacation: bool,
	pub learn_far_east_greeting: bool,
	pub learn_tai_chi: bool,
	pub learn_to_teleport: bool,
	pub learn_dragon_legend: bool,
	pub learn_acupuncture_massage: bool,
	pub have_a_very_good_vacation: bool,
	pub have_three_good_vacations: bool,
	pub have_five_good_vacation: bool,
	pub discover_a_secret_lot: bool,
	pub discover_all_secret_lots: bool,
	pub go_on_a_tour: bool,
	pub win_log_rolling: bool,
	pub win_at_lucky_shrine: bool,
	pub learn_all_greetings: bool,
	pub get_bullseye_at_axe_throwing: bool,
	pub play_on_pirate_ship: bool,
	pub dig_up_treasure: bool,
	pub find_secret_map: bool,
	pub rake_zen_garden: bool,
	pub make_offering_at_monkey_ruins: bool,
	pub sleep_in_tent: bool,
	pub find_seashell: bool,
	pub win_at_maj_jong: bool,
	pub serve_drink_tea: bool,
	pub examine_tree_ring_display: bool,
	pub go_on_all_tours: bool,
	pub go_on_five_tours: bool,
	pub eat_flapjacks: bool,
	pub eat_pineapple_surprise: bool,
	pub eat_chirashi: bool,
	pub order_room_service: bool,
	pub order_photo_album: bool,
	unused: B19,
}
