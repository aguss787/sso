module Login exposing (main)

import Browser exposing (Document, UrlRequest(..))
import Browser.Navigation exposing (load)
import Css exposing (..)
import Html.Styled exposing (Html, a, button, div, form, input, text, toUnstyled)
import Html.Styled.Attributes as Attributes exposing (align, autofocus, css, href, method, name, placeholder, type_, value)
import Html.Styled.Events exposing (on)
import Json.Decode as Json
import Layout exposing (mainPage)
import Loader exposing (loader)
import Url
import Url.Parser exposing (parse, query)
import Url.Parser.Query as Query


loginForm : Model -> Html Msg
loginForm model =
    form
        [ method "post"
        , css
            [ displayFlex
            , flexDirection column
            , border2 (px 1) solid
            , borderRadius (px 10)
            , padding (px 20)
            ]
        , on "submit" (Json.succeed Login)
        ]
        [ div [ css [ marginBottom (em 1) ] ]
            [ input
                [ Attributes.disabled model.loading
                , css [ width (pct 100) ]
                , name "username"
                , type_ "text"
                , placeholder "Username"
                , autofocus True
                ]
                []
            ]
        , div [ css [ marginBottom (em 1) ] ]
            [ input
                [ Attributes.disabled model.loading
                , css [ width (pct 100) ]
                , name "password"
                , type_ "password"
                , placeholder "Password"
                ]
                []
            ]
        , div []
            [ input [ name "client_id", type_ "hidden", value model.client_id ] []
            , input [ name "redirect_uri", type_ "hidden", value model.redirect_uri ] []
            ]
        , div [] <|
            case model.error of
                Just "not_activated" ->
                    [ div [ css [ displayFlex, flexDirection column ] ]
                        [ div [] [ text "Account not activated, please check you email" ]
                        , a [ href "/activate" ] [ text "Resend activation email" ]
                        ]
                    ]

                Just error ->
                    [ div [] [ text error ] ]

                Nothing ->
                    []
        , div [ css [ displayFlex, flexDirection row ] ]
            [ div [ css [ flexGrow (num 1) ] ] []
            , div []
                [ button
                    [ Attributes.disabled model.loading
                    , css
                        [ minWidth (px 100)
                        , textAlign center
                        , displayFlex
                        , justifyContent center
                        ]
                    ]
                  <|
                    case model.loading of
                        False ->
                            [ text "Login" ]

                        True ->
                            [ div [] [ loader 16 ] ]
                ]
            , div [ css [ flexGrow (num 1) ] ] []
            ]
        ]


registerText : Html msg
registerText =
    div [ css [ displayFlex, flexDirection column ] ]
        [ div []
            [ text "Don't have an account? Register "
            , a [ href "/register" ] [ text "here" ]
            , text "."
            ]
        ]


view : Model -> Document Msg
view model =
    { title = "Login"
    , body =
        [ toUnstyled <|
            mainPage
                [ loginForm model
                , div [ css [ height (px 10) ] ] []
                , registerText
                ]
        ]
    }


type alias Model =
    { client_id : String
    , redirect_uri : String
    , error : Maybe String
    , loading : Bool
    }


type Msg
    = Noop
    | Login
    | UrlRequest String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Noop ->
            ( model, Cmd.none )

        Login ->
            ( { model | loading = True }, Cmd.none )

        UrlRequest url ->
            ( model, load url )


modelFromUrl : Url.Url -> Model
modelFromUrl url =
    { client_id = parse (query <| Query.string "client_id") url |> Maybe.andThen identity |> Maybe.withDefault ""
    , redirect_uri = parse (query <| Query.string "redirect_uri") url |> Maybe.andThen identity |> Maybe.withDefault ""
    , error = parse (query <| Query.string "error") url |> Maybe.andThen identity
    , loading = False
    }


main : Program () Model Msg
main =
    Browser.application
        { init = \_ -> \url -> \_ -> ( modelFromUrl { url | path = "" }, Cmd.none )
        , view = view
        , update = update
        , subscriptions = \_ -> Sub.none
        , onUrlRequest =
            \request ->
                case request of
                    Internal url ->
                        UrlRequest <| Url.toString url

                    External url ->
                        UrlRequest url
        , onUrlChange = \_ -> Noop
        }
