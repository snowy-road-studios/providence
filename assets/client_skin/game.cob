#import
ui.skin as _

#defs
+button = \
    FlexNode{justify_main:Center justify_cross:Center}
    Splat<Border>(1px)
    BorderColor(#000000)

    "text"
        FlexNode{margin:{top:5px bottom:5px left:7px right:7px}}
        TextLine
\
+scoreboard_header_item = \
    GridNode{border:{bottom:1px}}
    BorderColor(#000000)

    "text"
        FlexNode{margin:{bottom:2px left:5px right:5px}}
\

#scenes
"game"
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:FlexStart}
    Picking::Ignore

    "header"
        FlexNode{width:100% height:25px flex_direction:Row justify_cross:FlexStart justify_main:FlexStart}

        "name_shim"
            AbsoluteNode{width:100% height:100% justify_main:Center justify_cross:Center}

            "name"
                TextLine{size:25}

        ""
            FlexNode{flex_grow:1}

        "fps"
            FlexNode{width:65px justify_self_cross:FlexEnd}

            "text"
                TextLine{size:15}

    "footer"
        FlexNode{width:100% flex_direction:Row justify_cross:FlexStart justify_main:FlexStart}

        "disconnect_button"
            +button{
                SetJustifySelfCross(Center)
                Margin{left:10px bottom:10px}
                Responsive<BackgroundColor>{
                    idle:#00000000 hover:#55888888 press:#77888888
                }

                "text"
                    TextLine{text:"Disconnect" size:20}
            }

