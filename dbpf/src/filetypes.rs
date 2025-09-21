// SPDX-FileCopyrightText: 2023-2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

// taken from http://simswiki.info/wiki.php?title=List_of_Sims_2_Formats_by_Type

use std::io::Cursor;

use binrw::{binrw, BinRead, BinWrite};
use enum_iterator::Sequence;

#[binrw]
#[brw(repr = u32)]
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Sequence)]
#[non_exhaustive]
pub enum KnownDBPFFileType {
	// UI
	UserInterface = 0x00000000,
	// WGRA
	WallGraph = 0x0A284D0B,
	// TRKS
	TrackSettings = 0x0B9EB87E,
	// LTXT
	LotDescription = 0x0BF999E7,
	// XMOL
	MeshOverlayXML = 0x0C1FE246,
	// BINX
	BinaryIndex = 0x0C560F39,
	// JPEG
	JPEGImage1 = 0x0C7E9A76,
	// POOL
	PoolSurface = 0x0C900FDB,
	// XFMD
	FaceModifierXML = 0x0C93E3DE,
	// BNFO
	BusinessInfo = 0x104F6A6E,
	// TXTR
	TextureResource = 0x1C4A276C,
	// INI, MP3, SPX1, XA, LTEXT
	Audio = 0x2026960B,
	// 5SC
	SceneNode = 0x25232B11,
	// 3ARY
	Array3D = 0x2A51171B,
	// XTOL
	TextureOverlayXML = 0x2C1FD8A1,
	// THUB
	FenceArchThumbnail = 0x2C30E040,
	// POPT
	PopupTracker = 0x2C310F46,
	// THUB
	FoundationOrPoolThumbnail = 0x2C43CBD4,
	// THUB
	DormerThumbnail = 0x2C488BCA,
	// XFNC
	FenceXML = 0x2CB230B8,
	// SCOR
	SimScores = 0x3053CF74,
	// BCON
	SimanticsBehaviourConstants = 0x42434F4E,
	// BHAV
	SimanticsBehaviourFunction = 0x42484156,
	// BMP
	BitmapImage = 0x424D505F,
	// CATS
	CatalogString = 0x43415453,
	// CIGE
	ImageLink = 0x43494745,
	// CTSS
	CatalogDescription = 0x43545353,
	// DGRP
	DrawGroup = 0x44475250,
	// FACE
	FaceProperties = 0x46414345,
	// FAMh
	FamilyData = 0x46414D68,
	// FAMI
	FamilyInformation = 0x46414D49,
	// FCNS
	GlobalTuningValues = 0x46434E53,
	// FWAV
	AudioReference = 0x46574156,
	// GLOB
	GlobalData = 0x474C4F42,
	// HOUS
	HouseData = 0x484F5553,
	// TXMT
	MaterialDefinition = 0x49596978,
	// WRLD
	WorldDatabase = 0x49FF7D76,
	// TMAP
	TerrainTextureMap = 0x4B58975B,
	// XSTN
	SkinToneXML = 0x4C158081,
	// MMAT
	MaterialOverride = 0x4C697E5A,
	// CINE
	CinematicScene = 0x4D51F042,
	// JPEG
	JPEGImage2 = 0x4D533EDD,
	// XFLR
	FloorXML = 0x4DCADB7E,
	// NGBH
	NeighborhoodData = 0x4E474248,
	// NREF
	NameReference = 0x4E524546,
	// NMAP
	NameMap = 0x4E6D6150,
	// OBJD
	ObjectData = 0x4F424A44,
	// OBJf
	ObjectFunctions = 0x4F424A66,
	// ObjM
	ObjectMetadata = 0x4F626A4D,
	// INIT
	InventoryItem = 0x4F6FD33D,
	// PALT
	ImageColorPalette = 0x50414C54,
	// PERS
	SimPersonalInformation = 0x50455253,
	// POSI
	StackScript = 0x504F5349,
	// PTBP
	PackageToolkit = 0x50544250,
	// SIMI
	SimInformation = 0x53494D49,
	// SLOT
	ObjectSlot = 0x534C4F54,
	// SPR2
	Sprites = 0x53505232,
	// STR
	TextList = 0x53545223,
	// TATT
	TATT = 0x54415454,
	// TPRP
	EdithSimanticsBehaviourLabels = 0x54505250,
	// TRCN
	BehaviourConstantsLabels = 0x5452434E,
	// TREE
	EdithFlowchartTrees = 0x54524545,
	// GROP
	GroupsCache = 0x54535053,
	// TTAB
	PieMenuFunctions = 0x54544142,
	// TTAs
	PieMenuStrings = 0x54544173,
	// XMTO
	MaterialObjectXML = 0x584D544F,
	// XOBJ
	ObjectClassDump = 0x584F424A,
	// SLUA
	SimPEObjectLua = 0x61754C1B,
	// 5EL
	EnvironmentCubeLighting = 0x6A97042F,
	// 2ARY
	Array2D = 0x6B943B43,
	// COLL
	Collection = 0x6C4F359D,
	// LOT
	LotInformation = 0x6C589723,
	// XFNU
	FaceNeutralXML = 0x6C93B566,
	// XNGB
	NeighbourhoodObjectXML = 0x6D619378,
	// WNTT
	WantsTreeItemXML = 0x6D814AFE,
	// Mobjt
	MainLotObjects = 0x6F626A74,
	// Mobjt
	UnlockableRewards = 0x7181C501,
	// AUDR
	AudioResource = 0x7B1ACFCD,
	// GMND
	GeometricNode = 0x7BA3838C,
	// PNG, TGA
	Image = 0x856DDBAC,
	// WLL
	WallLayer = 0x8A84D7B0,
	// XHTN
	HairToneXML = 0x8C1580B5,
	// THUMB
	WallThumbnail = 0x8C31125E,
	// THUMB
	FloorThumbnail = 0x8C311262,
	// JPEG
	JPEGImage3 = 0x8C3CE95A,
	// FAMT
	FamilyTies = 0x8C870743,
	// XFRG
	FaceRegionXML = 0x8C93BF6C,
	// XFCH
	FaceArchetypeXML = 0x8C93E35C,
	// PMAP
	PredictiveMap = 0x8CC0A14B,
	// SFX
	SoundEffects = 0x8DB5E4C2,
	// KEYD
	AcceleratorKeyDefinitions = 0xA2E3D533,
	// PDAT
	PersonData = 0xAACE2EFB,
	// FPL
	FencePostLayer = 0xAB4BA572,
	// ROOF
	RoofData = 0xAB9406AA,
	// NHTG
	NeighbourhoodTerrainGeometry = 0xABCB5DA4,
	// NHTR
	NeighborhoodTerrain = 0xABD0DC63,
	// 5LF
	LinearFogLighting = 0xAC06A66F,
	// 5DS
	DrawStateLighting = 0xAC06A676,
	// THUB
	Thumbnail = 0xAC2950C1,
	// GMDC
	GeometricDataContainer = 0xAC4F8687,
	// 3IDR
	IDReferenceFile = 0xAC506764,
	// SIMD
	SimDataXML = 0xAC598EAC,
	// NID
	NeighbourhoodID = 0xAC8A7A2E,
	// XROF
	RoofXML = 0xACA8EA06,
	// STXR
	SurfaceTexture = 0xACE46235,
	// NLO
	LightOverride = 0xADEE8D84,
	// TSSG
	TSSGSystem = 0xBA353CE1,
	// LGHT
	AmbientLight = 0xC9C81B9B,
	// LGHT
	DirectionalLight = 0xC9C81BA3,
	// LGHT
	PointLight = 0xC9C81BA9,
	// LGHT
	Spotlight = 0xC9C81BAD,
	// SMAP
	StringMap = 0xCAC4FC40,
	// VERT
	VertexLayer = 0xCB4387A1,
	// THUMB
	FenceThumbnail = 0xCC30CDF8,
	// SREL
	SimRelations = 0xCC364C2A,
	// THUMB
	ModularStairThumbnail = 0xCC44B5EC,
	// THUMB
	RoofThumbnail = 0xCC489E46,
	// THUMB
	ChimneyThumbnail = 0xCC48C51F,
	// XWLL
	WallXML = 0xCCA8E925,
	// LxNR
	FacialStructure = 0xCCCEF852,
	// MATSHAD
	MaxisMaterialShader = 0xCD7FE87A,
	// SWAF, WFR
	WantsAndFears = 0xCD95548E,
	// CREG
	ContentRegistry = 0xCDB467B8,
	// PBOP
	PetBodyOptions = 0xD1954460,
	// CRES
	CreationResource = 0xE519C933,
	// DIR
	DBPFDirectory = 0xE86B1EEF,
	// FX
	EffectsResourceTree = 0xEA5118B0,
	// GZPS
	PropertySet = 0xEBCF3E27,
	// SDNA
	SimDNA = 0xEBFEE33F,
	// VERS
	VersionInformation = 0xEBFEE342,
	// ATST
	AudioTestSettings = 0xEBFEE345,
	// THUB
	TerrainThumbnail = 0xEC3126C4,
	// HCAM
	HoodCamera = 0xEC44BDDC,
	// LIFO
	LevelInformation = 0xED534136,
	// XWNT
	WantsXML = 0xED7D7B4D,
	// THUMB
	AwningThumbnail = 0xF03D464C,
	// OBJT
	SingularLotObject = 0xFA1C39F7,
	// ANIM
	Animation = 0xFB00791E,
	// SHPE
	Shape = 0xFC6EB1F7,
}

