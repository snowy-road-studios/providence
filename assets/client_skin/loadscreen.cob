#import
builtin.colors.tailwind as tw
client.zsort as zsort

#defs
$COLOR_LOADSCREEN = $tw::STONE_400
$COLOR_LOADBAR_GUTTER = $tw::ZINC_400
$COLOR_LOADBAR = $tw::RED_600

#scenes
"loadscreen"
    GlobalZIndex($zsort::ZINDEX_LOADSCREEN)
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:Center justify_cross:Center}
    BackgroundColor($COLOR_LOADSCREEN)

    "text"
        FlexNode{margin:{bottom:30px}}
        TextLine{text:"Loading..." size:35}

    "gutter"
        FlexNode{width:20% height:30px flex_direction:Row justify_main:FlexStart justify_cross:Center}
        Splat<Border>(1px)
        BorderColor(#000000)
        BackgroundColor($COLOR_LOADBAR_GUTTER)

        "bar"
            FlexNode{height:100%}
            BackgroundColor($COLOR_LOADBAR)
