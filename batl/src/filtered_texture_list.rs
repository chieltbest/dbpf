use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

use eframe::egui::{ComboBox, Context, DragValue, Response, Ui, Window};
use eframe::Storage;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use dbpf::internal_file::resource_collection::texture_resource::TextureFormat;
use dbpf_utils::editor::{Editor, VecEditorState};

use crate::texture_finder::{deser_texture_format, FoundTexture, ser_texture_format, TextureId};

trait TextureFilterRule {
    /// filter the texture according to this rule, returns true if the texture should be shown
    fn filter(&self, tex: &FoundTexture) -> bool;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ComparisonType {
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Equal,
    NotEqual,
}

impl Display for ComparisonType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ComparisonType::Less => "<",
            ComparisonType::LessEq => "≤",
            ComparisonType::Greater => ">",
            ComparisonType::GreaterEq => "≥",
            ComparisonType::Equal => "=",
            ComparisonType::NotEqual => "≠",
        })
    }
}

impl ComparisonType {
    pub fn check(&self, x: usize, y: usize) -> bool {
        match self {
            ComparisonType::Less => x < y,
            ComparisonType::LessEq => x <= y,
            ComparisonType::Greater => x > y,
            ComparisonType::GreaterEq => x >= y,
            ComparisonType::Equal => x == y,
            ComparisonType::NotEqual => x != y,
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct SerTexFormat(TextureFormat);

impl Serialize for SerTexFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        ser_texture_format(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for SerTexFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deser_texture_format(deserializer).map(|t| t.into())
    }
}

impl From<TextureFormat> for SerTexFormat {
    fn from(value: TextureFormat) -> Self {
        Self(value)
    }
}

impl Into<TextureFormat> for SerTexFormat {
    fn into(self) -> TextureFormat {
        self.0
    }
}

fn ser_format_filter<S: Serializer>(t: &BTreeSet<TextureFormat>, ser: S) -> Result<S::Ok, S::Error> {
    let ser_btree: BTreeSet<SerTexFormat> = t.iter().map(|t| t.clone().into()).collect();

    ser_btree.serialize(ser)
}

fn deser_format_filter<'a, D>(d: D) -> Result<BTreeSet<TextureFormat>, D::Error> where D: Deserializer<'a> {
    Ok(BTreeSet::<SerTexFormat>::deserialize(d)?.iter().map(|t| t.0).collect())
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum TextureFilterOperation {
    Width(ComparisonType, usize),
    Height(ComparisonType, usize),
    Memory(ComparisonType, usize),
    #[serde(serialize_with="ser_format_filter", deserialize_with="deser_format_filter")]
    Format(BTreeSet<TextureFormat>),
    Mip(ComparisonType, usize),
}

impl TextureFilterRule for TextureFilterOperation {
    fn filter(&self, tex: &FoundTexture) -> bool {
        let (comp, y) = match self {
            TextureFilterOperation::Format(checked) => {
                return checked.contains(&tex.format);
            }
            TextureFilterOperation::Width(c, x) |
            TextureFilterOperation::Height(c, x) |
            TextureFilterOperation::Memory(c, x) |
            TextureFilterOperation::Mip(c, x) => (c, *x),
        };
        let x = match self {
            TextureFilterOperation::Width(_, _) => tex.width as usize,
            TextureFilterOperation::Height(_, _) => tex.height as usize,
            TextureFilterOperation::Memory(_, _) => tex.memory_size,
            TextureFilterOperation::Mip(_, _) => tex.mip_levels as usize,
            _ => 0,
        };
        comp.check(x, y)
    }
}

impl Default for TextureFilterOperation {
    fn default() -> Self {
        Self::Width(ComparisonType::Greater, 512)
    }
}

impl Into<usize> for TextureFilterOperation {
    fn into(self) -> usize {
        match self {
            TextureFilterOperation::Width(_, _) => 0,
            TextureFilterOperation::Height(_, _) => 1,
            TextureFilterOperation::Memory(_, _) => 2,
            TextureFilterOperation::Format(_) => 3,
            TextureFilterOperation::Mip(_, _) => 4,
        }
    }
}

impl Editor for TextureFilterOperation {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut selected: usize = self.clone().into();

        let ires = ComboBox::from_id_salt(ui.id().with(0))
            .selected_text(match selected {
                0 => "Width",
                1 => "Height",
                2 => "Memory",
                3 => "Format",
                _ => "Mip levels",
            })
            .show_ui(ui, |ui| {
                [
                    ui.selectable_value(&mut selected, 0, "Width"),
                    ui.selectable_value(&mut selected, 1, "Height"),
                    ui.selectable_value(&mut selected, 2, "Memory"),
                    ui.selectable_value(&mut selected, 3, "Format"),
                    ui.selectable_value(&mut selected, 4, "Mip levels")
                ].into_iter()
                    .reduce(|r1, r2| r1.union(r2))
                    .unwrap()
            });

        if ires.inner.as_ref().is_some_and(|r| r.changed()) {
            match selected {
                0 => *self = Self::Width(ComparisonType::Greater, 512),
                1 => *self = Self::Height(ComparisonType::Greater, 512),
                2 => *self = Self::Memory(ComparisonType::Greater, 1_000_000),
                3 => *self = Self::Format(BTreeSet::from([
                    TextureFormat::RawRGB24,
                    TextureFormat::RawARGB32,
                    TextureFormat::AltRGB24,
                    TextureFormat::AltARGB32,
                ])),
                _ => *self = Self::Mip(ComparisonType::Greater, 1),
            }
        }

        let mut res = ires.response;
        if let Some(inner) = ires.inner {
            res |= inner;
        }

        res |= match self {
            TextureFilterOperation::Width(c, y) |
            TextureFilterOperation::Height(c, y) |
            TextureFilterOperation::Memory(c, y) |
            TextureFilterOperation::Mip(c, y) => {
                let res = ComboBox::from_id_salt(ui.id().with(1))
                    .width(40.0)
                    .selected_text(format!("{c}"))
                    .show_ui(ui, |ui| {
                        [
                            ComparisonType::Less,
                            ComparisonType::LessEq,
                            ComparisonType::Greater,
                            ComparisonType::GreaterEq,
                            ComparisonType::Equal,
                            ComparisonType::NotEqual,
                        ].into_iter().map(|ct| {
                            ui.selectable_value(c, ct, format!("{ct}"))
                        }).reduce(|r1, r2| r1 | r2).unwrap()
                    });
                let mut res = if let Some(inner) = res.inner {
                    res.response | inner
                } else {
                    res.response
                };
                res |= ui.add(DragValue::new(y));
                res
            }
            TextureFilterOperation::Format(checked) => {
                let mut inner = None;
                let res = ui.menu_button("choose", |ui| {
                    let res = [TextureFormat::Alpha,
                        TextureFormat::Grayscale,
                        TextureFormat::RawRGB24,
                        TextureFormat::RawARGB32,
                        TextureFormat::AltRGB24,
                        TextureFormat::AltARGB32,
                        TextureFormat::DXT1,
                        TextureFormat::DXT3,
                        TextureFormat::DXT5].iter().map(|tf| {
                        let mut new = checked.contains(&tf);
                        let res = ui.checkbox(&mut new, format!("{tf:?}"));
                        if res.changed() {
                            if new {
                                checked.insert(*tf);
                            } else {
                                checked.remove(tf);
                            }
                        }
                        res
                    }).reduce(|r1, r2| r1 | r2).unwrap();
                    inner = Some(res);
                });

                if let Some(inner) = inner {
                    res.response | inner
                } else {
                    res.response
                }
            }
        };

        res
    }
}

#[derive(Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct TextureFilter {
    pub operations: Vec<TextureFilterOperation>,
}

impl TextureFilterRule for TextureFilter {
    fn filter(&self, tex: &FoundTexture) -> bool {
        self.operations.iter().find(|filter| !filter.filter(tex)).is_none()
    }
}

impl Editor for TextureFilter {
    type EditorState = VecEditorState<TextureFilterOperation>;

