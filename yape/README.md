# Yape: Yet Another Package Editor

A native Linux DBPF package editor. Currently several editors are implemented for various resource types, and the intention
is to add editors for resources for which SimPE does not have a convenient editor.

### Editors currently implemented:

| Abbreviation | Name                               | Reading            | Editing            |
|--------------|------------------------------------|--------------------|--------------------|
| GZPS         | Property Set                       | :white_check_mark: | :white_check_mark: |
| BINX         | Binary Index                       | :white_check_mark: | :white_check_mark: |
| 3IDR/SKIN    | 3D Index Referencing / Sim Outfits | :white_check_mark: | :white_check_mark: |
| TXTR         | Texture Resource                   | :white_check_mark: | :white_check_mark: |
| TXMT         | Material Definition                | :interrobang:      | :interrobang:      |
| STR          | Text List                          | :white_check_mark: | :white_check_mark: |
| CTSS         | Catalog Description                | :white_check_mark: | :white_check_mark: |
| TTAs         | Pie Menu Strings                   | :white_check_mark: | :white_check_mark: |
| BHAV         | Behaviour Function                 | :interrobang:      | :x:                |
| OBJD         | Object Data                        | :white_check_mark: | :white_check_mark: |
| GMDC         | Geometric Data Container           | :white_check_mark: | :interrobang:      |
| SDSC         | Sim Description                    | :white_check_mark: | :white_check_mark: |

### Generic editor for:
> Track Settings, 
> Floor XML, 
> Neighbourhood Object XML, 
> Wants XML,
> Mesh Overlay XML, 
> Face Modifier XML, 
> Texture Overlay XML, 
> Fence XML, 
> Skintone XML, 
> Material Override, 
> Collection,
> Face Neutral XML, 
> Hairtone XML, 
> Face Region XML,
> Face Archetype XML,
> Sim Data XML,
> Roof XML,
> Pet Body Options,
> Wall XML, 
> Sim DNA,
> Version Information,
> Sim Outfits,

And a hexadecimal editor for all other types
