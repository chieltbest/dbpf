// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editor::vector::VecEditorState;
use crate::editor::{drag_checkbox_fn, drag_fn, Editor};
use dbpf::internal_file::sim_description::apartment::{
	ApartmentLifeData, ApartmentLifePreReleaseData, TitlePostName,
};
use dbpf::internal_file::sim_description::base::{
	AspirationFlags, BodyFlags, BodyShape, CultFlags, Gender, GhostFlags, LifeSection, NpcType,
	PersonFlags0, PersonFlags1, SelectionFlags, SimRelation, ZodiacSign,
};
use dbpf::internal_file::sim_description::business::{BusinessData, JobAssignment};
use dbpf::internal_file::sim_description::freetime::{
	BugCollectionFlags, FreeTimeData, PreferredHobby,
};
use dbpf::internal_file::sim_description::nightlife::{
	NightlifeData, NightlifeTraitFlags, Species,
};
use dbpf::internal_file::sim_description::pets::PetTraitFlags;
use dbpf::internal_file::sim_description::university::{UniData, UniProgressionFlags};
use dbpf::internal_file::sim_description::voyage::BonVoyageData;
use dbpf::internal_file::sim_description::voyage::{BonVoyageMementosFlags, BonVoyageTraitFlags};
use dbpf::internal_file::sim_description::{SimDescription, Version};
use eframe::egui;
use eframe::egui::{ComboBox, Response, SliderClamping, Ui};

fn checkbox_member_get_set<'a, S, T: Into<bool> + From<bool>>(
	ui: &mut Ui,
	name: &str,
	obj: &mut S,
	get: impl FnOnce(&S) -> T,
	set: impl FnOnce(&mut S, T),
) -> Response {
	let mut cur_value = get(obj).into();
	let res = ui.checkbox(&mut cur_value, name);
	if res.changed() {
		set(obj, cur_value.into());
	}
	res
}

impl Editor for SimDescription {
	type EditorState = ();

	fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let Self {
			unknown_0,
			version,
			version_repeat,
			sitting,
			money_over_head,
			personality_nice,
			personality_active,
			uni_effort,
			personality_playful,
			personality_outgoing,
			personality_neat,
			current_outfit,
			skill_cleaning,
			skill_cooking,
			skill_charisma,
			skill_mechanical,
			skill_music,
			partner_id,
			skill_creativity,
			skill_art,
			skill_body,
			skill_logic,
			group_talk_state,
			body_temperature,
			interaction_current_index,
			preference_gender_male,
			preference_gender_female,
			job_data,
			interaction_data_field_1,
			interaction_sub_queue_count,
			tick_counter,
			interaction_data_field_2,
			motives_static,
			censorship_flags,
			neighbor_id,
			person_type,
			priority,
			greet_status,
			visitor_schedule,
			autonomy_level,
			route_slot,
			route_multi_slot_index,
			route_status,
			route_goal,
			look_at_object_id,
			look_at_slot_id,
			look_at_state,
			look_at_time_remaining,
			interaction_next_queued_index,
			aspiration,
			genetic_personality_neat,
			genetic_personality_nice,
			genetic_personality_active,
			genetic_personality_outgoing,
			genetic_personality_playful,
			ui_icon_flags,
			interaction_findbestaction_object_id,
			memory_score,
			route_start_slot,
			school_grade,
			job_promotion_level,
			age,
			social_menu_object_id,
			skin_color,
			family_instance,
			route_result,
			job_performance,
			swimming,
			gender,
			private,
			lingering_house_number,
			route_ghost_flags,
			job_pto,
			zodiac,
			non_interruptable,
			interaction_next_queued_continuation,
			footprint_extension,
			render_display_flags,
			interaction_sub_queue_master_interaction_object_id,
			interaction_sub_queue_master_interaction_index,
			interaction_sub_queue_next_interaction_index,
			interaction_sub_queue_next_interaction_object_id,
			interaction_queue_next_interaction_object_id,
			interaction_queue_current_interaction_object_id,
			body_flags,
			fatness,
			uni_grade,
			person_flags_0,
			life_score_teen,
			life_score_adult,
			life_score_elder,
			voice_type,
			job_object_guid,
			age_days_left,
			age_previous_days,
			decay_hunger,
			decay_comfort,
			decay_bladder,
			decay_energy,
			decay_hygiene,
			decay_social_family,
			decay_social,
			decay_shopping,
			decay_fun,
			interaction_current_running_index,
			interaction_current_running_object_id,
			motive_power,
			work_outfit_index,
			decay_scratch_chew,
			school_object_guid,
			interaction_current_guid,
			interaction_linked_deleted,
			skill_romance,
			loco_weight_0,
			loco_weight_1,
			loco_personality_index,
			loco_personality_weight,
			loco_mood_index,
			loco_mood_weight,
			loco_motives,
			outfit_source_guid,
			environment_score_override,
			fitness_preference,
			pension,
			interest_politics,
			interest_money,
			interest_environment,
			interest_crime,
			interest_entertainment,
			interest_culture,
			interest_food,
			interest_health,
			interest_fashion,
			interest_sports,
			interest_paranormal,
			interest_travel,
			interest_work,
			interest_weather,
			interest_animals,
			interest_school,
			interest_toys,
			interest_scifi,
			interest_unused_0,
			interest_unused_1,
			allocated_suburb,
			person_flags_1,
			bodyshape,
			interest_unused_5,
			cult_flags,
			interest_unused_7,
			interest_unused_8,
			interest_unused_9,
			religion_id,
			interest_unused_11,
			unselectable,
			npc_type,
			age_duration,
			interaction_sub_queue_object_id,
			selection_flags,
			person_flags_2,
			aspiration_score,
			aspiration_reward_points_spent,
			aspiration_score_raw,
			mood_booster,
			interaction_current_joinable,
			unlinked,
			interaction_autonomous,
			job_retired_guid,
			job_retired_level,
			uni_data,
			nightlife_data,
			business_data,
			pet_traits,
			bon_voyage_data,
			subspecies,
			free_time_data,
			apartment_life_pre_release_data,
			apartment_life_data,
			instance,
			guid,
			unknown_1,
			relations,
			unknown_3,
			mementos,
		} = self;
		ui.horizontal_wrapped(|ui| {
			ui.label("version");
			ComboBox::from_id_salt("version")
				.selected_text(version.human_name())
				.show_ui(ui, |ui| {
					for v in [
						Version::BaseGame,
						Version::University,
						Version::Nightlife,
						Version::Business,
						Version::Pets,
						Version::Castaway,
						Version::BonVoyage,
						Version::BonVoyageB,
						Version::FreeTime,
						Version::ApartmentLifePreRelease,
						Version::ApartmentLife,
					] {
						ui.selectable_value(version, v, v.human_name());
					}
				});
			ui.label("version 2");
			ComboBox::from_id_salt("version 2")
				.selected_text(version_repeat.human_name())
				.show_ui(ui, |ui| {
					for v in enum_iterator::all::<Version>() {
						ui.selectable_value(version_repeat, v, v.human_name());
					}
				});
			ui.end_row();
		});

		egui::CollapsingHeader::new("Character")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor character").show(ui, |ui| {
					drag_fn("instance", &mut instance.id, ui);
					drag_fn("GUID", &mut guid.id, ui);
					drag_fn("family instance", family_instance, ui);

