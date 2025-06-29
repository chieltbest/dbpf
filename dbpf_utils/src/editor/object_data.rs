use std::sync::Arc;
use crate::editor::drag_checkbox_fn;
use eframe::{egui, glow};
use eframe::egui::{ComboBox, DragValue, Response, Ui};
use dbpf::internal_file::object_data::{ObjectData, Version};
use crate::editor::Editor;

impl Editor for ObjectData {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui, gl: &Option<Arc<glow::Context>>) -> Response {
        let ObjectData {
            file_name,
            version,
            initial_stack_size,
            default_wall_adjacent,
            default_placement,
            default_wall_placement,
            default_allowed_height,
            interaction_table_id,
            interaction_group,
            object_type,
            multi_tile_master_id,
            multi_tile_sub_index,
            use_default_placement,
            look_at_score,
            guid,
            unlockable,
            catalog_use,
            price,
            body_strings_id,
            slot_id,
            diagonal_selector_guid,
            grid_aligned_selector_guid,
            object_ownership,
            ignore_globalsim,
            cannot_move_out_with,
            hauntable,
            proxy_guid,
            slot_group,
            aspiration,
            memory_nice,
            ignore_quarter_tile_placement,
            initial_depreciation,
            daily_depreciation,
            self_depreciating,
            depreciation_limit,
            room_sort,
            function_sort,
            catalog_strings_id,
            is_global_sim_object,
            tooltip_name_type,
            template_version,
            niceness_multiplier,
            no_duplicate_on_placement,
            want_category,
            no_new_name_from_template,
            object_version,
            default_thumbnail_id,
            motive_effects_id,
            job_object_guid,
            catalog_popup_id,
            ignore_current_model_index,
            level_offset,
            shadow_type,
            num_attributes,
            num_object_arrays,
            for_sale_flags,
            front_direction,
            unused2,
            multi_tile_lead,
            expansion_flags_1,
            expansion_flags_2,
            chair_entry_flags,
            tile_width,
            inhibit_suit_copying,
            build_mode_type,
            original_guid,
            default_graphic,
            unused3,
            build_mode_subsort,
            selector_category,
            selector_sub_category,
            footprint_mask,
            extend_footprint,
            object_size,
            unused4,
            wall_style_sprite_id,
            hunger_rating,
            comfort_rating,
            hygiene_rating,
            bladder_rating,
            energy_rating,
            fun_rating,
            room_rating,
            gives_skill,
            num_type_attributes,
            misc_flags,
            type_attribute_guid,
            function_sub_sort,
            downtown_sort,
            keep_buying,
            vacation_sort,
            reset_lot_action,
            object_type_3d,
            community_sort,
            dream_flags,
            thumbnail_flags,
            scratch_rating,
            chew_rating,
            unused5,
            unused6,
            requirements,
            file_name_2,
        } = self;

