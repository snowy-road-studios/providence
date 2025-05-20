#commands
CameraSettings{
    zoom_range: (0.3 3.82)
    line_zoom_factor: 1.25
    pixel_zoom_factor: 1.02
}
MapSettings{
    tile_aseprite: "client_skin/assets/tiles.aseprite"

    tiles: {
        "mountain": { aseprite_slice:"mountain" display_name:"Mountain Tile" }
        "water": { aseprite_slice:"water" display_name:"Water Tile" }
        "grass": { aseprite_slice:"grass" display_name:"Grass Tile" }
        "forest": { aseprite_slice:"forest" display_name:"Forest Tile" }
        "stone": { aseprite_slice:"stone" display_name:"Stone Tile" }
        "ore": { aseprite_slice:"ore" display_name:"Ore Tile" }
        // "hyperium": {aseprite_slice:"hyperium" display_name:"Hyperium Tile"}
    }
    select_effect_slice: "effect_selected"

    press_color: #E0E0E0

    cursor_buffer_min: 10.0
    cursor_buffer_start: 100.0
    cursor_buffer_decayrate_secs: 0.15
}
