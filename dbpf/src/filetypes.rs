// taken from http://simswiki.info/wiki.php?title=List_of_Sims_2_Formats_by_Type

use binrw::binrw;

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
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
    BehaviourConstantLabels = 0x5452434E,
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
    WantsAndFears = 0xCD95548E,
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
// 0x6C4F359D,
// 0x8B0C79D6,
// 0x9D796DB4,
// 0xB21BE28B,
// 0xCC2A6A34,
// 0xCC8A6A69,
// 0xEBFEE33F,
// 0xEBFEE342,
// 0xF9F0C21,

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
    /// Human readable name, subject to change at any time
    pub name: &'static str,
    /// Short (4 characters or less) abbreviation
    /// WARNING: abbreviations are not guaranteed to be unique
    pub abbreviation: &'static str,
    /// File extension (unique)
    pub extension: Option<&'static str>,
    /// Does the file have an embedded file name header
    pub embedded_filename: bool,
}

impl KnownDBPFFileType {
    pub fn properties(&self) -> FileTypeProperties {
        use EmbeddedFilename::*;
        use KnownDBPFFileType::*;
        let (name, abbreviation, extension, embedded_filename) = match self {
            UserInterface => ("User Interface", "UI", Some("ui.txt"), No),
            WallGraph => ("Wall Graph", "WGRA", None, No),
            TrackSettings => ("Track Settings", "TRKS", None, No),
            LotDescription => ("Lot Description", "LTXT", None, No),
            MeshOverlayXML => ("Mesh Overlay XML", "XMOL", Some("mesh_overlay.xml"), No),
            JPEGImage1 => ("JPEG Image", "JPEG", None, No),
            PoolSurface => ("Pool Surface", "POOL", None, No),
            FaceModifierXML => ("Face Modifier XML", "XFMD", Some("face_mod.xml"), No),
            TextureResource => ("Texture Resource", "TXTR", Some("6tx"), No),
            // might need header detection
            Audio => ("Audio", "AUD", Some("mp3"), No),
            SceneNode => ("Scene Node", "5SC", Some("5sc"), No),
            Array3D => ("Array 3D", "3ARY", None, No),
            TextureOverlayXML => ("Texture Overlay XML", "XTOL", Some("texture_overlay.xml"), No),
            FenceArchThumbnail => ("Fence Arch Thumbnail", "THUB", Some("fence_arch_thumb.jpg"), No),
            PopupTracker => ("Popup Tracker", "POPT", None, No),
            FoundationOrPoolThumbnail => ("Foundation Or Pool Thumbnail", "THUB", Some("pool_thumb.jpg"), No),
            DormerThumbnail => ("Dormer Thumbnail", "THUB", Some("dormer_thumb.jpg"), No),
            FenceXML => ("FenceXML", "XFNC", Some("fence.xml"), No),
            SimanticsBehaviourConstant => ("Simantics Behaviour Constant", "BCON", None, Embedded),
            SimanticsBehaviourFunction => ("Simantics Behaviour Function", "BHAV", None, Embedded),
            BitmapImage => ("Bitmap Image", "BMP", Some("bmp"), Embedded),
            CatalogString => ("Catalog String", "CATS", None, No),
            ImageLink => ("Image Link", "CIGE", None, No),
            CatalogDescription => ("Catalog Description", "CTSS", None, Embedded),
            DrawGroup => ("Draw Group", "DGRP", None, No),
            FaceProperties => ("Face Properties", "FACE", None, No),
            FamilyData => ("Family Data", "FAMH", None, No),
            FamilyInformation => ("Family Information", "FAMI", None, No),
            GlobalTuningValues => ("Global Tuning Values", "FCNS", None, No),
            AudioReference => ("Audio Reference", "FWAV", None, No),
            GlobalData => ("Global Data", "GLOB", None, Embedded),
            HouseData => ("House Data", "HOUS", None, No),
            MaterialDefinition => ("Material Definition", "TXMT", Some("5tm"), No),
            WorldDatabase => ("World Database", "WRLD", None, No),
            TerrainTextureMap => ("Terrain Texture Map", "TMAP", None, No),
            SkinToneXML => ("Skin Tone XML", "XSTN", Some("skin_tone.xml"), No),
            MaterialOverride => ("Material Override", "MMAT", None, No),
            CinematicScene => ("Cinematic Scene", "CINE", Some("5cs"), No),
            JPEGImage2 => ("JPEG Image", "JPEG", Some("1.jpg"), No),
            FloorXML => ("Floor XML", "XFLR", Some("floor.xml"), No),
            NeighborhoodData => ("Neighborhood Data", "NGBH", None, No),
            NameReference => ("Name Reference", "NREF", None, Embedded),
            NameMap => ("Name Map", "NMAP", None, No),
            ObjectData => ("Object Data", "OBJD", None, Embedded),
            ObjectFunctions => ("Object Functions", "OBJF", None, Embedded),
            ObjectMetadata => ("Object Metadata", "OBJM", None, No),
            ImageColorPalette => ("Image Color Palette", "PALT", None, No),
            SimPersonalInformation => ("Sim Personal Information", "PERS", None, No),
            StackScript => ("Stack Script", "POSI", None, No),
            PackageToolkit => ("Package Toolkit", "PTBP", None, No),
            SimInformation => ("Sim Information", "SIMI", None, No),
            ObjectSlot => ("Object Slot", "SLOT", None, No),
            Sprites => ("Sprites", "SPR2", None, Embedded),
            TextList => ("Text List", "STR", None, Embedded),
            TATT => ("TATT", "TATT", None, Embedded),
            EdithSimanticsBehaviourLabels => ("Edith Simantics Behaviour Labels", "TPRP", None, Embedded),
            BehaviourConstantLabels => ("Behaviour Constant Labels", "TRCN", None, No),
            EdithFlowchartTrees => ("Edith Flowchart Trees", "TREE", Some("tree.txt"), Embedded),
            PieMenuFunctions => ("Pie Menu Functions", "TTAB", None, Embedded),
            PieMenuStrings => ("Pie Menu Strings", "TTAS", None, Embedded),
            MaterialObjectXML => ("Material Object XML", "XMTO", Some("material_object.xml"), No),
            ObjectXML1 => ("Object XML", "XOBJ", Some("object.1.xml"), No),
            EnvironmentCubeLighting => ("Environment Cube Lighting", "5EL", None, No),
            Array2D => ("Array 2D", "2ARY", None, No),
            LotInformation => ("Lot Information", "LOT", None, No),
            FaceNeuralXML => ("Face Neural XML", "XFNU", Some("face_neural.xml"), No),
            NeighbourhoodObjectXML => ("Neighbourhood Object XML", "XNGB", Some("neighbourhood_object.xml"), No),
            WantsTreeItemXML => ("Wants Tree Item XML", "WNTT", Some("wants_tree_item.xml"), No),
            MainLotObjects => ("Main Lot Objects", "MOBJT", None, No),
            UnlockableRewards => ("Unlockable Rewards", "REWD", Some("rewards.txt"), No),
            AudioResource => ("Audio Resource", "AUDR", None, No),
            GeometricNode => ("Geometric Node", "GMND", Some("5gn"), No),
            // needs header detection
            Image => ("Image", "IMG", Some("img.jpg"), No),
            WallLayer => ("Wall Layer", "WLL", None, No),
            HairToneXML => ("Hair Tone XML", "XHTN", Some("hair_tone.xml"), No),
            WallThumbnail => ("Wall Thumbnail", "THUB", Some("wall_thumb.jpg"), No),
            FloorThumbnail => ("Floor Thumbnail", "THUB", Some("floor_thumb.jpg"), No),
            JPEGImage3 => ("JPEG Image", "JPEG", Some("2.jpg"), No),
            FamilyTies => ("Family Ties", "FAMT", None, No),
            FaceRegionXML => ("Face Region XML", "XFRG", Some("face_region.xml"), No),
            FaceArchXML => ("Face Arch XML", "XFCH", Some("face_arch.xml"), No),
            PredictiveMap => ("Predictive Map", "PMAP", None, No),
            SoundEffects => ("Sound Effects", "SFX", None, No),
            AcceleratorKeyDefinitions => ("Accelerator Key Definitions", "KEYD", Some("keys.txt"), No),
            PersonData => ("Person Data", "PDAT", None, No),
            FencePostLayer => ("Fence Post Layer", "FPL", None, No),
            RoofData => ("Roof Data", "ROOF", None, No),
            NeighbourhoodTerrainGeometry => ("Neighbourhood Terrain Geometry", "NHTG", None, No),
            NeighborhoodTerrain => ("Neighborhood Terrain", "NHTR", None, No),
            LinearFogLighting => ("Linear Fog Lighting", "5LF", Some("5lf"), No),
            DrawStateLighting => ("Draw State Lighting", "5DS", Some("5ds"), No),
            Thumbnail => ("Thumbnail", "THUB", Some("thumb.jpg"), No),
            GeometricDataContainer => ("Geometric Data Container", "GMDC", Some("5gd"), No),
            SimOutfits => ("Sim Outfits", "SKIN", None, No),
            NeighbourhoodID => ("Neighbourhood ID", "NID", None, No),
            RoofXML => ("Roof XML", "XROF", Some("roof.xml"), No),
            SurfaceTexture => ("Surface Texture", "STXR", None, No),
            LightOverride => ("Light Override", "NLO", Some("nlo"), No),
            TSSGSystem => ("TSSG System", "TSSG", None, No),
            AmbientLight => ("Ambient Light", "LGHT", Some("5al"), No),
            DirectionalLight => ("Directional Light", "LGHT", Some("5dl"), No),
            PointLight => ("Point Light", "LGHT", Some("5pl"), No),
            Spotlight => ("Spotlight", "LGHT", Some("5sl"), No),
            StringMap => ("String Map", "SMAP", None, No),
            VertexLayer => ("Vertex Layer", "VERT", None, No),
            FenceThumbnail => ("Fence Thumbnail", "THUB", Some("fence_thumb.jpg"), No),
            SimRelations => ("Sim Relations", "SREL", None, No),
            ModularStairThumbnail => ("Modular Stair Thumbnail", "THUB", Some("modular_stair_thumb.jpg"), No),
            RoofThumbnail => ("Roof Thumbnail", "THUB", Some("roof_thumbnail.jpg"), No),
            ChimneyThumbnail => ("Chimney Thumbnail", "THUB", Some("chimney_thumbnail.jpg"), No),
            ObjectXML2 => ("Object XML", "XOBJ", Some("object.2.xml"), No),
            FacialStructure => ("Facial Structure", "LXNR", None, No),
            MaxisMaterialShader => ("Maxis Material Shader", "MATSHAD", Some("mat.txt"), No),
            WantsAndFears => ("Wants And Fears", "SWAF", None, No),
            ContentRegistry => ("Content Registry", "CREG", None, No),
            CreationResource => ("Creation Resource", "CRES", Some("5cr"), No),
            DBPFDirectory => ("DBPF Directory", "DIR", Some("dir"), No),
            EffectsResourceTree => ("Effects Resource Tree", "FX", Some("fx"), No),
            PropertySet => ("Property Set", "GZPS", None, No),
            VersionInformation => ("Version Information", "VERS", None, No),
            TerrainThumbnail => ("Terrain Thumbnail", "THUB", Some("terrain_thumb.jpg"), No),
            HoodCamera => ("Hood Camera", "HCAM", None, No),
            LevelInformation => ("Level Information", "LIFO", Some("6li"), No),
            WantsXML => ("Wants XML", "XWNT", Some("wants.xml"), No),
            AwningThumbnail => ("Awning Thumbnail", "THUB", Some("awning_thumb.jpg"), No),
            SingularLotObject => ("Singular Lot Object", "OBJT", None, No),
            Animation => ("Animation", "ANIM", Some("5an"), No),
            Shape => ("Shape", "SHPE", Some("5sh"), No),
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
