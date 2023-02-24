use std::collections::{HashMap, HashSet};
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

#[derive(Clone, Default, Debug)]
pub struct FilteredConflictList {
    check_types: HashSet<KnownDBPFFileType>,

    found_conflicts: Vec<TGIConflict>,
    found_conflict_visible: HashMap<KnownDBPFFileType, ConflictTypeFilterWarning>,
    filtered_conflicts: Vec<TGIConflict>,
}

impl FilteredConflictList {
    pub fn new() -> Self {
        let new = Self {
            check_types: Self::filter_defaults(),

            ..Default::default()
        };
        new
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
        self.found_conflicts.len() != self.filtered_conflicts.len()
    }

    pub fn add(&mut self, conflict: TGIConflict) {
        self.found_conflicts.push(conflict.clone());
        self.filter_conflict(conflict);
    }

    fn filter_conflict(&mut self, conflict: TGIConflict) {
        let mut all_conflict_tgis = HashSet::new();
        let mut is_shown = false;
        for tgi in &conflict.tgis {
            if let DBPFFileType::Known(t) = tgi.type_id {
                if self.get_check_enabled(&t) {
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
        } else {
            // conflict was filtered out
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

        for conflict in self.found_conflicts.clone() {
            self.filter_conflict(conflict);
        }
    }
}
