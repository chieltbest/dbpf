use crate::editor::Editor;
use eframe::egui::{Id, Response, Ui};
use eframe::{egui, glow};
use egui_dnd::{DragDropConfig, DragDropItem};
use std::fmt::Debug;
use std::sync::Arc;

// In order to use the index as id we need to implement DragDropItem for a wrapper struct
struct EnumeratedItem<T> {
	item: T,
	index: usize,
}

impl<T> DragDropItem for EnumeratedItem<T> {
	fn id(&self) -> Id {
		Id::new(self.index)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum VecEditorState<T: Editor>
where
	T::EditorState: Clone + Debug,
{
	Vec(Vec<T::EditorState>),
	Shared(T::EditorState),
}

impl<T: Editor> Default for VecEditorState<T>
where
	T::EditorState: Clone + Debug,
{
	fn default() -> Self {
		Self::Vec(vec![])
	}
}

impl<T: Editor + Default> Editor for Vec<T>
where
	T::EditorState: Clone + Debug + PartialEq,
{
	type EditorState = VecEditorState<T>;

	fn new_editor(
		&self,
		_context: &egui::Context,
		_gl_context: &Option<Arc<glow::Context>>,
	) -> Self::EditorState {
		VecEditorState::Vec(
			self.iter()
				.map(|elem| elem.new_editor(_context, _gl_context))
				.collect(),
		)
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let mut changed = false;

		let mut res = ui.scope(|ui| {
			let mut delete_index = None;

			let dnd_response = egui_dnd::dnd(ui, "generic vector editor")
				.with_mouse_config(DragDropConfig::mouse())
				.with_touch_config(Some(DragDropConfig::touch_scroll()))
				.with_animation_time(0.0)
				.show(
					self.iter_mut()
						.enumerate()
						.map(|(index, item)| EnumeratedItem { index, item }),
					|ui, EnumeratedItem { item, index }, handle, _item_state| {
						ui.horizontal(|ui| {
							handle.ui(ui, |ui| {
								let delete_button = ui.button("🗑");
								if delete_button.clicked() {
									delete_index = Some(index);
								}
							});

							let state = match state {
								VecEditorState::Vec(v) => &mut v[index],
								VecEditorState::Shared(s) => s,
							};

							let res = ui.push_id(index, |ui| item.show_editor(state, ui));
							if res.inner.changed() {
								changed = true;
							}
						});
					},
				);

			if dnd_response.is_drag_finished() {
				dnd_response.update_vec(self);
				if let VecEditorState::Vec(state_vec) = state {
					dnd_response.update_vec(state_vec);
				}
				changed = true;
			}

			// for now, assume that deletion cannot happen at the same time as drag and drop
			if let Some(delete_index) = delete_index {
				self.remove(delete_index);
				if let VecEditorState::Vec(state_vec) = state {
					state_vec.remove(delete_index);
				}
			}

			let add_button = ui.button("➕");
			if add_button.clicked() {
				let new = T::default();
				if let VecEditorState::Vec(v) = state {
					v.push(new.new_editor(ui.ctx(), &None));
				}
				self.push(new);
				changed = true;
			}
		});

		if changed {
			res.response.mark_changed();
		}

		res.response
	}
}