					ui.label("gender");
					ComboBox::from_id_salt("gender")
						.selected_text(format!("{:?}", gender))
						.show_ui(ui, |ui| {
							ui.selectable_value(gender, Gender::Male, "Male");
							ui.selectable_value(gender, Gender::Female, "Female");
						});
					ui.end_row();

					drag_fn("voice type", voice_type, ui);

					ui.label("gender preference");
					ui.add(egui::Slider::new(preference_gender_male, -1000..=1000).text("male"));
					ui.end_row();
					ui.label("");
					ui.add(
						egui::Slider::new(preference_gender_female, -1000..=1000).text("female"),
					);
					ui.end_row();

					ui.label("zodiac");
					ComboBox::from_id_salt("zodiac")
						.selected_text(format!("{:?}", zodiac))
						.show_ui(ui, |ui| {
							for z in enum_iterator::all::<ZodiacSign>() {
								ui.selectable_value(zodiac, z, format!("{:?}", z));
							}
						});
					ui.end_row();

					ui.label("general");
					ui.horizontal(|ui| {
						checkbox_member_get_set(
							ui,
							"perma platinum",
							person_flags_0,
							PersonFlags0::perma_platinum,
							PersonFlags0::set_perma_platinum,
						);
						checkbox_member_get_set(
							ui,
							"want history",
							person_flags_0,
							PersonFlags0::want_history,
							PersonFlags0::set_want_history,
						);
					});
					ui.end_row();
					ui.label("vampire");
					ui.horizontal(|ui| {
						checkbox_member_get_set(
							ui,
							"vampire",
							person_flags_0,
							PersonFlags0::is_vampire,
							PersonFlags0::set_is_vampire,
						);
						checkbox_member_get_set(
							ui,
							"vampire smoke",
							person_flags_0,
							PersonFlags0::vampire_smoke,
							PersonFlags0::set_vampire_smoke,
						);
					});
					ui.end_row();
					ui.label("werewolf");
					ui.horizontal(|ui| {
						checkbox_member_get_set(
							ui,
							"werewolf",
							person_flags_0,
							PersonFlags0::is_werewolf,
							PersonFlags0::set_is_werewolf,
						);
						checkbox_member_get_set(
							ui,
							"lycanthropy carrier",
							person_flags_0,
							PersonFlags0::lycanthropy_carrier,
							PersonFlags0::set_lycanthropy_carrier,
						);
					});
					ui.end_row();
					ui.label("other");
					ui.horizontal(|ui| {
						checkbox_member_get_set(
							ui,
							"zombie",
							person_flags_0,
							PersonFlags0::is_zombie,
							PersonFlags0::set_is_zombie,
						);
						checkbox_member_get_set(
							ui,
							"runaway pet",
							person_flags_0,
							PersonFlags0::is_pet_runaway,
							PersonFlags0::set_is_pet_runaway,
						);
						checkbox_member_get_set(
							ui,
							"plant",
							person_flags_0,
							PersonFlags0::is_plantsim,
							PersonFlags0::set_is_plantsim,
						);
					});
					ui.end_row();

					ui.label("");
					ui.horizontal(|ui| {
						checkbox_member_get_set(
							ui,
							"bigfoot",
							person_flags_0,
							PersonFlags0::is_bigfoot,
							PersonFlags0::set_is_bigfoot,
						);
						checkbox_member_get_set(
							ui,
							"witch",
							person_flags_0,
							PersonFlags0::is_witch,
							PersonFlags0::set_is_witch,
						);
						checkbox_member_get_set(
							ui,
							"roommate",
							person_flags_0,
							PersonFlags0::is_roommate,
							PersonFlags0::set_is_roommate,
						);
					});
					ui.end_row();

					ui.label("height (LD asi mod)");
					ui.add(
						egui::Slider::new(skill_music, -100..=100).clamping(SliderClamping::Edits),
					);
					ui.end_row();

					drag_fn("autonomy level", autonomy_level, ui);
					drag_fn("skin color", skin_color, ui);

					ui.label("bodyshape");
					ComboBox::from_id_salt("bodyshape")
						.selected_text(format!("{:?}", bodyshape))
						.show_ui(ui, |ui| {
							for bs in enum_iterator::all::<BodyShape>() {
								ui.selectable_value(bodyshape, bs, format!("{:?}", bs));
							}
						});
					ui.end_row();

