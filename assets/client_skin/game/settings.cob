#import
client.zsort as zsort

#defs
+footer_button = \
    ControlRoot
    FlexNode{justify_main:Center justify_cross:Center}
    BrRadius(3px)
    Splat<Border>(1px)
    BorderColor(#000000)
    Multi<Responsive<BackgroundColor>>[
        {idle:#000000 hover:#444444}
        {state:[Selected] idle:#666666}
    ]

    "text"
        ControlMember
        FlexNode{margin:{top:5px bottom:5px left:7px right:7px}}
        TextLine
        TextLineSize(25)
        TextLineColor(#FFFFFF)
\

#scenes
"settings_popup"
    GlobalZIndex($zsort::ZINDEX_SETTINGS)
    FlexNode{width:100vw height:100vh justify_main:Center justify_cross:Center}
    FocusPolicy::Block
    Picking::Block
    BackgroundColor(#90444444) // Gray out background to indicate it's non-interactive.

    "window"
        FlexNode{width:80% height:80% flex_direction:Column}
        BrRadius(5px)
        Splat<Border>(3px)
        BackgroundColor(#880099)
        BorderColor(#550066)

        "main"
            FlexNode{flex_direction:Row width:100% flex_grow:1}

            "sidebar"
                RadioGroup
                FlexNode{
                    min_width:200px height:100% flex_direction:Column justify_main:FlexStart justify_cross:Center
                    border:{left:1px}
                }
                BorderColor(#BB222222)

            "content"

        "footer"
            FlexNode{
                width:100% height:200px flex_direction:Row justify_cross:Center
                border:{top:1px}
            }
            BorderColor(#BB222222)

            "quit_button"
                +footer_button{
                    Margin{right:20px}

                    "text"
                        TextLine{text:"Quit"}
                }

            ""
                FlexNode{flex_grow:1}

            "done_button"
                +footer_button{
                    Margin{left:20px}

                    "text"
                        TextLine{text:"Done"}
                }


"menu_button"
    RadioButton
    ControlRoot
    FlexNode{
        min_width:100% justify_main:Center justify_cross:Center
        margin:{top:10px}
    }
    Border{top:1px bottom:1px}
    BorderColor(#000000)
    Multi<Responsive<BackgroundColor>>[
        {idle:#000000 hover:#444444}
        {state:[Selected] idle:#666666}
    ]

    "text"
        ControlMember
        FlexNode{margin:{top:10px bottom:10px left:14px right:14px}}
        TextLine
        TextLineSize(25)
        TextLineColor(#FFFFFF)

"game_section"
    FlexNode{flex_grow:1 height:100%}

"hotkeys_section"
    FlexNode{flex_grow:1 height:100%}

"video_section"
    FlexNode{flex_grow:1 height:100%}

"audio_section"
    FlexNode{flex_grow:1 height:100%}

"dev_section"
    FlexNode{
        flex_grow:1 height:100% flex_direction:Column flex_wrap:Wrap justify_main:FlexStart justify_cross:FlexStart
        padding:{left:5px right:5px top:7px bottom:7px}
    }

"dev_button"
    ControlRoot
    FlexNode{justify_main:Center justify_cross:Center}
    Splat<Border>(1px)
    BorderColor(#000000)
    Multi<Responsive<BackgroundColor>>[
        {idle:#000000 hover:#444444}
        {state:[Selected] idle:#666666}
    ]

    "text"
        ControlMember
        FlexNode{margin:{top:5px bottom:5px left:7px right:7px}}
        TextLine
        TextLineSize(20)
        TextLineColor(#FFFFFF)
