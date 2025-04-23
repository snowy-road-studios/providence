#import
game.zsort as zsort

#scenes
"overlay"
    GlobalZIndex($zsort::ZINDEX_TILESELECT_OVERLAY)
    FlexNode{width:100% height:100% flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}
    Picking::Ignore
    FocusPolicy::Pass

    "text"
        FlexNode{margin:{top:30px}}
        Picking::Ignore
        FocusPolicy::Pass
        TextLine
        TextLineSize(40)
        TextLineColor(#FFFFFF)
