#manifest
self as user
"user_client/sidebar.cob" as user.sidebar
"user_client/sections/main.cob" as user.sections
"user_client/widgets/main.cob" as user.widgets
"user_client/zsort.cob" as user.zsort

#import
user.zsort as zsort
user.widgets as widgets

#defs
//button bg color
//button border radius
//button border thickness
//button border color

#scenes
"main"
    FlexNode{height:100vh width:100vw flex_direction:Row}

    "sidebar"
        FlexNode{height:100% flex_direction:Column justify_main:Center justify_cross:Center}

    "content"
        FlexNode{height:100% flex_grow:1 flex_direction:Column justify_main:FlexStart justify_cross:FlexStart}

"reconnecting_overlay"
    FlexNode{width:100vw height:100vh justify_main:Center justify_cross:Center}
    GlobalZIndex($zsort::ZINDEX_RECONNECTING_OVERLAY)
    FocusPolicy::Block
    Picking::Block
    BackgroundColor(#000000)

    "text"
        TextLine{text:"Reconnecting to game..." size:22}
        TextLineColor(#FFFFFF)

"ack_popup"
    +widgets::popup{
        GlobalZIndex($zsort::ZINDEX_ACK_LOBBY_POPUP)
        "window"
            "title"
                "text"
                    TextLine{text:"Start Game"}

            "content"
                SetJustifyMain(Center)
                SetJustifyCross(Center)
                // Add timer to content
                "timer"
                    "text"
                        TextLine{size:80}
                        TextLineColor(#FFFFFF)

            "footer"
                "cancel_button"
                    "text"
                        TextLine{text:"Reject"}
                "accept_button"
                    "text"
                        TextLine{text:"Accept"}
    }
