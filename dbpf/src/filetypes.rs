// taken from http://simswiki.info/wiki.php?title=List_of_Sims_2_Formats_by_Type

use binrw::binrw;

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
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
    // JPEG
    JPEGImage1 = 0x0C7E9A76,
    // POOL
    PoolSurface = 0x0C900FDB,
    // XFMD
    FaceModifierXML = 0x0C93E3DE,
    // TXTR
    TextureResource = 0x1C4A276C,
    // INI, MP3, SPX1, XA
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
    // BCON
    SimanticsBehaviourConstant = 0x42434F4E,
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
    BCONLabels = 0x5452434E,
    // TREE
    EdithFlowchartTrees = 0x54524545,
    // TTAB
    PieMenuFunctions = 0x54544142,
    // TTAs
    PieMenuStrings = 0x54544173,
    // XMTO
    MaterialObjectXML = 0x584D544F,
    // XOBJ
    ObjectXML1 = 0x584F424A,
    // 5EL
    EnvironmentCubeLighting = 0x6A97042F,
    // 2ARY
    Array2D = 0x6B943B43,
    // LOT
    LotInformation = 0x6C589723,
    // XFNU
    FaceNeuralXML = 0x6C93B566,
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
    FaceArchXML = 0x8C93E35C,
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
    // SKIN
    SimOutfits = 0xAC506764,
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
    // XOBJ
    ObjectXML2 = 0xCCA8E925,
    // LxNR
    FacialStructure = 0xCCCEF852,
    // MATSHAD
    MaxisMaterialShader = 0xCD7FE87A,
    // SWAF, WFR
    WantsAndFears1 = 0xCD95548E,
    // CREG
    ContentRegistry = 0xCDB467B8,
    // CRES
    CreationResource = 0xE519C933,
    // DIR
    DBPFDirectory = 0xE86B1EEF,
    // FX
    EffectsResourceTree = 0xEA5118B0,
    // GZPS
    PropertySet = 0xEBCF3E27,
    // VERS
    VersionInformation = 0xEBFEE342,
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
// // UNK
// Unknown14 = 0x6C4F359D,
// // UNK
// Unknown6 = 0x8B0C79D6,
// // UNK
// Unknown7 = 0x9D796DB4,
// // UNK
// Unknown15 = 0xB21BE28B,
// // UNK
// Unknown9 = 0xCC2A6A34,
// // UNK
// Unknown10 = 0xCC8A6A69,
// // UNK
// Unknown16 = 0xEBFEE33F,
// // UNK
// Unknown11 = 0xEBFEE342,
// // UNK
// Unknown13 = 0xF9F0C21,

#[binrw]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum DBPFFileType {
    Known(KnownDBPFFileType),
    Unknown(u32),
}

enum EmbeddedFilename {
    Embedded,
    No,
}

#[allow(dead_code)]
pub struct FileTypeProperties {
    pub name: &'static str,
    pub abbreviation: &'static str,
    pub extension: Option<&'static str>,
    pub embedded_filename: bool,
}

