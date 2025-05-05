#import
client.zsort as zsort

#scenes
"gameover"
    GlobalZIndex($zsort::ZINDEX_GAMEOVER)
    FlexNode{width:100vw height:100vh flex_direction:Column justify_main:Center justify_cross:Center}

    "text"
        TextLine{text:"GAME OVER" size:45}
        TextLineColor(#FFFFFF)

    "end_button"
        FlexNode{justify_main:Center justify_cross:Center}
        Splat<Border>(1px)
        BorderColor(#000000)
        Responsive<BackgroundColor>{
            idle:#00000000 hover:#55000000 press:#77000000
        }

        "text"
            FlexNode{margin:{top:5px bottom:5px left:7px right:7px}}
            TextLine{text: "Exit" size:20}
            TextLineColor(#FFFFFF)
