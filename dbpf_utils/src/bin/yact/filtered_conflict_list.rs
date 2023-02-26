use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use eframe::Storage;
use serde::{Deserialize, Serialize};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf_utils::tgi_conflicts::TGIConflict;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConflictTypeFilterWarning {
    /// Conflicts with this type have been found that are not displayed
    NotVisible,
    /// All conflicts with this type are being shown
    FoundVisible,
    /// No conflicts of this type have been found
    NotFound,
}

#[derive(Clone, Eq, PartialEq, Hash, Default, Debug, Serialize, Deserialize)]
pub struct KnownConflict(pub PathBuf, pub PathBuf);

impl KnownConflict {
    fn is_known(&self, conflict: &TGIConflict) -> bool {
        conflict.original == self.0 && conflict.new == self.1
    }
}

#[derive(Clone, Default, Debug)]
pub struct FilteredConflictList {
    known_conflicts: Vec<KnownConflict>,
    show_known: bool,
    check_types: HashSet<KnownDBPFFileType>,

    found_conflicts: Vec<TGIConflict>,
    found_conflict_visible: HashMap<KnownDBPFFileType, ConflictTypeFilterWarning>,
    filtered_conflicts: Vec<TGIConflict>,
    has_hidden: bool,
}

impl FilteredConflictList {
    pub fn new(storage: &Option<&dyn Storage>) -> Self {
        let mut new = Self {
            show_known: true,
            check_types: Self::filter_defaults(),

            ..Default::default()
        };
        if let Some(storage) = storage {
            if let Some(known_conflicts_str) = storage
                .get_string("known_conflicts") {
                if let Ok(vec) = serde_json::from_str(known_conflicts_str.as_str()) {
                    new.known_conflicts = vec;
                }
            }

            if let Some(show_known) = storage
                .get_string("show_known")
                .and_then(|str| str.parse().ok()) {
                new.show_known = show_known;
            }

            for t in Self::filter_types() {
                if let Some(enable) = storage
                    .get_string(format!("check_{}", t.properties().abbreviation).as_str())
                    .and_then(|str| str.parse().ok()) {
                    new.set_check_enabled(&t, enable);
                }
            }
        }
        new
    }

    pub fn save(&mut self, storage: &mut dyn Storage) {
        if let Ok(str) = serde_json::to_string(&self.known_conflicts) {
            storage.set_string("known_conflicts", str);
        }

        storage.set_string("show_known", self.get_show_known().to_string());

        for t in FilteredConflictList::filter_types() {
            storage.set_string(format!("check_{}", t.properties().abbreviation).as_str(),
                               self.get_check_enabled(&t).to_string());
        }
    }

    fn filter_defaults() -> HashSet<KnownDBPFFileType> {
        HashSet::from([KnownDBPFFileType::SimanticsBehaviourConstant,
            KnownDBPFFileType::SimanticsBehaviourFunction,
            // KnownDBPFFileType::CatalogDescription,
            KnownDBPFFileType::GlobalData,
            KnownDBPFFileType::PropertySet,
            KnownDBPFFileType::ObjectData,
            KnownDBPFFileType::ObjectFunctions,
            KnownDBPFFileType::ObjectSlot,
            KnownDBPFFileType::TextList,
            // KnownDBPFFileType::EdithSimanticsBehaviourLabels,
            // KnownDBPFFileType::BehaviourConstantLabels,
            KnownDBPFFileType::PieMenuFunctions,
            KnownDBPFFileType::PieMenuStrings,
            // KnownDBPFFileType::VersionInformation
        ])
    }

    pub fn filter_types() -> [KnownDBPFFileType; 14] {
        [KnownDBPFFileType::SimanticsBehaviourConstant,
            KnownDBPFFileType::SimanticsBehaviourFunction,
            KnownDBPFFileType::CatalogDescription,
            KnownDBPFFileType::GlobalData,
            KnownDBPFFileType::PropertySet,
            KnownDBPFFileType::ObjectData,
            KnownDBPFFileType::ObjectFunctions,
            KnownDBPFFileType::ObjectSlot,
            KnownDBPFFileType::TextList,
            KnownDBPFFileType::EdithSimanticsBehaviourLabels,
            KnownDBPFFileType::BehaviourConstantLabels,
            KnownDBPFFileType::PieMenuFunctions,
            KnownDBPFFileType::PieMenuStrings,
            KnownDBPFFileType::VersionInformation]
    }