// Unknown formats:
//
// 0x8B0C79D6,
// 0x9D796DB4,
// 0xB21BE28B,
// 0xCC2A6A34,
// 0xCC8A6A69,
// 0xF9F0C21,

#[binrw]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum DBPFFileType {
	Known(KnownDBPFFileType),
	Unknown(u32),
}

impl Default for DBPFFileType {
	fn default() -> Self {
		Self::Unknown(0)
	}
}

enum EmbeddedFilename {
	Embedded,
	No,
}

#[allow(dead_code)]
pub struct FileTypeProperties {
	/// Human readable name, subject to change at any time
	pub name: &'static str,
	/// Short (4 characters or less) abbreviation
	/// WARNING: abbreviations are not guaranteed to be unique
	pub abbreviation: &'static str,
	/// List of possible extension for this resource.
	/// The first extension is to be regarded as the preferred extension.
	pub extensions: Vec<&'static str>,
	/// Does the file have an embedded file name header
	pub embedded_filename: bool,
}

impl KnownDBPFFileType {
	pub fn properties(&self) -> FileTypeProperties {
		use EmbeddedFilename::*;
		use KnownDBPFFileType::*;
		let (name, abbreviation, extensions, embedded_filename) = match self {
			UserInterface => ("User Interface", "UI", vec!["ui.txt"], No),
			WallGraph => ("Wall Graph", "WGRA", vec![], No),
			TrackSettings => ("Track Settings", "TRKS", vec![], No),
			LotDescription => ("Lot Description", "LTXT", vec![], No),
			MeshOverlayXML => ("Mesh Overlay XML", "XMOL", vec!["mesh_overlay.xml"], No),
			BinaryIndex => ("Binary Index", "BINX", vec![], No),
			JPEGImage1 => ("JPEG Image", "JPEG", vec![], No),
			PoolSurface => ("Pool Surface", "POOL", vec![], No),
			FaceModifierXML => ("Face Modifier XML", "XFMD", vec!["face_mod.xml"], No),
			BusinessInfo => ("Business Info", "BNFO", vec![], No),
			TextureResource => ("Texture Resource", "TXTR", vec!["6tx"], No),
			// needs extensive file format checking
			// either an audio resource (mp3, Maxis eXtendable Audio, others)
			// or LTEXT localisation file
			Audio => ("Audio", "XA", vec![], No),
			SceneNode => ("Scene Node", "5SC", vec!["5sc"], No),
			Array3D => ("Array 3D", "3ARY", vec![], No),
			TextureOverlayXML => (
				"Texture Overlay XML",
				"XTOL",
				vec!["texture_overlay.xml"],
				No,
			),
			FenceArchThumbnail => (
				"Fence Arch Thumbnail",
				"THUB",
				vec!["fence_arch_thumb.jpg"],
				No,
			),
			PopupTracker => ("Popup Tracker", "POPT", vec![], No),
			FoundationOrPoolThumbnail => (
				"Foundation Or Pool Thumbnail",
				"THUB",
				vec!["pool_thumb.jpg"],
				No,
			),
			DormerThumbnail => ("Dormer Thumbnail", "THUB", vec!["dormer_thumb.jpg"], No),
			FenceXML => ("Fence XML", "XFNC", vec!["fence.xml"], No),
			SimScores => ("Sim Scores", "SCOR", vec![], No),
			SimanticsBehaviourConstants => {
				("Simantics Behaviour Constants", "BCON", vec![], Embedded)
			}
			SimanticsBehaviourFunction => {
				("Simantics Behaviour Function", "BHAV", vec![], Embedded)
			}
			BitmapImage => ("Bitmap Image", "BMP", vec!["bmp"], Embedded),
			CatalogString => ("Catalog String", "CATS", vec![], No),
			ImageLink => ("Image Link", "CIGE", vec![], No),
			CatalogDescription => ("Catalog Description", "CTSS", vec![], Embedded),
			DrawGroup => ("Draw Group", "DGRP", vec![], No),
			FaceProperties => ("Face Properties", "FACE", vec![], No),
			FamilyData => ("Family Data", "FAMH", vec![], No),
			FamilyInformation => ("Family Information", "FAMI", vec![], No),
			GlobalTuningValues => ("Global Tuning Values", "FCNS", vec![], No),
			AudioReference => ("Audio Reference", "FWAV", vec![], No),
			GlobalData => ("Global Data", "GLOB", vec![], Embedded),
			HouseData => ("House Data", "HOUS", vec![], No),
			MaterialDefinition => ("Material Definition", "TXMT", vec!["5tm"], No),
			WorldDatabase => ("World Database", "WRLD", vec![], No),
			TerrainTextureMap => ("Terrain Texture Map", "TMAP", vec![], No),
			SkinToneXML => ("Skin Tone XML", "XSTN", vec!["skin_tone.xml"], No),
			MaterialOverride => ("Material Override", "MMAT", vec![], No),
			CinematicScene => ("Cinematic Scene", "CINE", vec!["5cs"], No),
			JPEGImage2 => ("JPEG Image", "JPEG", vec!["1.jpg"], No),
			FloorXML => ("Floor XML", "XFLR", vec!["floor.xml"], No),
			NeighborhoodData => ("Neighborhood Data", "NGBH", vec![], No),
			NameReference => ("Name Reference", "NREF", vec![], Embedded),
			NameMap => ("Name Map", "NMAP", vec![], No),
			ObjectData => ("Object Data", "OBJD", vec![], Embedded),
			ObjectFunctions => ("Object Functions", "OBJF", vec![], Embedded),
			ObjectMetadata => ("Object Metadata", "OBJM", vec![], No),
			InventoryItem => ("Inventory Item", "INIT", vec![], No),
			ImageColorPalette => ("Image Color Palette", "PALT", vec![], No),
			SimPersonalInformation => ("Sim Personal Information", "PERS", vec![], No),
			StackScript => ("Stack Script", "POSI", vec![], No),
			PackageToolkit => ("Package Toolkit", "PTBP", vec![], No),
			SimInformation => ("Sim Information", "SIMI", vec![], No),
			ObjectSlot => ("Object Slot", "SLOT", vec![], No),
			Sprites => ("Sprites", "SPR2", vec![], Embedded),
			TextList => ("Text List", "STR", vec![], Embedded),
			TATT => ("TATT", "TATT", vec![], Embedded),
			EdithSimanticsBehaviourLabels => {
				("Edith Simantics Behaviour Labels", "TPRP", vec![], Embedded)
			}
			BehaviourConstantsLabels => ("Behaviour Constants Labels", "TRCN", vec![], No),
			EdithFlowchartTrees => ("Edith Flowchart Trees", "TREE", vec!["tree.txt"], Embedded),
			GroupsCache => ("Groups Cache", "GROP", vec![], No),
			PieMenuFunctions => ("Pie Menu Functions", "TTAB", vec![], Embedded),
			PieMenuStrings => ("Pie Menu Strings", "TTAS", vec![], Embedded),
			MaterialObjectXML => (
				"Material Object XML",
				"XMTO",
				vec!["material_object.xml"],
				No,
			),
			ObjectClassDump => ("Object Class Dump", "XOBJ", vec!["object.1.xml"], No),
			SimPEObjectLua => ("SimPE Object Lua", "SLUA", vec![], No),
			EnvironmentCubeLighting => ("Environment Cube Lighting", "5EL", vec![], No),
			Array2D => ("Array 2D", "2ARY", vec![], No),
			Collection => ("Collection", "COLL", vec![], No),
			LotInformation => ("Lot Information", "LOT", vec![], No),
			FaceNeutralXML => ("Face Neutral XML", "XFNU", vec!["face_neutral.xml"], No),
			NeighbourhoodObjectXML => (
				"Neighbourhood Object XML",
				"XNGB",
				vec!["neighbourhood_object.xml"],
				No,
			),
			WantsTreeItemXML => (
				"Wants Tree Item XML",
				"WNTT",
				vec!["wants_tree_item.xml"],
				No,
			),
			MainLotObjects => ("Main Lot Objects", "MOBJT", vec![], No),
			UnlockableRewards => ("Unlockable Rewards", "REWD", vec!["rewards.txt"], No),
			AudioResource => ("Audio Resource", "AUDR", vec![], No),
			GeometricNode => ("Geometric Node", "GMND", vec!["5gn"], No),
			// needs header detection
			Image => ("Image", "IMG", vec!["img.jpg"], No),
			WallLayer => ("Wall Layer", "WLL", vec![], No),
			HairToneXML => ("Hair Tone XML", "XHTN", vec!["hair_tone.xml"], No),
			WallThumbnail => ("Wall Thumbnail", "THUB", vec!["wall_thumb.jpg"], No),
			FloorThumbnail => ("Floor Thumbnail", "THUB", vec!["floor_thumb.jpg"], No),
			JPEGImage3 => ("JPEG Image", "JPEG", vec!["2.jpg"], No),
			FamilyTies => ("Family Ties", "FAMT", vec![], No),
			FaceRegionXML => ("Face Region XML", "XFRG", vec!["face_region.xml"], No),
			FaceArchetypeXML => ("Face Archetype XML", "XFCH", vec!["face_arch.xml"], No),
			PredictiveMap => ("Predictive Map", "PMAP", vec![], No),
			SoundEffects => ("Sound Effects", "SFX", vec![], No),
			AcceleratorKeyDefinitions => {
				("Accelerator Key Definitions", "KEYD", vec!["keys.txt"], No)
			}
			PersonData => ("Person Data", "PDAT", vec![], No),
			FencePostLayer => ("Fence Post Layer", "FPL", vec![], No),
			RoofData => ("Roof Data", "ROOF", vec![], No),
			NeighbourhoodTerrainGeometry => ("Neighbourhood Terrain Geometry", "NHTG", vec![], No),
			NeighborhoodTerrain => ("Neighborhood Terrain", "NHTR", vec![], No),
			LinearFogLighting => ("Linear Fog Lighting", "5LF", vec!["5lf"], No),
			DrawStateLighting => ("Draw State Lighting", "5DS", vec!["5ds"], No),
			Thumbnail => ("Thumbnail", "THUB", vec!["thumb.jpg"], No),
			GeometricDataContainer => ("Geometric Data Container", "GMDC", vec!["5gd", "gmdc"], No),
			IDReferenceFile => ("3D ID Referencing File", "3IDR", vec![], No),
			SimDataXML => ("Sim Data XML", "XSIM", vec![], No),
			NeighbourhoodID => ("Neighbourhood ID", "NID", vec![], No),
			RoofXML => ("Roof XML", "XROF", vec!["roof.xml"], No),
			SurfaceTexture => ("Surface Texture", "STXR", vec![], No),
			LightOverride => ("Light Override", "NLO", vec!["nlo"], No),
			TSSGSystem => ("TSSG System", "TSSG", vec![], No),
			AmbientLight => ("Ambient Light", "LGHT", vec!["5al"], No),
			DirectionalLight => ("Directional Light", "LGHT", vec!["5dl"], No),
			PointLight => ("Point Light", "LGHT", vec!["5pl"], No),
			Spotlight => ("Spotlight", "LGHT", vec!["5sl"], No),
			StringMap => ("String Map", "SMAP", vec![], No),
			VertexLayer => ("Vertex Layer", "VERT", vec![], No),
			FenceThumbnail => ("Fence Thumbnail", "THUB", vec!["fence_thumb.jpg"], No),
			SimRelations => ("Sim Relations", "SREL", vec![], No),
			ModularStairThumbnail => (
				"Modular Stair Thumbnail",
				"THUB",
				vec!["modular_stair_thumb.jpg"],
				No,
			),
			RoofThumbnail => ("Roof Thumbnail", "THUB", vec!["roof_thumbnail.jpg"], No),
			ChimneyThumbnail => (
				"Chimney Thumbnail",
				"THUB",
				vec!["chimney_thumbnail.jpg"],
				No,
			),
			WallXML => ("Wall XML", "XWLL", vec!["wall.xml"], No),
			FacialStructure => ("Facial Structure", "LXNR", vec![], No),
			MaxisMaterialShader => ("Maxis Material Shader", "MATSHAD", vec!["mat.txt"], No),
			WantsAndFears => ("Wants And Fears", "SWAF", vec![], No),
			ContentRegistry => ("Content Registry", "CREG", vec![], No),
			PetBodyOptions => ("Pet Body Options", "PBOP", vec![], No),
			CreationResource => ("Creation Resource", "CRES", vec!["5cr"], No),
			DBPFDirectory => ("DBPF Directory", "DIR", vec!["dir"], No),
			EffectsResourceTree => ("Effects Resource Tree", "FX", vec!["fx"], No),
			PropertySet => ("Property Set", "GZPS", vec![], No),
			SimDNA => ("Sim DNA", "SDNA", vec![], No),
			VersionInformation => ("Version Information", "VERS", vec![], No),
			AudioTestSettings => ("Audio Test Settings", "ATST", vec![], No),
			TerrainThumbnail => ("Terrain Thumbnail", "THUB", vec!["terrain_thumb.jpg"], No),
			HoodCamera => ("Hood Camera", "HCAM", vec![], No),
			LevelInformation => ("Level Information", "LIFO", vec!["6li"], No),
			WantsXML => ("Wants XML", "XWNT", vec!["wants.xml"], No),
			AwningThumbnail => ("Awning Thumbnail", "THUB", vec!["awning_thumb.jpg"], No),
			SingularLotObject => ("Singular Lot Object", "OBJT", vec![], No),
			Animation => ("Animation", "ANIM", vec!["5an"], No),
			Shape => ("Shape", "SHPE", vec!["5sh"], No),
		};
		FileTypeProperties {
			name,
			abbreviation,
			extensions,
			embedded_filename: matches!(embedded_filename, Embedded),
		}
	}
}

