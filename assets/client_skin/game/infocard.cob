#import
client.zsort as zsort

#scenes
"infocard_root"
    GlobalZIndex($zsort::ZINDEX_INFOCARD)
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:FlexEnd justify_cross:Center}
    Picking::Ignore
    FocusPolicy::Pass

    "infocard_frame"
        FlexNode{width:300px height:150px}
        Picking::Block
        FocusPolicy::Block
        Splat<Border>(2px)
        BrRadiusTopLeft(1px)
        BrRadiusTopRight(1px)
        BackgroundColor(#a69462)
        BorderColor(#3e3723)


        "content"