    fn new_editor(&self, context: &Context) -> Self::EditorState {
        self.operations.new_editor(context)
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        self.operations.show_editor(state, ui)
    }
}

#[derive(Clone, Default, Debug)]
pub struct FilteredTextureList {
    known_textures: Vec<TextureId>,
    show_known: bool,
    texture_filter: TextureFilter,
    texture_filter_ui_state: Option<<TextureFilter as Editor>::EditorState>,
    open_texture_filter_ui: bool,

    found_textures: Vec<FoundTexture>,
    filtered_textures: Vec<FoundTexture>,
}

impl FilteredTextureList {
    pub fn new(storage: &Option<&dyn Storage>) -> Self {
        let mut new = Self {
            show_known: true,

            ..Default::default()
        };
        if let Some(storage) = storage {
            if let Some(known_textures_str) = storage
                .get_string("known_textures") {
                if let Ok(vec) = serde_json::from_str(known_textures_str.as_str()) {
                    new.known_textures = vec;
                }
            }

            if let Some(show_known) = storage
                .get_string("show_known")
                .and_then(|str| str.parse().ok()) {
                new.show_known = show_known;
            }

            if let Some(filter) = storage.get_string("filter_list")
                .and_then(|str| serde_json::from_str(&str).ok()) {
                new.texture_filter = filter;
            }
        }
        new
    }

