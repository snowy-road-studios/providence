#import
builtin.colors.basic as basic

#commands
CameraSettings{
    zoom_range: (0.3 3.82)
    line_zoom_factor: 1.25
    pixel_zoom_factor: 1.02
}
MapSettings{
    aseprite: "client_skin/assets/tiles.aseprite"

    sorting: {
        tile: 0.0
        select_effect: 0.01
        building: 0.02
    }

    press_color: $basic::GREY

    cursor_buffer_min: 10.0
    cursor_buffer_start: 100.0
    cursor_buffer_decayrate_secs: 0.15
}
