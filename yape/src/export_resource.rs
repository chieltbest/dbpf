use crate::{OpenFileState, OpenResource};
use dbpf_utils::async_execute;
use eframe::egui;
use eframe::egui::{Button, Ui};
use egui_inbox::UiInbox;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use tracing::error;

#[derive(Debug, Default)]
pub struct ExportResourceData {
	save_resource_inbox: UiInbox<Option<rfd::FileHandle>>,
	export_resource: Option<Rc<RefCell<OpenResource>>>,
}

impl ExportResourceData {
	/// call once per frame
	pub fn update(&mut self, open_file_state: &mut Option<OpenFileState>, ctx: &egui::Context) {
		if let Some(message) = self.save_resource_inbox.read(ctx).last() {
			if let Some(handle) = message {
				if let Some(res) = &self.export_resource {
					if let Some(open_file_state) = open_file_state {
						let mut res = res.borrow_mut();
						match res
							.data
							.data(&mut open_file_state.bytes)
							.map_err(|err| err.into())
							.and_then(|data| data.decompressed())
						{
							Err(error) => error!(%error),
							Ok(bytes) => {
								let bytes = bytes.clone();
								async_execute(async move {
									if let Err(error) = handle.write(&bytes.data).await {
										error!(%error);
									}
								});
							}
						}
					}
				}
			}

			self.export_resource = None;
		}
	}

	/// call inside a context menu
	pub fn button<P: AsRef<Path>>(
		&mut self,
		resource: &Rc<RefCell<OpenResource>>,
		open_file_path: &Option<P>,
		ui: &mut Ui,
	) {
		if ui
			.add_enabled(self.export_resource.is_none(), Button::new("Export..."))
			.clicked()
		{
			self.export_resource = Some(resource.clone());

			let res = &resource.borrow().data;
			let type_id = res.type_id;
			let extensions = type_id.extensions();

			let export_extension = extensions.first().cloned().unwrap_or("simpe".to_string());

			let mut dialog = rfd::AsyncFileDialog::new().set_file_name(format!(
				"{:08X}-{:08X}-{:08X}-{:08X}.{}",
				res.type_id.code(),
				(res.instance_id & 0xFFFF_FFFF_0000_0000) >> 32,
				res.group_id,
				res.instance_id & 0x0000_0000_FFFF_FFFF,
				export_extension,
			));

			if !extensions.is_empty() {
				let filter_name = format!("{} ({})", type_id.full_name(), extensions.join(", "));
				dialog = dialog.add_filter(filter_name, &extensions);
			}

			dialog = dialog.add_filter("All files", &[""]);

			if let Some(path) = open_file_path.as_ref().and_then(|p| p.as_ref().parent()) {
				dialog = dialog.set_directory(path);
			}
			let dialog = dialog.save_file();

			let sender = self.save_resource_inbox.sender();
			async_execute(async move {
				let _ = sender.send(dialog.await);
			});
		}
	}
}