impl DBPFFileType {
	pub fn code(&self) -> u32 {
		match self {
			DBPFFileType::Known(t) => (*t) as u32,
			DBPFFileType::Unknown(n) => *n,
		}
	}

	pub fn properties(&self) -> Option<FileTypeProperties> {
		match self {
			DBPFFileType::Known(t) => Some(t.properties()),
			DBPFFileType::Unknown(_) => None,
		}
	}

	pub fn extensions(&self) -> Vec<String> {
		match self {
			Self::Known(t) => t
				.properties()
				.extensions
				.into_iter()
				.map(|s| s.to_string())
				.collect(),
			Self::Unknown(u) => vec![format!("{u:08X}")],
		}
	}

	pub fn abbreviation(&self) -> String {
		match self {
			Self::Known(t) => t.properties().abbreviation.to_string(),
			Self::Unknown(u) => format!("{u:08X}"),
		}
	}

	pub fn full_name(&self) -> String {
		match self {
			Self::Known(t) => t.properties().name.to_string(),
			Self::Unknown(u) => format!("{u:08X}"),
		}
	}
}

impl From<u32> for DBPFFileType {
	fn from(value: u32) -> Self {
		let mut bytes = Cursor::new(vec![]);
		value.write_le(&mut bytes).unwrap();
		bytes.set_position(0);
		<Self as BinRead>::read_le(&mut bytes).unwrap()
	}
}
