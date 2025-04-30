#import
client.zsort as zsort

#scenes
"overlay"
    GlobalZIndex($zsort::ZINDEX_TILESELECT_OVERLAY)
    FlexNode{width:100vw height:100px flex_direction:Column justify_main:FlexEnd justify_cross:Center}
    Picking::Ignore
    FocusPolicy::Pass

    "text"
        FlexNode{margin:{top:30px}}
        Picking::Ignore
        FocusPolicy::Pass
        TextLine
        TextLineSize(40)
        TextLineColor(#FFFFFF)