impl KnownDBPFFileType {
    pub fn properties(&self) -> FileTypeProperties {
        use EmbeddedFilename::*;
        use KnownDBPFFileType::*;
        let (name, abbreviation, extension, embedded_filename) = match self {
            UserInterface => ("UserInterface", "", Some("ui.txt"), No),
            WallGraph => ("WallGraph", "", None, No),
            TrackSettings => ("TrackSettings", "", None, No),
            LotDescription => ("LotDescription", "", None, No),
            MeshOverlayXML => ("MeshOverlayXML", "", Some("mesh_overlay.xml"), No),
            JPEGImage1 => ("JPEGImage1", "", None, No),
            PoolSurface => ("PoolSurface", "", None, No),
            FaceModifierXML => ("FaceModifierXML", "", Some("face_mod.xml"), No),
            TextureResource => ("TextureResource", "", Some("6tx"), No),
            // might need header detection
            Audio => ("Audio", "", Some("mp3"), No),
            SceneNode => ("SceneNode", "", Some("5sc"), No),
            Array3D => ("Array3D", "", None, No),
            TextureOverlayXML => ("TextureOverlayXML", "", Some("texture_overlay.xml"), No),
            FenceArchThumbnail => ("FenceArchThumbnail", "", Some("fence_arch_thumb.jpg"), No),
            PopupTracker => ("PopupTracker", "", None, No),
            FoundationOrPoolThumbnail => ("FoundationOrPoolThumbnail", "", Some("pool_thumb.jpg"), No),
            DormerThumbnail => ("DormerThumbnail", "", Some("dormer_thumb.jpg"), No),
            FenceXML => ("FenceXML", "", Some("fence.xml"), No),
            SimanticsBehaviourConstant => ("SimanticsBehaviourConstant", "", None, Embedded),
            SimanticsBehaviourFunction => ("SimanticsBehaviourFunction", "", None, Embedded),
            BitmapImage => ("BitmapImage", "", Some("bmp"), Embedded),
            CatalogString => ("CatalogString", "", None, No),
            ImageLink => ("ImageLink", "", None, No),
            CatalogDescription => ("CatalogDescription", "", None, Embedded),
            DrawGroup => ("DrawGroup", "", None, No),
            FaceProperties => ("FaceProperties", "", None, No),
            FamilyData => ("FamilyData", "", None, No),
            FamilyInformation => ("FamilyInformation", "", None, No),
            GlobalTuningValues => ("GlobalTuningValues", "", None, No),
            AudioReference => ("AudioReference", "", None, No),
            GlobalData => ("GlobalData", "", None, Embedded),
            HouseData => ("HouseData", "", None, No),
            MaterialDefinition => ("MaterialDefinition", "", Some("5tm"), No),
            WorldDatabase => ("WorldDatabase", "", None, No),
            TerrainTextureMap => ("TerrainTextureMap", "", None, No),
            SkinToneXML => ("SkinToneXML", "", Some("skin_tone.xml"), No),
            MaterialOverride => ("MaterialOverride", "", None, No),
            CinematicScene => ("CinematicScene", "", Some("5cs"), No),
            JPEGImage2 => ("JPEGImage2", "", Some("1.jpg"), No),
            FloorXML => ("FloorXML", "", Some("floor.xml"), No),
            NeighborhoodData => ("NeighborhoodData", "", None, No),
            NameReference => ("NameReference", "", None, Embedded),
            NameMap => ("NameMap", "", None, No),
            ObjectData => ("ObjectData", "", None, Embedded),
            ObjectFunctions => ("ObjectFunctions", "", None, Embedded),
            ObjectMetadata => ("ObjectMetadata", "", None, No),
            ImageColorPalette => ("ImageColorPalette", "", None, No),
            SimPersonalInformation => ("SimPersonalInformation", "", None, No),
            StackScript => ("StackScript", "", None, No),
            PackageToolkit => ("PackageToolkit", "", None, No),
            SimInformation => ("SimInformation", "", None, No),
            ObjectSlot => ("ObjectSlot", "", None, No),
            Sprites => ("Sprites", "", None, Embedded),
            TextList => ("TextList", "", None, Embedded),
            TATT => ("TATT", "", None, Embedded),
            EdithSimanticsBehaviourLabels => ("EdithSimanticsBehaviourLabels", "", None, Embedded),
            BCONLabels => ("BCONLabels", "", None, No),
            EdithFlowchartTrees => ("EdithFlowchartTrees", "", Some("tree.txt"), Embedded),
            PieMenuFunctions => ("PieMenuFunctions", "", None, Embedded),
            PieMenuStrings => ("PieMenuStrings", "", None, Embedded),
            MaterialObjectXML => ("MaterialObjectXML", "", Some("material_object.xml"), No),
            ObjectXML1 => ("ObjectXML1", "", Some("object.1.xml"), No),
            EnvironmentCubeLighting => ("EnvironmentCubeLighting", "", None, No),
            Array2D => ("Array2D", "", None, No),
            LotInformation => ("LotInformation", "", None, No),
            FaceNeuralXML => ("FaceNeuralXML", "", Some("face_neural.xml"), No),
            NeighbourhoodObjectXML => ("NeighbourhoodObjectXML", "", Some("neighbourhood_object.xml"), No),
            WantsTreeItemXML => ("WantsTreeItemXML", "", Some("wants_tree_item.xml"), No),
            MainLotObjects => ("MainLotObjects", "", None, No),
            UnlockableRewards => ("UnlockableRewards", "", Some("rewards.txt"), No),
            AudioResource => ("AudioResource", "", None, No),
            GeometricNode => ("GeometricNode", "", Some("5gn"), No),
            // needs header detection
            Image => ("Image", "", Some("img.jpg"), No),
            WallLayer => ("WallLayer", "", None, No),
            HairToneXML => ("HairToneXML", "", Some("hair_tone.xml"), No),
            WallThumbnail => ("WallThumbnail", "", Some("wall_thumb.jpg"), No),
            FloorThumbnail => ("FloorThumbnail", "", Some("floor_thumb.jpg"), No),
            JPEGImage3 => ("JPEGImage3", "", Some("2.jpg"), No),
            FamilyTies => ("FamilyTies", "", None, No),
            FaceRegionXML => ("FaceRegionXML", "", Some("face_region.xml"), No),
            FaceArchXML => ("FaceArchXML", "", Some("face_arch.xml"), No),
            PredictiveMap => ("PredictiveMap", "", None, No),
            SoundEffects => ("SoundEffects", "", None, No),
            AcceleratorKeyDefinitions => ("AcceleratorKeyDefinitions", "", Some("keys.txt"), No),
            PersonData => ("PersonData", "", None, No),
            FencePostLayer => ("FencePostLayer", "", None, No),
            RoofData => ("RoofData", "", None, No),
            NeighbourhoodTerrainGeometry => ("NeighbourhoodTerrainGeometry", "", None, No),
            NeighborhoodTerrain => ("NeighborhoodTerrain", "", None, No),
            LinearFogLighting => ("LinearFogLighting", "", Some("5lf"), No),
            DrawStateLighting => ("DrawStateLighting", "", Some("5ds"), No),
            Thumbnail => ("Thumbnail", "", Some("thumb.jpg"), No),
            GeometricDataContainer => ("GeometricDataContainer", "", Some("5gd"), No),
            SimOutfits => ("SimOutfits", "", None, No),
            NeighbourhoodID => ("NeighbourhoodID", "", None, No),
            RoofXML => ("RoofXML", "", Some("roof.xml"), No),
            SurfaceTexture => ("SurfaceTexture", "", None, No),
            LightOverride => ("LightOverride", "", Some("nlo"), No),
            TSSGSystem => ("TSSGSystem", "", None, No),
            AmbientLight => ("AmbientLight", "", Some("5al"), No),
            DirectionalLight => ("DirectionalLight", "", Some("5dl"), No),
            PointLight => ("PointLight", "", Some("5pl"), No),
            Spotlight => ("Spotlight", "", Some("5sl"), No),
            StringMap => ("StringMap", "", None, No),
            VertexLayer => ("VertexLayer", "", None, No),
            FenceThumbnail => ("FenceThumbnail", "", Some("fence_thumb.jpg"), No),
            SimRelations => ("SimRelations", "", None, No),
            ModularStairThumbnail => ("ModularStairThumbnail", "", Some("modular_stair_thumb.jpg"), No),
            RoofThumbnail => ("RoofThumbnail", "", Some("roof_thumbnail.jpg"), No),
            ChimneyThumbnail => ("ChimneyThumbnail", "", Some("chimney_thumbnail.jpg"), No),
            ObjectXML2 => ("ObjectXML2", "", Some("object.2.xml"), No),
            FacialStructure => ("FacialStructure", "", None, No),
            MaxisMaterialShader => ("MaxisMaterialShader", "", Some("mat.txt"), No),
            WantsAndFears1 => ("WantsAndFears1", "", None, No),
            ContentRegistry => ("ContentRegistry", "", None, No),
            CreationResource => ("CreationResource", "", Some("5cr"), No),
            DBPFDirectory => ("DBPFDirectory", "", Some("dir"), No),
            EffectsResourceTree => ("EffectsResourceTree", "", Some("fx"), No),
            PropertySet => ("PropertySet", "GZPS", None, No),
            VersionInformation => ("VersionInformation", "VERS", None, No),
            TerrainThumbnail => ("TerrainThumbnail", "", Some("terrain_thumb.jpg"), No),
            HoodCamera => ("HoodCamera", "", None, No),
            LevelInformation => ("LevelInformation", "", Some("6li"), No),
            WantsXML => ("WantsXML", "", Some("wants.xml"), No),
            AwningThumbnail => ("AwningThumbnail", "", Some("awning_thumb.jpg"), No),
            SingularLotObject => ("SingularLotObject", "", None, No),
            Animation => ("Animation", "", Some("5an"), No),
            Shape => ("Shape", "", Some("5sh"), No),
        };
        FileTypeProperties {
            name,
            abbreviation,
            extension,
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
            DBPFFileType::Unknown(_) => None
        }
    }

    pub fn extension(&self) -> String {
        match self {
            Self::Known(t) => {
                t.properties().extension
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{t:?}"))
            }
            Self::Unknown(u) => format!("{u:08X}"),
        }
    }
}