    pub fn get_known(&self) -> &Vec<KnownConflict> {
        &self.known_conflicts
    }

    pub fn add_known(&mut self, known: KnownConflict) -> bool {
        for self_known in &self.known_conflicts {
            if &known == self_known {
                return false;
            }
        }
        self.known_conflicts.push(known);
        self.re_filter();
        return true;
    }

    pub fn remove_known(&mut self, i: usize) {
        self.known_conflicts.remove(i);
        self.re_filter();
    }

    pub fn is_known(&self, conflict: &TGIConflict) -> bool {
        self.known_conflicts.iter().any(|known| known.is_known(conflict))
    }

    pub fn set_show_known(&mut self, show: bool) {
        self.show_known = show;
        self.re_filter();
    }

    pub fn get_show_known(&self) -> bool {
        self.show_known
    }

    pub fn get_check_enabled(&self, internal_type: &KnownDBPFFileType) -> bool {
        self.check_types.get(internal_type).is_some()
    }

    pub fn set_check_enabled(&mut self, internal_type: &KnownDBPFFileType, enabled: bool) {
        if enabled {
            self.check_types.insert(*internal_type);
        } else {
            self.check_types.remove(internal_type);
        }
        // this could maybe be faster but for now it's fast enough
        self.re_filter();
    }

    pub fn get_type_visibility(&self, internal_type: &KnownDBPFFileType) -> ConflictTypeFilterWarning {
        match self.found_conflict_visible.get(internal_type) {
            None => ConflictTypeFilterWarning::NotFound,
            Some(c) => c.clone(),
        }
    }

    pub fn has_hidden_conflicts(&self) -> bool {
        self.has_hidden
    }

    pub fn add(&mut self, conflict: TGIConflict) {
        self.found_conflicts.push(conflict.clone());
        self.filter_conflict(conflict);
    }

    fn filter_conflict(&mut self, conflict: TGIConflict) {
        let mut all_conflict_tgis = HashSet::new();
        let mut is_shown = false;
        let known = self.is_known(&conflict);
        for tgi in &conflict.tgis {
            if let DBPFFileType::Known(t) = tgi.type_id {
                if self.get_check_enabled(&t) &&
                    (self.show_known || !self.is_known(&conflict)) {
                    // type should be shown in filtered list
                    is_shown = true;
                }
                all_conflict_tgis.insert(t);
            }
        }
        if is_shown {
            // the conflict has passed the filter, show it
            self.filtered_conflicts.push(conflict);

            for known_t in all_conflict_tgis {
                if !self.found_conflict_visible.contains_key(&known_t) {
                    self.found_conflict_visible.insert(known_t, ConflictTypeFilterWarning::FoundVisible);
                }
            }
        } else if !known {
            // conflict was filtered out
            self.has_hidden = true;
            for known_t in all_conflict_tgis {
                self.found_conflict_visible.insert(known_t, ConflictTypeFilterWarning::NotVisible);
            }
        }
    }

    pub fn get_filtered(&self) -> &Vec<TGIConflict> {
        &self.filtered_conflicts
    }

    pub fn clear(&mut self) {
        self.found_conflicts = Vec::new();
        self.re_filter();
    }

    pub fn reset_filters(&mut self) {
        self.check_types = Self::filter_defaults();
        self.re_filter();
    }

    fn re_filter(&mut self) {
        self.found_conflict_visible = HashMap::new();
        self.filtered_conflicts = Vec::new();
        self.has_hidden = false;

        for conflict in self.found_conflicts.clone() {
            self.filter_conflict(conflict);
        }
    }
}
