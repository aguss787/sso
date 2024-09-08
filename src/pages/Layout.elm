module Layout exposing (mainPage)

import Css exposing (column, displayFlex, flexDirection, flexGrow, height, minWidth, num, px, row)
import Html.Styled exposing (Html, div, text)
import Html.Styled.Attributes exposing (align, css)


mainPage : List (Html msg) -> Html msg
mainPage content =
    div
        [ css
            [ displayFlex
            , flexDirection row
            ]
        ]
        [ div [ css [ flexGrow (num 1) ] ] []
        , div [ css [ displayFlex, flexDirection column, minWidth (px 550) ] ] <|
            [ div [ css [ height (px 100) ] ] []
            , div [ align "center" ] [ text "No time to create a logo :)" ]
            , div [ css [ height (px 25) ] ] []
            ]
                ++ content
        , div [ css [ flexGrow (num 1) ] ] []
        ]
