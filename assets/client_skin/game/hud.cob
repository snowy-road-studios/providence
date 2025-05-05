#import
client.zsort as zsort

#scenes
"hud"
    GlobalZIndex($zsort::ZINDEX_HUD)
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:FlexStart}
    Picking::Ignore

    "top"
        FlexNode{width:100% height:25px flex_direction:Row justify_cross:FlexStart justify_main:FlexStart}
        Picking::Ignore

        "name"
            TextLine{size:25}

        ""
            FlexNode{flex_grow:1}

        "round_info"
            FlexNode{flex_direction:Row}
            "timer"
                TextLine{size:20}
            ""
                FlexNode{width:25px}
            "round"
                TextLine{size:20}

        ""
            FlexNode{flex_grow:1}

        "fps"
            FlexNode{width:65px justify_self_cross:FlexEnd}

            "text"
                TextLine{size:15}

    "center"
        FlexNode{flex_grow:1 flex_direction:Row}
        Picking::Ignore

        "left"
            FlexNode{height:100%}
            Picking::Ignore

        "middle"
            FlexNode{flex_grow:1 height:100%}
            Picking::Ignore

        "right"
            FlexNode{height:100%}
            Picking::Ignore

    "bottom"
        FlexNode{width:100% flex_direction:Row justify_cross:FlexStart justify_main:FlexStart}
        Picking::Ignore

        "settings_button"
            FlexNode{
                justify_main:Center justify_cross:Center
                margin:{left:10px bottom:10px}
            }
            Splat<Border>(2px)
            BorderColor(#000000)
            Responsive<BackgroundColor>{
                idle:#00000000 hover:#55000000 press:#77000000
            }

            "text"
                FlexNode{margin:{top:5px bottom:5px left:7px right:7px}}
                TextLine{text:"Settings" size:20}
