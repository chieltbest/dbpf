# Changelog

## [0.4.0] - 2025-05-28

### 🚀 Features

- *(dbpf)* Add support for decoding of Behaviour Function resources
- Add primitive Behaviour Function editor/viewer
- Add primitive Behaviour Function editor/viewer
- Add open in hex editor option
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
- Remember the position and fraction of the last opened tab when no tabs are open
- Add save as button
- Save original image when changing texture format
- Add resource type tooltip
- Add header editor
- Add simple type filter
- Add right click filter on type option
- Add zoom area to texture viewer
- Add preserve transparency option
- *(dbpf)* Add dds import function
- *(dbpf)* Add dds import conversion functions
- Add dds import function
- *(dbpf)* Add dds export function
- Add dds export button
- Add simple node based bhav viewer

### ♿ Accessibility

- Swap the locations of the ui preference and save/load buttons
- Make the tab separator interaction area extra large

### 🐛 Bug Fixes

- Properly log crashes (panics) instead of discarding them
- Textures that have bad data are reported and will not crash the program
- Improve error handling
- *(dbpf)* Decompression of dxt textures that are not a multiple of 4
- Open utf-8 compatible files correctly in the hex editor
- *(dbpf)* Remove CatalogString from the list of TextList resources
- Change the highlight when opening the hex editor
- *(dbpf)* Scale down mipmaps to 1x1 pixels in all cases
- *(dbpf)* Properly write resource reference in cpf
- *(dbpf)* Remove RawFileData utf-8 debug print check
- *(dbpf)* Write textlist count in big endian for untagged string
- Do not write file_name_repeat if resource block version is not V9
- Remember closed tab position even when it was a list of tabs
- Make settings load more resilient to upgrades and corruption
- When a resource is already open just focus it
- Add extra column for delete resource button
- Do not show vertical scrollbar in index tab
- Recompress texture from original when changing mipmap levels
- Close context menu on filter on type clicked
- Fix wrong ObjectData version editor
- *(dbpf)* Premultiply alpha during texture shrink
- Store original texture as BGRA
- *(dbpf)* Retain color information for fully transparent pixels
- *(dbpf)* Also retain color information for rectangular shrink
- Fix off-by-one in zoom state mip select
- Adjust mipmap and zoom level when removing largest texture
- Store bgra generated mipmaps when drag-adding texture
- *(dbpf)* Correctly write data during dds export
- *(dbpf)* Fix alpha and luminance dds export header
- *(dbpf)* Fix divide by zero
- Set default resource type filter

### 📚 Documentation

- Update yape readme

### 🚜 Refactor

- *(editor)* Simplify Editor trait
- Make BigString implementation an instance of PascalString
- *(dbpf)* Move behaviour related resources into separate module
- Make Timestamp and UserVersion fields public
- Add editor width option to string editor
- Change string editors to explicit width
- Clean up
- Rename texture formats to more closely align with the D3D texture format specification

### 🎨 Styling

- Add frame around texture viewer
- Move filename into tab name and tooltip
- Change texture resource viewer background to a more neutral colour
- Add "filter" text to filter enable button

### 🧪 Testing

- *(dbpf)* Add tests for string types
- *(dbpf)* Assert that a nullstring cannot have nulls

## [0.3.2] - 2025-05-10

### 🚀 Features

- *(dbpf)* Add groups cache file type

### 🐛 Bug Fixes

- *(dbpf)* Correct name of XML-encoded cpf writer signed integer type (AnyInt -> AnySint32)
- *(dbpf)* Fix reading and writing of all CPF files
- Remove empty label in generic CPF editor
- *(dbpf)* Add additional error on XML CPF write to ensure that the key attribute can have no control chararcters
- Fix writing of BigInt
- *(dbpf)* Fix reading of textlist resources
- *(dbpf)* Fix reading of rcol resources
- *(dbpf)* Fix NullString conversion to String

### 🚜 Refactor

- Change file type editor to use shared implementation
- *(dbpf)* Add separate datatype enum for XML CPF version field
- *(dbpf)* Rename String -> PascalString
- *(dbpf)* Make PascalString take an integer type argument
- *(dbpf)* Add maximum length/padding argument to NullString

### 🧪 Testing

- *(dbpf)* Add test for writing and then reading any CPF object
- *(dbpf)* Add test asserting that strings may be invalid utf-8
- *(dbpf)* Split CPF test to assert that XML CPF objects cannot contain invalid UTF-8
- *(dbpf)* Split xml test to assert writing and reading separately

## [0.3.1] - 2025-05-01

### 🐛 Bug Fixes

- Properly write CPF file header
- Fix typo in Binary Index file write: stringindex -> stringsetidx

## [0.3.0] - 2025-04-30

### 🚀 Features

- *(dbpf)* Add all known XML file types
- *(dbpf)* Add XML reader support for CPF resources
- *(dbpf)* Add generic decode support for all known CPF types
- Enable generic editor for CPF files

### 🐛 Bug Fixes

- *(dbpf)* Fix reading of Sims 3/4 package files

### 📚 Documentation

- Update readme with supported editor types

### 🚜 Refactor

- Rework language code support

## [0.2.0] - 2025-04-26

### 🚀 Features

- *(dbpf)* Add function to replace all textures with a new texture
- Add ability to replace texture resource texture by dragging an image into the editor
- Allow editing creator ID
- *(dbpf)* Add texture format recompression function
- Add ability to edit texture format/compression type
- Allow editing texture resource purpose value
- *(dbpf)* Add support for BINX (Binary Index) resource type
- Add editor support for BINX (Binary Index) resources

### 🐛 Bug Fixes

- Correct conversion into raw and grayscale textures

### 🚜 Refactor

- Move internal_file.rs to module
- Properly calculate rcol index
- Calcalate mipmap levels from amount of textures
- *(dbpf)* Add mip_levels convenience function

## [0.1.2] - 2025-04-24

### 🚀 Features

- Add ability to open files from a path

## [0.1.1] - 2025-04-19

### 🚀 Features

- Log to a file in the config directory
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
