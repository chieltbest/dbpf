# Changelog

## [0.2.0] - 2025-12-24

### 🚀 Features

- *(dbpf)* Add function to replace all textures with a new texture
- *(dbpf)* Add texture format recompression function
- Add ability to edit texture format/compression type
- *(dbpf)* Add support for BINX (Binary Index) resource type
- *(dbpf)* Add all known XML file types
- *(dbpf)* Add XML reader support for CPF resources
- *(dbpf)* Add generic decode support for all known CPF types
- Enable generic editor for CPF files
- *(dbpf)* Add groups cache file type
- *(dbpf)* Add support for decoding of Behaviour Function resources
- Add primitive Behaviour Function editor/viewer
- Add scan again button
- *(dbpf)* Add support for decoding bcon resources
- *(dbpf)* Add support for decoding AudioReference resources
- *(dbpf)* Add support for decoding ObjectFunctions resources
- *(dbpf)* Add basic support for decoding BehaviourConstantsLabels and BehaviourFunctionLabels resources
- *(dbpf)* Add api for adding mip levels
- Add buttons to add/remove mipmap levels
- Switch to using RawFileData debug print impl
- *(dbpf)* Add support for decoding ObjectData resources
- Add simple editor for ObjectData resource
- *(dbpf)* Add equality operator for all resource types
- Save original image when changing texture format
- Add zoom area to texture viewer
- Add preserve transparency option
- *(dbpf)* Add dds import function
- *(dbpf)* Add dds import conversion functions
- *(dbpf)* Add dds export function
- Add bidirectional texture shrink function
- *(dbpf)* Add GeometricDataContainer decoder
- Add proper parsing and display for more GMDC attributes
- Add GMDC glTF export
- Add resource export
- *(dbpf)* Implement sim description parser
- *(dbpf)* Add SimDescriptions to decoded resources
- Updater

### 🐛 Bug Fixes

- Correct conversion into raw and grayscale textures
- *(dbpf)* Fix reading of Sims 3/4 package files
- Properly write CPF file header
- Fix typo in Binary Index file write: stringindex -> stringsetidx
- *(dbpf)* Correct name of XML-encoded cpf writer signed integer type (AnyInt -> AnySint32)
- *(dbpf)* Fix reading and writing of all CPF files
- *(dbpf)* Add additional error on XML CPF write to ensure that the key attribute can have no control chararcters
- Fix writing of BigInt
- *(dbpf)* Fix reading of textlist resources
- *(dbpf)* Fix reading of rcol resources
- *(dbpf)* Fix NullString conversion to String
- Textures that have bad data are reported and will not crash the program
- *(dbpf)* Decompression of dxt textures that are not a multiple of 4
- *(dbpf)* Remove CatalogString from the list of TextList resources
- *(dbpf)* Scale down mipmaps to 1x1 pixels in all cases
- *(dbpf)* Properly write resource reference in cpf
- *(dbpf)* Remove RawFileData utf-8 debug print check
- *(dbpf)* Write textlist count in big endian for untagged string
- Do not write file_name_repeat if resource block version is not V9
- *(dbpf)* Premultiply alpha during texture shrink
- *(dbpf)* Retain color information for fully transparent pixels
- *(dbpf)* Also retain color information for rectangular shrink
- *(dbpf)* Correctly write data during dds export
- *(dbpf)* Fix alpha and luminance dds export header
- *(dbpf)* Fix divide by zero
- Rename Bulgarian -> Cyrillic
- Export all unknown attributes is glTF
- *(dbpf)* Fix missing/incorrect parsing information
- Keep resources popup open on clicks
- Remove vestigial dark/light mode code

### 📚 Documentation

- Add missing copyright headers
- Do not include dbpf_utils in yact and batl changelogs

### 🚜 Refactor

- Move internal_file.rs to module
- Properly calculate rcol index
- Calcalate mipmap levels from amount of textures
- *(dbpf)* Add mip_levels convenience function
- Rework language code support
- *(dbpf)* Add separate datatype enum for XML CPF version field
- *(dbpf)* Rename String -> PascalString
- *(dbpf)* Make PascalString take an integer type argument
- *(dbpf)* Add maximum length/padding argument to NullString
- Make BigString implementation an instance of PascalString
- *(dbpf)* Move behaviour related resources into separate module
- Make Timestamp and UserVersion fields public
- Clean up
- Rename texture formats to more closely align with the D3D texture format specification
- *(dbpf)* Change texture resource purpose to enum
- *(dbpf)* Make SizedVec struct and refactor string to use it
- Fix most clippy lints
- Add code formatting with rustfmt
- Add license and copyright information
- Move gltf-kun patch to workspace Cargo.toml
- Reverse z-buffer
- *(dbpf)* Change index instance_id to newtype
- *(dbpf)* Rename PDAT/Person Data to SDSC/Sim Description
- *(dbpf)* Correct more information in sim description
- *(dbpf)* More dbpf renames
- *(dbpf)* Add BugCollectionFlags
- *(dbpf)* Rename collection field to bug collection
- *(dbpf)* Split SDSC into multiple files
- *(dbpf)* Add mementos bitfield
- *(dbpf)* Add TitlePostName enum
- *(dbpf)* Split base game data into separate file
- *(dbpf)* Add human name to Version
- *(dbpf)* Add Sequence derive to all SDSC enums
- *(dbpf)* Derive Copy for all SDSC data
- *(dbpf)* More SDSC changes
- Make SDSC height range -100 to 100
- Dark/light mode and ui scale into shared settings
- *(dbpf)* Allow warning caused by binrw issue
- Move shared dependency versions to workspace configuration

### 🧪 Testing

- *(dbpf)* Add test for writing and then reading any CPF object
- *(dbpf)* Add test asserting that strings may be invalid utf-8
- *(dbpf)* Split CPF test to assert that XML CPF objects cannot contain invalid UTF-8
- *(dbpf)* Split xml test to assert writing and reading separately
- *(dbpf)* Add tests for string types
- *(dbpf)* Assert that a nullstring cannot have nulls

### 📦️ Dependencies

- *(dbpf)* Update refpack to 5.0.0
- Update binrw to 0.15
- Update modular-bitfield to 0.12
- Update refpack
- Update thiserror to 2.0
- Update miniz-oxide to 0.8
- Update log
- Change serde_json dependency
- Update egui and dependencies to 0.32
- *(dbpf)* Update modular_bitfield to 0.13
- *(dbpf)* Update gltf_kun and xmltree dependencies

## [0.1.2] - 2025-04-22

### 🐛 Bug Fixes

- Properly set conflict list min column width
- Temporarily fix double hover text by switching text wrap mode to extend
- Properly set maximum size for all columns in known conflict menu

### 📦️ Dependencies

- Finish egui 0.31 migration

## [0.1.1] - 2025-04-19

### 🚀 Features

- *(dbpf)* Add support for header V3 (Spore) files

### 📚 Documentation

- Add dependencies to changelog

### 📦️ Dependencies

- Update to egui 0.31

## [0.1.0] - 2025-04-19

### 📚 Documentation

- Set up release process
- Create readme
- Create changelog

<!-- generated by git-cliff -->