        let res = egui::Grid::new("ObjectData editor")
            .show(ui, |ui| {
                macro_rules! separator {
                    ($name:expr) => {
                        ui.label($name);
                        ui.separator();
                        ui.end_row();
                    };
                }
                macro_rules! drag {
                    ($name:expr, $key:ident) => {
                        {
                            ui.label($name);
                            let res = ui.add(DragValue::new($key));
                            ui.end_row();
                            res
                        }
                    };
                }
                macro_rules! drag_hex {
                    ($name:expr, $key:ident) => {
                        {
                            ui.label($name);
                            let res = ui.add(DragValue::new($key).hexadecimal(1, false, false));
                            ui.end_row();
                            res
                        }
                    };
                }
                macro_rules! drag_checkbox {
                    ($name:expr, $key:ident, $($c_name:expr),*) => {
                        drag_checkbox_fn($name, $key, [$($c_name),*], ui)
                    };
                }

                ui.label("filename");
                let mut res = file_name.name.show_editor(&mut 500.0, ui, gl);
                ui.end_row();

                ui.label("version");
                ComboBox::from_id_salt("version")
                    .selected_text(match version {
                        Version::S2 => "Base game",
                        Version::S2U => "University",
                        Version::S2P => "Pets",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(version, Version::S2, "Base game");
                        ui.selectable_value(version, Version::S2U, "University");
                        ui.selectable_value(version, Version::S2P, "Pets");
                    });
                ui.end_row();

                separator!("Price");

                res |= drag!("price", price);
                res |= drag!("initial depreciation", initial_depreciation);
                res |= drag!("daily depreciation", daily_depreciation);
                res |= drag!("self depreciating", self_depreciating);
                res |= drag!("depreciation limit", depreciation_limit);

                separator!("Catalog sort");

                res |= drag_checkbox!("catalog use", catalog_use,
                    "adults", "children", "group", "teens", "elders", "toddlers");
                res |= drag_checkbox!("room", room_sort,
                    "kitchen", "bedroom", "bathroom", "living room", "outside", "dining room", "misc", "study", "kids");
                res |= drag_checkbox!("function", function_sort,
                    "seating", "surfaces", "appliances", "electronics", "plumbing", "decorative", "general", "lighting",
                    "hobbies", "unknown", "aspiration rewards", "career rewards");
                res |= drag_checkbox!("valid ep", expansion_flags_1,
                    "base", "university", "night life", "business", "family", "glamour", "pets", "seasons", "celebration",
                    "HM fashion", "bon voyage", "teen style", "unknown", "free time", "kitchen & bath", "ikea");
                res |= drag_checkbox!("valid ep", expansion_flags_2,
                    "apartment life", "mansion & garden"); // TODO store edition
                res |= drag_hex!("build mode type", build_mode_type);
                res |= drag_hex!("build mode subsort", build_mode_subsort); // TODO proper enum
                res |= drag_hex!("function sub sort", function_sub_sort);
                res |= drag_hex!("downtown sort", downtown_sort);
                res |= drag_hex!("holiday sort", vacation_sort);
                res |= drag_checkbox!("community sort", community_sort,
                    "food", "shopping", "outdoor", "street", "miscellaneous");

                separator!("Catalog ratings");

                res |= drag!("hunger", hunger_rating);
                res |= drag!("comfort", comfort_rating);
                res |= drag!("hygiene", hygiene_rating);
                res |= drag!("bladder", bladder_rating);
                res |= drag!("energy", energy_rating);
                res |= drag!("fun", fun_rating);
                res |= drag!("room", room_rating);
                res |= drag_checkbox!("gives skill", gives_skill,
                    "cooking", "mechanical", "logic", "body", "creativity", "charisma", "cleaning");
                res |= drag!("scratch", scratch_rating);
                res |= drag!("chew", chew_rating);

                separator!("User placement");

                res |= drag_hex!("default wall adjacent flags", default_wall_adjacent);
                res |= drag_hex!("default placement flags", default_placement);
                res |= drag_hex!("default wall placement flags", default_wall_placement);
                res |= drag_hex!("default allowed height flags", default_allowed_height);
                res |= drag_checkbox!("use default placement flags", use_default_placement, "");
                res |= drag_checkbox!("ignore quarter tile placement", ignore_quarter_tile_placement, "");
                res |= drag_checkbox!("no duplicate on placement", no_duplicate_on_placement, "");
                res |= drag_checkbox!("keep buying", keep_buying, "");

                separator!("Mesh & graphics");

                res |= drag_hex!("multi-tile master id", multi_tile_master_id);
                res |= drag_hex!("multi-tile sub index", multi_tile_sub_index);
                res |= drag_hex!("multi-tile lead object", multi_tile_lead);
                res |= drag_hex!("level offset", level_offset);
                res |= drag_hex!("shadow type", shadow_type);
                res |= drag_hex!("front direction", front_direction);
                res |= drag_hex!("chair entry flags", chair_entry_flags);
                res |= drag_hex!("tile width", tile_width);
                res |= drag_hex!("footprint mask", footprint_mask);
                res |= drag_hex!("extend footprint", extend_footprint);
                res |= drag_hex!("object size", object_size);
                res |= drag_hex!("3D object type", object_type_3d);

                separator!("Resource cross-refs");

                res |= drag_hex!("interaction table id", interaction_table_id);
                res |= drag_hex!("interaction group", interaction_group);
                res |= drag_hex!("body strings id", body_strings_id);
                res |= drag_hex!("slots id", slot_id);
                res |= drag_hex!("slot group", slot_group);
                res |= drag_hex!("catalog strings id", catalog_strings_id);
                res |= drag_hex!("default thumbnail id", default_thumbnail_id);
                res |= drag_hex!("motive effects id", motive_effects_id);
                res |= drag_hex!("catalog popup id", catalog_popup_id);
                res |= drag_hex!("default graphic", default_graphic);

                separator!("GUIDs");

                res |= drag_hex!("guid", guid);
                res |= drag_hex!("diagonal selector", diagonal_selector_guid);
                res |= drag_hex!("grid-aligned selector", grid_aligned_selector_guid);
                res |= drag_hex!("proxy", proxy_guid);
                res |= drag_hex!("job object", job_object_guid);
                res |= drag_hex!("original", original_guid);

                separator!("Data space");

                res |= drag!("initial stack size", initial_stack_size);
                res |= drag!("number of attributes", num_attributes);
                res |= drag!("number of object arrays", num_object_arrays);
                res |= drag!("number of type attributes", num_type_attributes);

                separator!("Memories & wants");

                res |= drag_hex!("aspiration flags", aspiration);
                res |= drag_checkbox!("memory nice/bag", memory_nice, "");
                res |= drag_hex!("want category", want_category);

                separator!("Miscellaneous");

                res |= drag_hex!("type", object_type);
                res |= drag!("look at score", look_at_score);
                res |= drag_checkbox!("unlockable", unlockable, "");
                res |= drag_hex!("object ownership flags", object_ownership);
                res |= drag_checkbox!("ignore globalsim field", ignore_globalsim, "");
                res |= drag_checkbox!("cannot move out with", cannot_move_out_with, "");
                res |= drag_checkbox!("hauntable", hauntable, "");
                res |= drag_checkbox!("is global sim object", is_global_sim_object, "");
                res |= drag_hex!("tooltip name type", tooltip_name_type);
                res |= drag_hex!("template version", template_version);
                res |= drag!("niceness multiplier", niceness_multiplier);
                res |= drag_checkbox!("no new name from template", no_new_name_from_template, "");
                res |= drag_hex!("object version", object_version);
                res |= drag_checkbox!("ignore model index in icon", ignore_current_model_index, "");
                res |= drag_hex!("for sale flags", for_sale_flags);
                res |= drag_checkbox!("inhibit suit copying", inhibit_suit_copying, "");
                res |= drag_hex!("selector category", selector_category);
                res |= drag_hex!("selector sub-category", selector_sub_category);
                res |= drag_hex!("misc flags", misc_flags);
                res |= drag_checkbox!("reset lot action", reset_lot_action, "");
                res |= drag_hex!("dream flags", dream_flags);
                res |= drag_hex!("thumbnail flags", thumbnail_flags);
                res |= drag_hex!("requirements", requirements);

                separator!("Sims 1");

                res |= drag_hex!("dynamic sprite base id", expansion_flags_1);
                res |= drag_hex!("number of dynamic sprites", expansion_flags_2);
                res |= drag_hex!("thumbnail graphic", selector_category);
                res |= drag_hex!("shadow flags", selector_sub_category);
                res |= drag_hex!("shadow brightness", object_size);
                res |= drag_hex!("wall sprite id", wall_style_sprite_id);
                res |= drag_hex!("type attribute guid", type_attribute_guid);

                separator!("Unknown");

                res |= drag!("unknown 0x3E", unused2);
                res |= drag!("unknown 0x49", unused3);
                res |= drag!("unknown 0x50", unused4);
                res |= drag!("unknown 0x69", unused5);
                res |= drag!("unknown 0x6A", unused6);

                ui.label("filename 2");
                res |= file_name_2.show_editor(&mut 500.0, ui, gl);
                ui.end_row();

                res
            });

        res.response | res.inner
    }
}
