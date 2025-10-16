use crate::{OpenFileState, OpenResource};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::IndexEntry;
use dbpf_utils::async_execute;
use eframe::egui;
use eframe::egui::{Button, Color32, DroppedFile, Event, HoveredFile, Sense, Ui};
use egui_inbox::UiInbox;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use std::rc::Rc;
use tracing::error;

fn file_drop_area<R>(
	ui: &mut Ui,
	mut can_drop: impl FnMut(&HoveredFile) -> bool,
	contents: impl FnOnce(&mut Ui) -> R,
) -> (Vec<DroppedFile>, R) {
	let frame = egui::Frame::new();

	let mut prepared = frame.begin(ui);
	let inner = contents(&mut prepared.content_ui);
	let response = prepared.allocate_space(ui);

	let (any_can_drop, any_hovered, dropped_files) = ui.input_mut(|i| {
		(
			i.raw.hovered_files.extract_if(.., |f| can_drop(f)).count() > 0,
			!i.raw.hovered_files.is_empty(),
			i.raw
				.dropped_files
				.extract_if(.., |f| {
					can_drop(&HoveredFile {
						path: f.path.clone(),
						mime: f.mime.clone(),
					})
				})
				.collect::<Vec<_>>(),
		)
	});

	// TODO winit can't detect drag move events yet, wait for winit 0.32
	let style = if any_can_drop
	/*&& response.contains_pointer()*/
	{
		ui.visuals().widgets.active
	} else {
		ui.visuals().widgets.inactive
	};

	let mut stroke = style.bg_stroke;

	if any_hovered && !any_can_drop {
		// When dragging something else, show that it can't be dropped here:
		stroke.color = ui.visuals().disable(stroke.color);
	}

	prepared.frame.fill = Color32::TRANSPARENT;
	prepared.frame.stroke = stroke;

	prepared.paint(ui);

	(dropped_files, inner)
}

fn tgi_from_filename(name: &str) -> (Option<DBPFFileType>, Option<u32>, Option<u64>) {
	let mut t = None;
	let mut g = None;
	let mut i = None;

	let parts = name.split(".").collect::<Vec<_>>();

	let mut ext_lookup = BTreeMap::new();

	for t in enum_iterator::all::<KnownDBPFFileType>() {
		for ext in t.properties().extensions {
			ext_lookup.insert(ext, t);
		}
	}

	for i in 1..parts.len() {
		let extension = parts[i..parts.len()].join(".");
		if let Some(ty) = ext_lookup.get(extension.as_str()) {
			t = Some(DBPFFileType::Known(*ty));
			break;
		}
	}

	// TODO t from name
	// TOOD g from name
	// TODO i from name

	(t, g, i)
}

fn filename_from_tgi(t: DBPFFileType, g: u32, i: u64) -> String {
	let extension = t
		.extensions()
		.first()
		.cloned()
		.unwrap_or("simpe".to_string());

	format!(
		"{:08X}-{:08X}-{:08X}-{:08X}.{}",
		t.code(),
		(i & 0xFFFF_FFFF_0000_0000) >> 32,
		g,
		i & 0x0000_0000_FFFF_FFFF,
		extension,
	)
}

pub fn resource_import_overlay<R: Read + Seek, Res>(
	ui: &mut Ui,
	resource: &mut OpenResource,
	reader: &mut R,
	contents: impl FnOnce(&mut Ui, &mut OpenResource, &mut R) -> Res,
) -> (Res, bool) {
	let type_id = resource.data.type_id;
	let (dropped, response) = file_drop_area(
		ui,
		|hovered| {
			hovered
				.path
				.as_ref()
				.and_then(|p| p.file_name())
				.and_then(|name| name.to_str())
				.is_none_or(|name| {
					name.ends_with("simpe")
						|| type_id.extensions().iter().any(|ext| name.ends_with(ext))
				})
		},
		|ui| contents(ui, resource, reader),
	);

	if let Some(dropped) = dropped.first() {
		if let Ok(data) = resource.data.data(reader) {
			if let Ok(raw) = data.decompressed() {
				if let Some(file_bytes) = dropped
					.bytes
					.as_ref()
					.map(|b| Vec::from(b.as_ref()))
					.or_else(|| {
						dropped.path.as_ref().and_then(|p| {
							let mut file = File::open(p).ok()?;
							let mut buf = vec![];
							file.read_to_end(&mut buf).ok()?;
							Some(buf)
						})
					}) {
					raw.data = file_bytes;

					return (response, true);
				}
			}
		}
	}

	(response, false)
}

#[derive(Debug, Default)]
pub struct ExportResourceData {
	save_resource_inbox: UiInbox<Option<rfd::FileHandle>>,
	export_resource: Option<Rc<RefCell<OpenResource>>>,
}

impl ExportResourceData {
	pub fn update_import_index<R>(
		&mut self,
		ui: &mut Ui,
		open_file_state: &mut Option<OpenFileState>,
		contents: impl FnOnce(&mut Ui) -> R,
	) -> R {
		let (dropped, response) = file_drop_area(ui, |_| true, contents);
		if let Some(file_state) = open_file_state {
			for f in dropped {
				match tgi_from_filename(
					f.path
						.as_ref()
						.and_then(|p| p.file_name())
						.and_then(|name| name.to_str())
						.unwrap_or(f.name.as_str()),
				) {
					(Some(t), g, i) => {
						let g = g.unwrap_or(0x1C050000);
						let i = i.unwrap_or(0);

						/*let entry = IndexEntry::

						file_state.resources.push(Rc::new(RefCell::new(OpenResource {
							ui_deleted: false,
							data:
						})))*/
						// TODO add resource
					}
					_ => {
						// TODO use modal to ask for details
					}
				}
			}
		}
		response
	}

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

			let mut dialog = rfd::AsyncFileDialog::new().set_file_name(filename_from_tgi(
				type_id,
				res.group_id,
				res.instance_id,
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
