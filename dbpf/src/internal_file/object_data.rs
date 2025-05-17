use binrw::{binrw, parser, writer, BinRead, BinResult, BinWrite};
use crate::common::{BigString, ByteString, FileName, PascalString};

#[parser(reader)]
fn filename_parser(version: Version) -> BinResult<ByteString> {
    Ok(if version > Version::S2 {
        PascalString::<u32>::read_le(reader)?.into()
    } else {
        BigString::read_le(reader)?.into()
    })
}

#[writer(writer)]
fn filename_writer(value: &ByteString, version: Version) -> BinResult<()> {
    if version > Version::S2 {
        PascalString::<u32>::from(value.clone()).write_le(writer)
    } else {
        BigString::from(value.clone()).write_le(writer)
    }
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum Version {
    S2 = 0x8b,
    #[default]
    S2U = 0x8c,
    S2P = 0x8d,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default)]
pub struct ObjectData {
    pub file_name: FileName,
    pub version: Version,

    pub initial_stack_size: u16,
    pub default_wall_adjacent: u16,
    pub default_placement: u16,
    pub default_wall_placement: u16,
    pub default_allowed_height: u16,
    pub interaction_table_id: u16,
    pub interaction_group: u16,
    pub object_type: u16,
    pub multi_tile_master_id: u16,
    pub multi_tile_sub_index: u16,
    pub use_default_placement: u16,
    pub look_at_score: u16,
    pub guid: u32,
    pub unlockable: u16,
    pub catalog_use: u16,
    pub price: u16,
    pub body_strings_id: u16,
    pub slot_id: u16,
    pub diagonal_selector_guid: u32,
    pub grid_aligned_selector_guid: u32,
    pub object_ownership: u16,
    pub ignore_globalsim: u16,
    pub cannot_move_out_with: u16,
    pub hauntable: u16,
    pub proxy_guid: u32,
    pub slot_group: u16,
    pub aspiration: u16,
    pub memory: u16,
    pub sale_price_different: u16,
    pub initial_depreciation: u16,
    pub daily_depreciation: u16,
    pub self_depreciation: u16,
    pub depreciation_limit: u16,
    pub room_sort: u16,
    pub function_sort: u16,
    pub catalog_strings_id: u16,
    pub is_global_sim_object: u16,
    pub tooltip_name_type: u16,
    pub template_version: u16,
    pub niceness_multiplier: u16,
    pub no_duplicate_on_placement: u16,
    pub want_category: u16,
    pub no_new_name_from_template: u16,
    pub object_version: u16,
    pub default_thumbnail_id: u16,
    pub motive_effects_id: u16,
    pub job_object_guid: u32,
    pub catalog_popup_id: u16,
    pub ignore_current_model_index: u16,
    pub level_offset: u16,
    pub shadow_type: u16,
    pub num_attributes: u16,
    pub num_object_arrays: u16,
    unused1: u16,
    pub front_direction: u16,
    unused2: u16,
    pub multi_tile_lead: u16,
    pub expansion_flags: u16,
    pub num_dynamic_sprites: u16,
    pub chair_entry_flags: u16,
    pub tile_width: u16,
    pub inhibit_suit_copying: u16,
    pub build_mode_type: u16,
    pub original_guid: u32,
    pub object_model_guid: u32,
    pub build_mode_subsort: u16,
    pub thumbnail_graphic: u16,
    pub shadow_flags: u16,
    pub footprint_mask: u16,
    unused3: u16,
    pub shadow_brightness: u16,
    unused4: u16,
    pub wall_style_sprite_id: u16,
    pub hunger_rating: u16,
    pub comfort_rating: u16,
    pub hygiene_rating: u16,
    pub bladder_rating: u16,
    pub energy_rating: u16,
    pub fun_rating: u16,
    pub room_rating: u16,
    pub gives_skill: u16,
    pub num_type_attributes: u16,
    pub misc_flags: u16,
    pub type_attribute_guid: u32,
    pub function_sub_sort: u16,
    pub downtown_sort: u16,
    pub keep_buying: u16,
    pub vacation_sort: u16,
    pub reset_lot_action: u16,
    pub object_type_3d: u16,
    pub community_sort: u16,
    pub dream_flags: u16,
    unused: [u16; 6],

    // SimPE will sometimes not write this string for some reason, so we have to support that behaviour
    #[br(try)]
    #[br(parse_with = filename_parser)]
    #[bw(write_with = filename_writer)]
    pub file_name_2: ByteString,
}