					ui.label("NPC type");
					ComboBox::from_id_salt("NPC type")
						.selected_text(format!("{:?}", npc_type))
						.show_ui(ui, |ui| {
							for nt in enum_iterator::all::<NpcType>() {
								ui.selectable_value(npc_type, nt, format!("{:?}", nt));
							}
						});
					ui.end_row();
				})
			});

		egui::CollapsingHeader::new("Current State")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor state").show(ui, |ui| {
					ui.label("fitness");
					ui.horizontal(|ui| {
						checkbox_member_get_set(
							ui,
							"fat",
							body_flags,
							BodyFlags::fat,
							BodyFlags::set_fat,
						);
						checkbox_member_get_set(
							ui,
							"fit",
							body_flags,
							BodyFlags::fit,
							BodyFlags::set_fit,
						);
						checkbox_member_get_set(
							ui,
							"hospital",
							body_flags,
							BodyFlags::hospital,
							BodyFlags::set_hospital,
						);
					});
					ui.end_row();

					ui.label("pregnancy");
					ui.horizontal(|ui| {
						checkbox_member_get_set(
							ui,
							"1st",
							body_flags,
							BodyFlags::pregnant_1st_trimester,
							BodyFlags::set_pregnant_1st_trimester,
						);
						checkbox_member_get_set(
							ui,
							"2nd",
							body_flags,
							BodyFlags::pregnant_2nd_trimester,
							BodyFlags::set_pregnant_2nd_trimester,
						);
						checkbox_member_get_set(
							ui,
							"3rd",
							body_flags,
							BodyFlags::pregnant_3rd_trimester,
							BodyFlags::set_pregnant_3rd_trimester,
						);
						checkbox_member_get_set(
							ui,
							"birth control",
							body_flags,
							BodyFlags::birth_control,
							BodyFlags::set_birth_control,
						);
					});
					ui.end_row();

					ui.label("fatness");
					ui.add(egui::Slider::new(fatness, 0..=1000));
					ui.end_row();

					drag_fn("body temperature", body_temperature, ui);

					drag_fn("current outfit", current_outfit, ui);
					drag_fn("outfit source GUID", &mut outfit_source_guid.id, ui);

					ui.label("life section");
					ComboBox::from_id_salt("life section")
						.selected_text(format!("{:?}", age))
						.show_ui(ui, |ui| {
							for ls in enum_iterator::all::<LifeSection>() {
								ui.selectable_value(age, ls, format!("{:?}", ls));
							}
						});
					ui.end_row();
					drag_fn("age days left", age_days_left, ui);
					drag_fn("age duration", age_duration, ui);
					drag_fn("age previous days", age_previous_days, ui);
				})
			});

		egui::CollapsingHeader::new("Aspiration")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor aspiration").show(ui, |ui| {
					macro_rules! aspiration_cb {
						($name:expr, $get_fn:ident, $set_fn:ident) => {
							ui.label("");
							checkbox_member_get_set(
								ui,
								$name,
								aspiration,
								AspirationFlags::$get_fn,
								AspirationFlags::$set_fn,
							);
							ui.end_row();
						};
					}

					ui.label("aspiration");
					checkbox_member_get_set(
						ui,
						"romance",
						aspiration,
						AspirationFlags::romance,
						AspirationFlags::set_romance,
					);
					ui.end_row();
					aspiration_cb!("family", family, set_family);
					aspiration_cb!("fortune", fortune, set_fortune);
					aspiration_cb!("reputation", reputation, set_reputation);
					aspiration_cb!("knowledge", knowledge, set_knowledge);
					aspiration_cb!("pleasure", pleasure, set_pleasure);
					aspiration_cb!("grilled cheese", grilled_cheese, set_grilled_cheese);
					aspiration_cb!("grow up", grow_up, set_grow_up);
					aspiration_cb!("power", power, set_power);
				});

				egui::Grid::new("SimDescription editor aspiration data").show(ui, |ui| {
					drag_fn("aspiration score", aspiration_score, ui);
					drag_fn(
						"aspiration reward points spent",
						aspiration_reward_points_spent,
						ui,
					);
					drag_fn("aspiration score raw", aspiration_score_raw, ui);
				})
			});

		egui::CollapsingHeader::new("Personality")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor personality").show(ui, |ui| {
					ui.label("");
					ui.label("current");
					ui.label("genetic");
					ui.end_row();

					macro_rules! slider_reciprocal {
						($name:expr, $p_var:ident, $gp_var:ident) => {
							ui.label($name);
							if ui.add(egui::Slider::new($p_var, 0..=1000)).double_clicked() {
								*$p_var = *$gp_var;
							}
							if ui
								.add(egui::Slider::new($gp_var, 0..=1000))
								.double_clicked()
							{
								*$gp_var = *$p_var;
							};
							ui.end_row();
						};
					}

					slider_reciprocal!("nice", personality_nice, genetic_personality_nice);
					slider_reciprocal!("active", personality_active, genetic_personality_active);
					slider_reciprocal!("playful", personality_playful, genetic_personality_playful);
					slider_reciprocal!(
						"outgoing",
						personality_outgoing,
						genetic_personality_outgoing
					);
					slider_reciprocal!("neat", personality_neat, genetic_personality_neat);
				})
			});

		egui::CollapsingHeader::new("Skills")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor skills").show(ui, |ui| {
					ui.label("cleaning");
					ui.add(egui::Slider::new(skill_cleaning, 0..=1000));
					ui.end_row();
					ui.label("cooking");
					ui.add(egui::Slider::new(skill_cooking, 0..=1000));
					ui.end_row();
					ui.label("charisma");
					ui.add(egui::Slider::new(skill_charisma, 0..=1000));
					ui.end_row();
					ui.label("mechanical");
					ui.add(egui::Slider::new(skill_mechanical, 0..=1000));
					ui.end_row();
					ui.label("creativity");
					ui.add(egui::Slider::new(skill_creativity, 0..=1000));
					ui.end_row();
					ui.label("body");
					ui.add(egui::Slider::new(skill_body, 0..=1000));
					ui.end_row();
					ui.label("logic");
					ui.add(egui::Slider::new(skill_logic, 0..=1000));
					ui.end_row();
					ui.label("music");
					ui.add(egui::Slider::new(skill_music, 0..=1000));
					ui.end_row();
					ui.label("art");
					ui.add(egui::Slider::new(skill_art, 0..=1000));
					ui.end_row();
					ui.label("romance");
					ui.add(egui::Slider::new(skill_romance, 0..=1000));
					ui.end_row();
				})
			});

		egui::CollapsingHeader::new("Decay")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor decay").show(ui, |ui| {
					let FreeTimeData {
						decay_hunger_modifier,
						decay_comfort_modifier,
						decay_bladder_modifier,
						decay_energy_modifier,
						decay_hygiene_modifier,
						decay_fun_modifier,
						decay_social_modifier,
						..
					} = free_time_data;

					macro_rules! decay_modifier {
						($name:expr, $var:expr, $mod_var:expr) => {
							ui.label($name);
							ui.add(egui::Slider::new($var, -1000..=1000));
							ui.add_enabled(
								*version >= Version::FreeTime,
								egui::Slider::new($mod_var, -1000..=1000),
							);
							ui.end_row();
						};
					}

					ui.label("");
					ui.label("decay");
					ui.label("modifier");
					ui.end_row();

					decay_modifier!("hunger", decay_hunger, decay_hunger_modifier);
					decay_modifier!("comfort", decay_comfort, decay_comfort_modifier);
					decay_modifier!("bladder", decay_bladder, decay_bladder_modifier);
					decay_modifier!("energy", decay_energy, decay_energy_modifier);
					decay_modifier!("hygiene", decay_hygiene, decay_hygiene_modifier);
					decay_modifier!("social", decay_social, decay_social_modifier);
					decay_modifier!("fun", decay_fun, decay_fun_modifier);

					ui.label("social family");
					ui.add(egui::Slider::new(decay_social_family, -1000..=0));
					ui.end_row();
					ui.label("shopping");
					ui.add(egui::Slider::new(decay_shopping, -1000..=0));
					ui.end_row();
					ui.label("scratch/chew");
					ui.add(egui::Slider::new(decay_scratch_chew, -1000..=0));
					ui.end_row();
				})
			});

		egui::CollapsingHeader::new("Interests")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor interests").show(ui, |ui| {
					ui.label("politics");
					ui.add(egui::Slider::new(interest_politics, 0..=1000));
					ui.end_row();
					ui.label("money");
					ui.add(egui::Slider::new(interest_money, 0..=1000));
					ui.end_row();
					ui.label("environment");
					ui.add(egui::Slider::new(interest_environment, 0..=1000));
					ui.end_row();
					ui.label("crime");
					ui.add(egui::Slider::new(interest_crime, 0..=1000));
					ui.end_row();
					ui.label("entertainment");
					ui.add(egui::Slider::new(interest_entertainment, 0..=1000));
					ui.end_row();
					ui.label("culture");
					ui.add(egui::Slider::new(interest_culture, 0..=1000));
					ui.end_row();
					ui.label("food");
					ui.add(egui::Slider::new(interest_food, 0..=1000));
					ui.end_row();
					ui.label("health");
					ui.add(egui::Slider::new(interest_health, 0..=1000));
					ui.end_row();
					ui.label("fashion");
					ui.add(egui::Slider::new(interest_fashion, 0..=1000));
					ui.end_row();
					ui.label("sports");
					ui.add(egui::Slider::new(interest_sports, 0..=1000));
					ui.end_row();
					ui.label("paranormal");
					ui.add(egui::Slider::new(interest_paranormal, 0..=1000));
					ui.end_row();
					ui.label("travel");
					ui.add(egui::Slider::new(interest_travel, 0..=1000));
					ui.end_row();
					ui.label("work");
					ui.add(egui::Slider::new(interest_work, 0..=1000));
					ui.end_row();
					ui.label("weather");
					ui.add(egui::Slider::new(interest_weather, 0..=1000));
					ui.end_row();
					ui.label("animals");
					ui.add(egui::Slider::new(interest_animals, 0..=1000));
					ui.end_row();
					ui.label("school");
					ui.add(egui::Slider::new(interest_school, 0..=1000));
					ui.end_row();
					ui.label("toys");
					ui.add(egui::Slider::new(interest_toys, 0..=1000));
					ui.end_row();
					ui.label("sci-fi");
					ui.add(egui::Slider::new(interest_scifi, 0..=1000));
					ui.end_row();
				})
			});

		egui::CollapsingHeader::new("School/Work")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor university").show(ui, |ui| {
					drag_fn("uni effort", uni_effort, ui);
					drag_fn("uni grade", uni_grade, ui);
					drag_fn("school object GUID", &mut school_object_guid.id, ui);
					drag_fn("school grade", school_grade, ui);
					drag_fn("job object GUID", &mut job_object_guid.id, ui);
					drag_fn("job promotion level", job_promotion_level, ui);
					drag_fn("job data", job_data, ui);
					drag_fn("job performance", job_performance, ui);
					drag_fn("job PTO", job_pto, ui);
					drag_fn("job outfit index", work_outfit_index, ui);
					drag_fn("retired job GUID", &mut job_retired_guid.id, ui);
					drag_fn("retired job promotion level", job_retired_level, ui);
					drag_fn("pension", pension, ui);
				})
			});

		egui::CollapsingHeader::new("Relations")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor relations").show(ui, |ui| {
					drag_fn("partner ID", &mut partner_id.id, ui);
				});
				ui.label("relations (sim ID)");
				relations.show_editor(&mut VecEditorState::Shared(()), ui)
			});

		egui::CollapsingHeader::new("Traits")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor bon voyage traits")
					.striped(true)
					.show(ui, |ui| {
						if *version < Version::Nightlife {
							ui.disable();
						}

						ui.label("");
						ui.label("trait");
						ui.label("turn-on");
						ui.label("turn-off");
						ui.end_row();

						let NightlifeData {
							traits,
							turn_ons,
							turn_offs,
							..
						} = nightlife_data;

						macro_rules! trait_cb {
							($name:expr, $get_fn:ident, $set_fun:ident) => {
								ui.label($name);
								checkbox_member_get_set(
									ui,
									"",
									traits,
									NightlifeTraitFlags::$get_fn,
									NightlifeTraitFlags::$set_fun,
								);
								checkbox_member_get_set(
									ui,
									"",
									turn_ons,
									NightlifeTraitFlags::$get_fn,
									NightlifeTraitFlags::$set_fun,
								);
								checkbox_member_get_set(
									ui,
									"",
									turn_offs,
									NightlifeTraitFlags::$get_fn,
									NightlifeTraitFlags::$set_fun,
								);
								ui.end_row();
							};
						}

						trait_cb!("cologne", cologne, set_cologne);
						trait_cb!("stink", stink, set_stink);
						trait_cb!("fatness", fatness, set_fatness);
						trait_cb!("fitness", fitness, set_fitness);
						trait_cb!("formal wear", formal_wear, set_formal_wear);
						trait_cb!("swim wear", swim_wear, set_swim_wear);
						trait_cb!("underwear", underwear, set_underwear);
						trait_cb!("vampirism", vampirism, set_vampirism);
						trait_cb!("facial hair", facial_hair, set_facial_hair);
						trait_cb!("glasses", glasses, set_glasses);
						trait_cb!("makeup", makeup, set_makeup);
						trait_cb!("full face makeup", full_face_makeup, set_full_face_makeup);
						trait_cb!("hats", hats, set_hats);
						trait_cb!("jewelry", jewelry, set_jewelry);
						trait_cb!("blonde hair", blonde_hair, set_blonde_hair);
						trait_cb!("red hair", red_hair, set_red_hair);
						trait_cb!("brown hair", brown_hair, set_brown_hair);
						trait_cb!("black hair", black_hair, set_black_hair);
						trait_cb!("custom hair", custom_hair, set_custom_hair);
						trait_cb!("grey hair", grey_hair, set_grey_hair);
						trait_cb!("hard worker", hard_worker, set_hard_worker);
						trait_cb!("unemployed", unemployed, set_unemployed);
						trait_cb!("logical", logical, set_logical);
						trait_cb!("charismatic", charismatic, set_charismatic);
						trait_cb!("good cook", good_cook, set_good_cook);
						trait_cb!("mechanical", mechanical, set_mechanical);
						trait_cb!("creative", creative, set_creative);
						trait_cb!("athletic", athletic, set_athletic);
						trait_cb!("good cleaner", good_cleaner, set_good_cleaner);
						trait_cb!("zombiism", zombiism, set_zombiism);

						if *version < Version::BonVoyage && *version >= Version::Nightlife {
							ui.disable();
						}

						let BonVoyageData {
							turn_ons,
							turn_offs,
							traits,
							..
						} = bon_voyage_data;

						macro_rules! trait_cb {
							($name:expr, $get_fn:ident, $set_fun:ident) => {
								ui.label($name);
								checkbox_member_get_set(
									ui,
									"",
									traits,
									BonVoyageTraitFlags::$get_fn,
									BonVoyageTraitFlags::$set_fun,
								);
								checkbox_member_get_set(
									ui,
									"",
									turn_ons,
									BonVoyageTraitFlags::$get_fn,
									BonVoyageTraitFlags::$set_fun,
								);
								checkbox_member_get_set(
									ui,
									"",
									turn_offs,
									BonVoyageTraitFlags::$get_fn,
									BonVoyageTraitFlags::$set_fun,
								);
								ui.end_row();
							};
						}

						trait_cb!("robots", robots, set_robots);
						trait_cb!("plants", plants, set_plants);
						trait_cb!("lycanthropy", lycanthropy, set_lycanthropy);
						trait_cb!("witchiness", witchiness, set_witchiness);
					})
			});

		egui::CollapsingHeader::new("University")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				ui.add_enabled_ui(*version >= Version::University, |ui| {
					let UniData {
						college_major_guid,
						semester_remaining_time,
						progression_flags,
						semester,
						on_campus,
						influence_bar_level,
						influence_minimum,
						influence,
					} = uni_data;
					egui::Grid::new("SimDescription editor university").show(ui, |ui| {
						drag_checkbox_fn("on campus", on_campus, [""], ui);

						drag_fn("major GUID", &mut college_major_guid.id, ui);

						ui.label("year");
						ui.horizontal(|ui| {
							checkbox_member_get_set(
								ui,
								"1",
								progression_flags,
								UniProgressionFlags::year_1,
								UniProgressionFlags::set_year_1,
							);
							checkbox_member_get_set(
								ui,
								"2",
								progression_flags,
								UniProgressionFlags::year_2,
								UniProgressionFlags::set_year_2,
							);
							checkbox_member_get_set(
								ui,
								"3",
								progression_flags,
								UniProgressionFlags::year_3,
								UniProgressionFlags::set_year_3,
							);
							checkbox_member_get_set(
								ui,
								"4",
								progression_flags,
								UniProgressionFlags::year_4,
								UniProgressionFlags::set_year_4,
							);
						});
						ui.end_row();
						ui.label("semester");
						ui.add(egui::Slider::new(semester, 0..=8));
						ui.end_row();
						ui.label("remaining time");
						ui.add(egui::Slider::new(semester_remaining_time, 0..=72));
						ui.end_row();

						drag_fn("influence", influence, ui);
						drag_fn("influence minimum", influence_minimum, ui);
						drag_fn("influence bar level", influence_bar_level, ui);

						ui.label("flags");
						ui.horizontal(|ui| {
							checkbox_member_get_set(
								ui,
								"good semester",
								progression_flags,
								UniProgressionFlags::good_semester,
								UniProgressionFlags::set_good_semester,
							);
							checkbox_member_get_set(
								ui,
								"probation",
								progression_flags,
								UniProgressionFlags::probation,
								UniProgressionFlags::set_probation,
							);
							checkbox_member_get_set(
								ui,
								"graduated",
								progression_flags,
								UniProgressionFlags::graduated,
								UniProgressionFlags::set_graduated,
							);
							checkbox_member_get_set(
								ui,
								"at class",
								progression_flags,
								UniProgressionFlags::at_class,
								UniProgressionFlags::set_at_class,
							);
						});
						ui.end_row();

						ui.label("unknown (gates?)");
						ui.horizontal(|ui| {
							checkbox_member_get_set(
								ui,
								"0",
								progression_flags,
								UniProgressionFlags::gates_0,
								UniProgressionFlags::set_gates_0,
							);
							checkbox_member_get_set(
								ui,
								"1",
								progression_flags,
								UniProgressionFlags::gates_1,
								UniProgressionFlags::set_gates_1,
							);
							checkbox_member_get_set(
								ui,
								"2",
								progression_flags,
								UniProgressionFlags::gates_2,
								UniProgressionFlags::set_gates_2,
							);
							checkbox_member_get_set(
								ui,
								"3",
								progression_flags,
								UniProgressionFlags::gates_3,
								UniProgressionFlags::set_gates_3,
							);
						});
						ui.end_row();
					})
				})
			});

		egui::CollapsingHeader::new("Nightlife")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				ui.add_enabled_ui(*version >= Version::Nightlife, |ui| {
					let NightlifeData {
						route_start_slot_owner_id,
						species,
						countdown,
						perfume_timer,
						date_timer,
						date_score,
						date_unlock_counter,
						love_potion_timer,
						aspiration_score_lock,
						date_neighbor_id,
						..
					} = nightlife_data;

					egui::Grid::new("SimDescription editor nightlife").show(ui, |ui| {
						ui.label("species");
						ComboBox::from_id_salt("species")
							.selected_text(format!("{:?}", species))
							.show_ui(ui, |ui| {
								for s in enum_iterator::all::<Species>() {
									ui.selectable_value(species, s, format!("{:?}", s));
								}
							});
						ui.end_row();

						drag_fn("perfume duration", perfume_timer, ui);
						drag_fn("love potion timer", love_potion_timer, ui);

						drag_fn("date timer", date_timer, ui);
						drag_fn("date score", date_score, ui);
						drag_fn("date unlock counter", date_unlock_counter, ui);
						drag_fn("date neighbor id?", date_neighbor_id, ui);

						drag_fn("countdown?", countdown, ui);
						drag_fn("aspiration score lock?", aspiration_score_lock, ui);

						drag_fn("route start slot owner id?", route_start_slot_owner_id, ui);
					});
				})
			});

		egui::CollapsingHeader::new("Business")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				ui.add_enabled_ui(*version >= Version::Business, |ui| {
					let BusinessData {
						lot_id,
						salary,
						flags,
						assignment,
					} = business_data;

					egui::Grid::new("SimDescription editor business").show(ui, |ui| {
						drag_fn("lot ID", lot_id, ui);
						drag_fn("salary", salary, ui);
						drag_fn("flags", flags, ui);

						ui.label("job assignment");
						ComboBox::from_id_salt("assignment")
							.selected_text(format!("{:?}", assignment))
							.show_ui(ui, |ui| {
								for a in enum_iterator::all::<JobAssignment>() {
									ui.selectable_value(assignment, a, format!("{:?}", a));
								}
							});
						ui.end_row();
					});
				})
			});

		egui::CollapsingHeader::new("Pets")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				ui.add_enabled_ui(*version >= Version::Pets, |ui| {
					egui::Grid::new("SimDescription editor pets").show(ui, |ui| {
						ui.label("traits");
						checkbox_member_get_set(
							ui,
							"gifted",
							pet_traits,
							PetTraitFlags::gifted,
							PetTraitFlags::set_gifted,
						);
						checkbox_member_get_set(
							ui,
							"doofus",
							pet_traits,
							PetTraitFlags::doofus,
							PetTraitFlags::set_doofus,
						);
						ui.end_row();
						ui.label("");
						checkbox_member_get_set(
							ui,
							"hyper",
							pet_traits,
							PetTraitFlags::hyper,
							PetTraitFlags::set_hyper,
						);
						checkbox_member_get_set(
							ui,
							"lazy",
							pet_traits,
							PetTraitFlags::lazy,
							PetTraitFlags::set_lazy,
						);
						ui.end_row();
						ui.label("");
						checkbox_member_get_set(
							ui,
							"independent",
							pet_traits,
							PetTraitFlags::independent,
							PetTraitFlags::set_independent,
						);
						checkbox_member_get_set(
							ui,
							"cowardly",
							pet_traits,
							PetTraitFlags::cowardly,
							PetTraitFlags::set_cowardly,
						);
						ui.end_row();
						ui.label("");
						checkbox_member_get_set(
							ui,
							"friendly",
							pet_traits,
							PetTraitFlags::friendly,
							PetTraitFlags::set_friendly,
						);
						checkbox_member_get_set(
							ui,
							"aggressive",
							pet_traits,
							PetTraitFlags::aggressive,
							PetTraitFlags::set_aggressive,
						);
						ui.end_row();
						ui.label("");
						checkbox_member_get_set(
							ui,
							"pigpen",
							pet_traits,
							PetTraitFlags::pigpen,
							PetTraitFlags::set_pigpen,
						);
						checkbox_member_get_set(
							ui,
							"finicky",
							pet_traits,
							PetTraitFlags::finicky,
							PetTraitFlags::set_finicky,
						);
						ui.end_row();
					});
				})
			});

		egui::CollapsingHeader::new("Castaway")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				ui.add_enabled_ui(*version == Version::Castaway, |ui| {
					ui.horizontal(|ui| {
						drag_fn("subspecies", subspecies, ui);
					})
				})
			});

		egui::CollapsingHeader::new("Bon Voyage")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				ui.add_enabled_ui(*version >= Version::BonVoyage, |ui| {
					let BonVoyageData {
						vacation_days_left, ..
					} = bon_voyage_data;
					ui.horizontal(|ui| {
						drag_fn("vacation days left", vacation_days_left, ui);
					});

					egui::Grid::new("SimDescription editor bon voyage mementos").show(ui, |ui| {
						macro_rules! memento_cb {
							($name:expr, $get_fn:ident, $set_fun:ident) => {
								ui.label("");
								checkbox_member_get_set(
									ui,
									$name,
									mementos,
									BonVoyageMementosFlags::$get_fn,
									BonVoyageMementosFlags::$set_fun,
								);
								ui.end_row();
							};
						}

						ui.label("mementos");
						checkbox_member_get_set(
							ui,
							"go on island vacation",
							mementos,
							BonVoyageMementosFlags::go_on_island_vacation,
							BonVoyageMementosFlags::set_go_on_island_vacation,
						);
						ui.end_row();

						memento_cb!(
							"learn island greeting",
							learn_island_greeting,
							set_learn_island_greeting
						);
						memento_cb!("learn hula dance", learn_hula_dance, set_learn_hula_dance);
						memento_cb!(
							"learn hot stone massage",
							learn_hot_stone_massage,
							set_learn_hot_stone_massage
						);
						memento_cb!("learn firedance", learn_fire_dance, set_learn_fire_dance);
						memento_cb!(
							"learn sea chantey",
							learn_sea_chantey,
							set_learn_sea_chantey
						);
						memento_cb!("get voodoo dool", get_voodoo_dool, set_get_voodoo_dool);
						memento_cb!(
							"go on mountain vacation",
							go_on_mountain_vacation,
							set_go_on_mountain_vacation
						);
						memento_cb!(
							"learn mountain greeting",
							learn_mountain_greeting,
							set_learn_mountain_greeting
						);
						memento_cb!("learn slap dance", learn_slap_dance, set_learn_slap_dance);
						memento_cb!(
							"learn deep tissue massage",
							learn_deep_tissue_massage,
							set_learn_deep_tissue_massage
						);
						memento_cb!("befriend bigfoot", befriend_bigfoot, set_befriend_bigfoot);
						memento_cb!(
							"go on far east vacation",
							go_on_far_east_vacation,
							set_go_on_far_east_vacation
						);
						memento_cb!(
							"learn far east greeting",
							learn_far_east_greeting,
							set_learn_far_east_greeting
						);
						memento_cb!("learn tai chi", learn_tai_chi, set_learn_tai_chi);
						memento_cb!(
							"learn to teleport",
							learn_to_teleport,
							set_learn_to_teleport
						);
						memento_cb!(
							"learn dragon legend",
							learn_dragon_legend,
							set_learn_dragon_legend
						);
						memento_cb!(
							"learn acupuncture massage",
							learn_acupuncture_massage,
							set_learn_acupuncture_massage
						);
						memento_cb!(
							"have a very good vacation",
							have_a_very_good_vacation,
							set_have_a_very_good_vacation
						);
						memento_cb!(
							"have three good vacations",
							have_three_good_vacations,
							set_have_three_good_vacations
						);
						memento_cb!(
							"have five good vacation",
							have_five_good_vacation,
							set_have_five_good_vacation
						);
						memento_cb!(
							"discover a secret lot",
							discover_a_secret_lot,
							set_discover_a_secret_lot
						);
						memento_cb!(
							"discover all secret lots",
							discover_all_secret_lots,
							set_discover_all_secret_lots
						);
						memento_cb!("go on a tour", go_on_a_tour, set_go_on_a_tour);
						memento_cb!("win log rolling", win_log_rolling, set_win_log_rolling);
						memento_cb!(
							"win at lucky shrine",
							win_at_lucky_shrine,
							set_win_at_lucky_shrine
						);
						memento_cb!(
							"learn all greetings",
							learn_all_greetings,
							set_learn_all_greetings
						);
						memento_cb!(
							"get bullseye at axe throwing",
							get_bullseye_at_axe_throwing,
							set_get_bullseye_at_axe_throwing
						);
						memento_cb!(
							"play on pirate ship",
							play_on_pirate_ship,
							set_play_on_pirate_ship
						);
						memento_cb!("dig up treasure", dig_up_treasure, set_dig_up_treasure);
						memento_cb!("find secret map", find_secret_map, set_find_secret_map);
						memento_cb!("rake zen garden", rake_zen_garden, set_rake_zen_garden);
						memento_cb!(
							"make offering at monkey ruins",
							make_offering_at_monkey_ruins,
							set_make_offering_at_monkey_ruins
						);
						memento_cb!("sleep in tent", sleep_in_tent, set_sleep_in_tent);
						memento_cb!("find seashell", find_seashell, set_find_seashell);
						memento_cb!("win at maj jong", win_at_maj_jong, set_win_at_maj_jong);
						memento_cb!("serve drink tea", serve_drink_tea, set_serve_drink_tea);
						memento_cb!(
							"examine tree ring display",
							examine_tree_ring_display,
							set_examine_tree_ring_display
						);
						memento_cb!("go on all tours", go_on_all_tours, set_go_on_all_tours);
						memento_cb!("go on five tours", go_on_five_tours, set_go_on_five_tours);
						memento_cb!("eat flapjacks", eat_flapjacks, set_eat_flapjacks);
						memento_cb!(
							"eat pineapple surprise",
							eat_pineapple_surprise,
							set_eat_pineapple_surprise
						);
						memento_cb!("eat chirashi", eat_chirashi, set_eat_chirashi);
						memento_cb!(
							"order room service",
							order_room_service,
							set_order_room_service
						);
						memento_cb!(
							"order photo album",
							order_photo_album,
							set_order_photo_album
						);
					});
				})
			});

		egui::CollapsingHeader::new("Free Time")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				ui.add_enabled_ui(*version >= Version::FreeTime, |ui| {
					let FreeTimeData {
						hobbies_cooking,
						hobbies_arts,
						hobbies_film,
						hobbies_sports,
						hobbies_games,
						hobbies_nature,
						hobbies_tinkering,
						hobbies_fitness,
						hobbies_science,
						hobbies_music,
						hobbies_reserved,
						preferred_hobby,
						lifetime_aspiration,
						lifetime_aspiration_points,
						lifetime_aspiration_points_spent,
						bugs_collection,
						..
					} = free_time_data;
					egui::Grid::new("SimDescription editor free time hobbies").show(ui, |ui| {
						macro_rules! hobby {
							($name:expr, $var:expr, $enum_var:expr) => {
								ui.label($name);
								ui.radio_value(preferred_hobby, $enum_var, "");
								ui.add(egui::Slider::new($var, 0..=1000));
								ui.end_row();
							};
						}

						ui.label("");
						ui.label("preferred");
						ui.label("enthusiasm");
						ui.end_row();
						hobby!("cooking", hobbies_cooking, PreferredHobby::Cooking);
						hobby!("arts", hobbies_arts, PreferredHobby::Arts);
						hobby!("film", hobbies_film, PreferredHobby::Film);
						hobby!("sports", hobbies_sports, PreferredHobby::Sports);
						hobby!("games", hobbies_games, PreferredHobby::Games);
						hobby!("nature", hobbies_nature, PreferredHobby::Nature);
						hobby!("tinkering", hobbies_tinkering, PreferredHobby::Tinkering);
						hobby!("fitness", hobbies_fitness, PreferredHobby::Fitness);
						hobby!("science", hobbies_science, PreferredHobby::Science);
						hobby!("music", hobbies_music, PreferredHobby::Music);
						ui.label("reserved");
						ui.label("");
						ui.add(egui::Slider::new(hobbies_reserved, 0..=1000));
						ui.end_row();
					});

					egui::Grid::new("SimDescription editor free time lta").show(ui, |ui| {
						drag_fn("lifetime aspiration", lifetime_aspiration, ui);
						drag_fn("lifetime aspiration points", lifetime_aspiration_points, ui);
						drag_fn(
							"lifetime aspiration points spent",
							lifetime_aspiration_points_spent,
							ui,
						);
					});

					egui::Grid::new("SimDescription editor free time bugs").show(ui, |ui| {
						macro_rules! bug_cb {
							($name:expr, $get_fn:ident, $set_fun:ident) => {
								ui.label("");
								checkbox_member_get_set(
									ui,
									$name,
									bugs_collection,
									BugCollectionFlags::$get_fn,
									BugCollectionFlags::$set_fun,
								);
								ui.end_row();
							};
						}

						ui.label("bug collection");
						checkbox_member_get_set(
							ui,
							"grey widow spider",
							bugs_collection,
							BugCollectionFlags::grey_widow_spider,
							BugCollectionFlags::set_grey_widow_spider,
						);
						ui.end_row();

						bug_cb!(
							"striped spindler spider",
							striped_spindler_spider,
							set_striped_spindler_spider
						);
						bug_cb!(
							"huntsperson spider",
							huntsperson_spider,
							set_huntsperson_spider
						);
						bug_cb!("teddybear spider", teddybear_spider, set_teddybear_spider);
						bug_cb!(
							"itsius bitsius spider",
							itsius_bitsius_spider,
							set_itsius_bitsius_spider
						);
						bug_cb!(
							"single fanged betsy spider",
							single_fanged_betsy_spider,
							set_single_fanged_betsy_spider
						);
						bug_cb!("hotdog spider", hotdog_spider, set_hotdog_spider);
						bug_cb!(
							"queen charlotte spider",
							queen_charlotte_spider,
							set_queen_charlotte_spider
						);
						bug_cb!(
							"paratrooper spider",
							paratrooper_spider,
							set_paratrooper_spider
						);
						bug_cb!("mock spider", mock_spider, set_mock_spider);
						bug_cb!(
							"socialus butterfly",
							socialus_butterfly,
							set_socialus_butterfly
						);
						bug_cb!(
							"blue featherwing butterfly",
							blue_featherwing_butterfly,
							set_blue_featherwing_butterfly
						);
						bug_cb!(
							"pygmalion butterfly",
							pygmalion_butterfly,
							set_pygmalion_butterfly
						);
						bug_cb!(
							"empress butterfly",
							empress_butterfly,
							set_empress_butterfly
						);
						bug_cb!("jelly butterfly", jelly_butterfly, set_jelly_butterfly);
						bug_cb!("peanut butterfly", peanut_butterfly, set_peanut_butterfly);
						bug_cb!(
							"margarina butterfly",
							margarina_butterfly,
							set_margarina_butterfly
						);
						bug_cb!(
							"copper pot butterfly",
							copper_pot_butterfly,
							set_copper_pot_butterfly
						);
						bug_cb!(
							"vampire butterfly",
							vampire_butterfly,
							set_vampire_butterfly
						);
						bug_cb!("madame butterfly", madame_butterfly, set_madame_butterfly);
						bug_cb!("prancer beetle", prancer_beetle, set_prancer_beetle);
						bug_cb!("jack beetle", jack_beetle, set_jack_beetle);
						bug_cb!(
							"mock ladybug beetle",
							mock_ladybug_beetle,
							set_mock_ladybug_beetle
						);
						bug_cb!("polka beetle", polka_beetle, set_polka_beetle);
						bug_cb!(
							"green bottle beetle",
							green_bottle_beetle,
							set_green_bottle_beetle
						);
						bug_cb!(
							"dapper pinstripe beetle",
							dapper_pinstripe_beetle,
							set_dapper_pinstripe_beetle
						);
						bug_cb!(
							"couch potato beetle",
							couch_potato_beetle,
							set_couch_potato_beetle
						);
						bug_cb!("ringo beetle", ringo_beetle, set_ringo_beetle);
						bug_cb!(
							"trihorn greaves beetle",
							trihorn_greaves_beetle,
							set_trihorn_greaves_beetle
						);
						bug_cb!("gentleman beetle", gentleman_beetle, set_gentleman_beetle);
					})
				})
			});

		egui::CollapsingHeader::new("Apartment Life")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::Grid::new("SimDescription editor apartment life").show(ui, |ui| {
					let ApartmentLifePreReleaseData {
						reputation,
						probability_to_appear,
					} = apartment_life_pre_release_data;
					if *version < Version::ApartmentLifePreRelease {
						ui.disable();
					}
					ui.label("reputation");
					ui.add(egui::Slider::new(reputation, -100..=100));
					ui.end_row();
					drag_fn("probability to appear", probability_to_appear, ui);

					let ApartmentLifeData { title_post_name } = apartment_life_data;
					if *version < Version::ApartmentLife {
						ui.disable();
					}
					ui.label("title post name");
					ComboBox::from_id_salt("title post name")
						.selected_text(format!("{:?}", title_post_name))
						.show_ui(ui, |ui| {
							for n in enum_iterator::all::<TitlePostName>() {
								ui.selectable_value(title_post_name, n, format!("{:?}", n));
							}
						});
					ui.end_row();
				})
			});

		let res = egui::CollapsingHeader::new("Internal")
			.default_open(false) // TODO global state
			.show(ui, |ui| {
				egui::CollapsingHeader::new("Interaction")
					.default_open(false) // TODO global state
					.show(ui, |ui| {
						egui::Grid::new("SimDescription editor interaction").show(ui, |ui| {
							drag_fn("data field 1", interaction_data_field_1, ui);
							drag_fn("data field 2", interaction_data_field_2, ui);

							drag_fn(
								"FindBestAction object ID",
								&mut interaction_findbestaction_object_id.id,
								ui,
							);

							drag_fn("current joinable", interaction_current_joinable, ui);
							drag_fn("autonomous", interaction_autonomous, ui);

							drag_fn("current GUID", interaction_current_guid, ui);
							drag_fn("linked deleted", interaction_linked_deleted, ui);

							drag_fn("current index", interaction_current_index, ui);
							drag_fn("next queued index", interaction_next_queued_index, ui);

							drag_fn(
								"next queued continuation",
								interaction_next_queued_continuation,
								ui,
							);

							drag_fn(
								"current running object ID",
								&mut interaction_current_running_object_id.id,
								ui,
							);
							drag_fn(
								"current running index",
								interaction_current_running_index,
								ui,
							);

							drag_fn(
								"queue current interaction object ID",
								&mut interaction_queue_current_interaction_object_id.id,
								ui,
							);
							drag_fn(
								"queue next interaction object ID",
								&mut interaction_queue_next_interaction_object_id.id,
								ui,
							);

							drag_fn("sub-queue count", interaction_sub_queue_count, ui);
							drag_fn(
								"sub-queue master interaction object ID",
								&mut interaction_sub_queue_master_interaction_object_id.id,
								ui,
							);
							drag_fn(
								"sub-queue master interaction index",
								interaction_sub_queue_master_interaction_index,
								ui,
							);
							drag_fn(
								"sub-queue next interaction object ID",
								&mut interaction_sub_queue_next_interaction_object_id.id,
								ui,
							);
							drag_fn(
								"sub-queue next interaction index",
								interaction_sub_queue_next_interaction_index,
								ui,
							);
							drag_fn(
								"sub-queue object ID",
								&mut interaction_sub_queue_object_id.id,
								ui,
							);
						})
					});

				egui::CollapsingHeader::new("Routing")
					.default_open(false) // TODO global state
					.show(ui, |ui| {
						egui::Grid::new("SimDescription editor routing").show(ui, |ui| {
							drag_fn("slot", route_slot, ui);
							drag_fn("start slot", route_start_slot, ui);
							drag_fn("multi slot index", route_multi_slot_index, ui);
							drag_fn("status", route_status, ui);
							drag_fn("goal", route_goal, ui);
							drag_fn("result", route_result, ui);

							ui.label("ghost flags");
							ui.horizontal(|ui| {
								checkbox_member_get_set(
									ui,
									"is ghost",
									route_ghost_flags,
									GhostFlags::is_ghost,
									GhostFlags::set_is_ghost,
								);
								checkbox_member_get_set(
									ui,
									"ignore traversal costs",
									route_ghost_flags,
									GhostFlags::ignore_traversal_costs,
									GhostFlags::set_ignore_traversal_costs,
								);
							});
							ui.end_row();
							ui.label("");
							ui.horizontal(|ui| {
								checkbox_member_get_set(
									ui,
									"force route recalc",
									route_ghost_flags,
									GhostFlags::force_route_recalc,
									GhostFlags::set_force_route_recalc,
								);
								checkbox_member_get_set(
									ui,
									"can swim in ocean",
									route_ghost_flags,
									GhostFlags::can_swim_in_ocean,
									GhostFlags::set_can_swim_in_ocean,
								);
							});
							ui.end_row();
							ui.label("pass through");
							ui.horizontal(|ui| {
								checkbox_member_get_set(
									ui,
									"objects",
									route_ghost_flags,
									GhostFlags::can_pass_through_objects,
									GhostFlags::set_can_pass_through_objects,
								);
								checkbox_member_get_set(
									ui,
									"walls",
									route_ghost_flags,
									GhostFlags::can_pass_through_walls,
									GhostFlags::set_can_pass_through_walls,
								);
								checkbox_member_get_set(
									ui,
									"people",
									route_ghost_flags,
									GhostFlags::can_pass_through_people,
									GhostFlags::set_can_pass_through_people,
								);
								checkbox_member_get_set(
									ui,
									"low objects",
									route_ghost_flags,
									GhostFlags::can_fly_over_low_objects,
									GhostFlags::set_can_fly_over_low_objects,
								);
							});
							ui.end_row();
						})
					});

				egui::CollapsingHeader::new("Loco")
					.default_open(false) // TODO global state
					.show(ui, |ui| {
						egui::Grid::new("SimDescription editor loco").show(ui, |ui| {
							drag_fn("weight 0", loco_weight_0, ui);
							drag_fn("weight 1", loco_weight_1, ui);
							drag_fn("personality index", loco_personality_index, ui);
							drag_fn("personality weight", loco_personality_weight, ui);
							drag_fn("mood index", loco_mood_index, ui);
							drag_fn("mood weight", loco_mood_weight, ui);
							drag_fn("motives", loco_motives, ui);

							// TODO ghost flags
						})
					});

				egui::CollapsingHeader::new("Look At")
					.default_open(false) // TODO global state
					.show(ui, |ui| {
						egui::Grid::new("SimDescription editor look at").show(ui, |ui| {
							drag_fn("object ID", &mut look_at_object_id.id, ui);
							drag_fn("slot ID", look_at_slot_id, ui);
							drag_fn("state", look_at_state, ui);
							drag_fn("time remaining", look_at_time_remaining, ui);
						})
					});

				egui::Grid::new("SimDescription editor misc").show(ui, |ui| {
					drag_fn("religion ID", religion_id, ui);

					ui.label("cult flags");
					checkbox_member_get_set(
						ui,
						"allow family",
						cult_flags,
						CultFlags::allow_family,
						CultFlags::set_allow_family,
					);
					ui.end_row();
					ui.label("");
					checkbox_member_get_set(
						ui,
						"no alcohol",
						cult_flags,
						CultFlags::no_alcohol,
						CultFlags::set_no_alcohol,
					);
					ui.end_row();
					ui.label("");
					checkbox_member_get_set(
						ui,
						"no auto woohoo",
						cult_flags,
						CultFlags::no_auto_woohoo,
						CultFlags::set_no_auto_woohoo,
					);
					ui.end_row();
					ui.label("");
					checkbox_member_get_set(
						ui,
						"marked sim",
						cult_flags,
						CultFlags::marked_sim,
						CultFlags::set_marked_sim,
					);
					ui.end_row();
					ui.label("");
					checkbox_member_get_set(
						ui,
						"not used f(?)",
						cult_flags,
						CultFlags::not_used_f,
						CultFlags::set_not_used_f,
					);
					ui.end_row();

					ui.label("selection flags");
					checkbox_member_get_set(
						ui,
						"selectable",
						selection_flags,
						SelectionFlags::selectable,
						SelectionFlags::set_selectable,
					);
					ui.end_row();
					ui.label("");
					checkbox_member_get_set(
						ui,
						"not selectable",
						selection_flags,
						SelectionFlags::not_selectable,
						SelectionFlags::set_not_selectable,
					);
					ui.end_row();
					ui.label("");
					checkbox_member_get_set(
						ui,
						"hide relationships",
						selection_flags,
						SelectionFlags::hide_relationships,
						SelectionFlags::set_hide_relationships,
					);
					ui.end_row();
					ui.label("");
					checkbox_member_get_set(
						ui,
						"holiday mate",
						selection_flags,
						SelectionFlags::holiday_mate,
						SelectionFlags::set_holiday_mate,
					);
					ui.end_row();

					ui.label("misc flags");
					checkbox_member_get_set(
						ui,
						"is owned",
						person_flags_1,
						PersonFlags1::is_owned,
						PersonFlags1::set_is_owned,
					);
					ui.end_row();
					ui.label("");
					checkbox_member_get_set(
						ui,
						"stay naked",
						person_flags_1,
						PersonFlags1::stay_naked,
						PersonFlags1::set_stay_naked,
					);
					ui.end_row();

					drag_fn("sitting", sitting, ui);
					drag_fn("swimming", swimming, ui);
					drag_fn("money over head", money_over_head, ui);
					drag_fn("group talk state", group_talk_state, ui);
					// drag_fn("job data", job_data, ui);
					drag_fn("motives static", motives_static, ui);
					drag_fn("censorship flags", censorship_flags, ui);
					drag_fn("neighbor id", neighbor_id, ui);
					drag_fn("person type", person_type, ui);
					drag_fn("priority", priority, ui);
					drag_fn("greet status", greet_status, ui);
					drag_fn("visitor_schedule", visitor_schedule, ui);
					drag_fn("ui icon flags", ui_icon_flags, ui);
					drag_fn("memory score", memory_score, ui);
					drag_fn("social menu object ID", &mut social_menu_object_id.id, ui);
					drag_fn("private", private, ui);
					drag_fn("lingering house number", lingering_house_number, ui);
					drag_fn("non interruptable", non_interruptable, ui);
					drag_fn("footprint extension", footprint_extension, ui);
					drag_fn("render display flags", render_display_flags, ui);
					drag_fn("tick counter", tick_counter, ui);

					drag_fn("life score teen", life_score_teen, ui);
					drag_fn("life score adult", life_score_adult, ui);
					drag_fn("life score elder", life_score_elder, ui);

					drag_fn("motive power", motive_power, ui);
					drag_fn("environment_score_override", environment_score_override, ui);
					drag_fn("fitness preference", fitness_preference, ui);

					drag_fn("interest unused 0", interest_unused_0, ui);
					drag_fn("interest unused 1", interest_unused_1, ui);
					drag_fn("allocated suburb", allocated_suburb, ui);
					drag_fn("interest unused 5", interest_unused_5, ui);
					drag_fn("interest unused 7", interest_unused_7, ui);
					drag_fn("interest unused 8", interest_unused_8, ui);
					drag_fn("interest unused 9", interest_unused_9, ui);
					drag_fn("interest unused 11", interest_unused_11, ui);

					drag_fn("unselectable", unselectable, ui);
					drag_fn("person flags 2", person_flags_2, ui);
					drag_fn("mood booster", mood_booster, ui);
					drag_fn("unlinked", unlinked, ui);

					drag_fn("unknown 0", unknown_0, ui);
					drag_fn("unknown 1", unknown_1, ui);
					drag_fn("unknown 3 ('end byte')", unknown_3, ui);
				});
			});

		// TODO changed
		res.header_response
	}
}

impl Editor for SimRelation {
	type EditorState = ();

	fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		ui.add(egui::DragValue::new(&mut self.relation.id))
			.on_hover_text("Relation sim ID")
			| ui.add(egui::DragValue::new(&mut self.unknown))
				.on_hover_text("Unknown")
	}
}
