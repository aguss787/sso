module Register exposing (main)

import Browser exposing (Document)
import Css exposing (border2, borderRadius, center, color, column, displayFlex, em, flexDirection, flexGrow, fontSize, height, justifyContent, marginBottom, minWidth, num, padding, pct, px, rgb, row, solid, textAlign, width)
import Html.Styled exposing (Html, button, div, form, input, text, toUnstyled)
import Html.Styled.Attributes as Attributes exposing (align, autofocus, css, method, name, placeholder, type_)
import Html.Styled.Events exposing (onInput, onSubmit)
import Http
import Json.Encode
import Layout exposing (mainPage)
import Loader exposing (loader)


type alias Model =
    { path : String
    , loading : Bool
    , username : String
    , email : String
    , password : String
    , error : Maybe String
    , done : Bool
    }


registerForm : Model -> Html Msg
registerForm model =
    form
        [ method "post"
        , css
            [ displayFlex
            , flexDirection column
            , border2 (px 1) solid
            , borderRadius (px 10)
            , padding (px 20)
            ]
        , onSubmit Register
        ]
        [ div [ css [ marginBottom (em 1) ] ]
            [ input
                [ Attributes.disabled model.loading
                , css [ width (pct 100) ]
                , name "username"
                , type_ "text"
                , placeholder "Username"
                , autofocus True
                , onInput UpdateUsername
                ]
                []
            ]
        , div [ css [ marginBottom (em 1) ] ]
            [ input
                [ Attributes.disabled model.loading
                , css [ width (pct 100) ]
                , name "email"
                , type_ "text"
                , placeholder "Email"
                , onInput UpdateEmail
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
                , onInput UpdatePassword
                ]
                []
            ]
        , div [] <|
            case model.error of
                Just error ->
                    [ div [ css [ textAlign center, color (rgb 255 0 0) ] ] [ text error ] ]

                Nothing ->
                    []
        , div [ css [ displayFlex, flexDirection row ] ]
            [ div [ css [ flexGrow (num 1) ] ] []
            , div []
                [ button
                    [ Attributes.disabled model.loading
                    , css
                        [ minWidth (px 100)
                        , displayFlex
                        , justifyContent center
                        ]
                    ]
                  <|
                    case model.loading of
                        False ->
                            [ text "Register" ]

                        True ->
                            [ div [] [ loader 16 ] ]
                ]
            , div [ css [ flexGrow (num 1) ] ] []
            ]
        ]


view : Model -> Document Msg
view model =
    { title = "Register"
    , body =
        [ toUnstyled <|
            mainPage
                [ if model.done then
                    div []
                        [ div [ align "center", css [ fontSize (px 36) ] ] [ text "Registration successful!" ]
                        , div [ align "center", css [ fontSize (px 20) ] ] [ text "Please check your email for activation" ]
                        ]

                  else
                    registerForm model
                ]
        ]
    }


type Msg
    = Noop
    | Register
    | UpdateUsername String
    | UpdateEmail String
    | UpdatePassword String
    | RegisterSuccess
    | RegisterFailure String


register : Model -> Cmd Msg
register model =
    Http.post
        { url = model.path
        , body =
            Http.jsonBody <|
                Json.Encode.object
                    [ ( "username", Json.Encode.string model.username )
                    , ( "email", Json.Encode.string model.email )
                    , ( "password", Json.Encode.string model.password )
                    ]
        , expect =
            Http.expectStringResponse
                (\response ->
                    case response of
                        Err (Http.BadStatus_ _ body) ->
                            RegisterFailure body

                        Err _ ->
                            RegisterFailure "network_error"

                        Ok _ ->
                            RegisterSuccess
                )
                (\response ->
                    case response of
                        Http.GoodStatus_ metadata body ->
                            Ok body

                        _ ->
                            Err response
                )
        }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Noop ->
            ( model, Cmd.none )

        Register ->
            ( { model | loading = True }, register model )

        RegisterSuccess ->
            ( { model | loading = False, done = True }, Cmd.none )

        RegisterFailure error ->
            ( { model | loading = False, error = Just error }, Cmd.none )

        UpdateUsername username ->
            ( { model | username = username }, Cmd.none )

        UpdateEmail email ->
            ( { model | email = email }, Cmd.none )

        UpdatePassword password ->
            ( { model | password = password }, Cmd.none )


main : Program () Model Msg
main =
    Browser.application
        { init =
            \_ ->
                \url ->
                    \_ ->
                        ( { path = url.path
                          , loading = False
                          , username = ""
                          , email = ""
                          , password = ""
                          , error = Nothing
                          , done = False
                          }
                        , Cmd.none
                        )
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        , onUrlChange = \_ -> Noop
        , onUrlRequest = \_ -> Noop
        }