    pub fn save(&mut self, storage: &mut dyn Storage) {
        if let Ok(str) = serde_json::to_string(&self.known_textures) {
            storage.set_string("known_textures", str);
        }

        storage.set_string("show_known", self.get_show_known().to_string());

        storage.set_string("filter_list", serde_json::to_string(&self.texture_filter).unwrap());
    }

    pub fn show_filter_menu(&mut self, ui: &mut Ui) {
        let res = Window::new("Filter List")
            .resizable(false)
            .open(&mut self.open_texture_filter_ui)
            .show(ui.ctx(), |ui| {
                let state = self.texture_filter_ui_state.get_or_insert_with(|| {
                    self.texture_filter.new_editor(ui.ctx())
                });
                self.texture_filter.show_editor(state, ui)
            });
        res.map(|r|
            r.inner.map(|inner|
                inner.changed().then(|| {
                    self.re_filter();
                })));

        ui.button("Filter")
            .on_hover_text("The filters that are being applied to the found texture list")
            .clicked().then(|| {
            self.open_texture_filter_ui = !self.open_texture_filter_ui;
        });
    }

    pub fn get_known(&self) -> &Vec<TextureId> {
        &self.known_textures
    }

    pub fn add_known(&mut self, known: TextureId) -> bool {
        for self_known in &self.known_textures {
            if &known == self_known {
                return false;
            }
        }
        self.known_textures.push(known);
        self.re_filter();
        return true;
    }

    pub fn remove_known(&mut self, i: usize) {
        self.known_textures.remove(i);
        self.re_filter();
    }

    pub fn is_known(&self, found: &FoundTexture) -> bool {
        self.known_textures.iter().any(|known| *known == found.id)
    }

    pub fn set_show_known(&mut self, show: bool) {
        self.show_known = show;
        self.re_filter();
    }

    pub fn get_show_known(&self) -> bool {
        self.show_known
    }

    pub fn add(&mut self, found: FoundTexture) {
        self.found_textures.push(found.clone());
        if self.filter_texture(&found) {
            self.filtered_textures.push(found);
        }
    }

    fn filter_texture(&self, found: &FoundTexture) -> bool {
        self.texture_filter.filter(found) && (self.show_known || !self.is_known(found))
    }

    pub fn get_filtered(&self) -> &Vec<FoundTexture> {
        &self.filtered_textures
    }

    pub fn clear(&mut self) {
        self.found_textures = Vec::new();
        self.re_filter();
    }

    fn re_filter(&mut self) {
        self.filtered_textures = Vec::new();

        self.filtered_textures = self.found_textures.iter()
            .filter_map(|tex| {
                self.filter_texture(tex).then(|| tex.clone())
            })
            .collect();
    }
}
