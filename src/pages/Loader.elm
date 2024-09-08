module Loader exposing (loader)

import Css exposing (animationDuration, animationIterationCount, animationName, border3, borderRadius, borderTop3, deg, height, hex, infinite, pct, property, px, rotate, sec, solid, width)
import Css.Animations as Animations
import Html.Styled exposing (..)
import Html.Styled.Attributes exposing (css)


loader : Float -> Html msg
loader size =
    div
        [ css
            [ border3 (px <| size / 4) solid (hex "#f3f3f3")
            , borderTop3 (px <| size / 4) solid (hex "#3498db")
            , borderRadius (pct 50)
            , width (px size)
            , height (px size)
            , Animations.keyframes
                [ ( 0
                  , [ Animations.transform
                        [ rotate (deg 0) ]
                    ]
                  )
                , ( 100
                  , [ Animations.transform
                        [ rotate (deg 360) ]
                    ]
                  )
                ]
                |> animationName
            , animationDuration <| sec 3
            , animationIterationCount infinite
            , property "animation-timing-function" "linear"
            ]
        ]
        []
