module Activate exposing (main)

import Browser exposing (Document)
import Css exposing (border2, borderRadius, center, color, column, displayFlex, em, flexDirection, flexGrow, fontSize, justifyContent, marginBottom, minWidth, num, padding, pct, px, rgb, row, solid, textAlign, width)
import Html.Styled exposing (Html, button, div, form, input, text, toUnstyled)
import Html.Styled.Attributes as Attributes exposing (autofocus, css, name, placeholder, type_)
import Html.Styled.Events exposing (onInput, onSubmit)
import Http
import Json.Encode
import Layout exposing (mainPage)
import Loader exposing (loader)
import Task
import Url exposing (Protocol(..), Url)
import Url.Parser as Url exposing (query)
import Url.Parser.Query as Query


type alias Model =
    { code : Maybe String
    , loading : Bool
    , error : Maybe String
    , email : String
    , message : Maybe String
    }


type Msg
    = Noop
    | Activate String
    | ActivationSuccess
    | ActivationFailure String
    | SetEmail String
    | SendActivationEmail
    | SendActivationEmailSuccess
    | SendActivationEmailFailure String


activatedMessage : List (Html Msg)
activatedMessage =
    [ div
        [ css
            [ displayFlex
            , flexDirection column
            , fontSize <| em 2
            ]
        ]
        [ div [ css [ textAlign center ] ] [ text "Your account has been activated!" ]
        , div [ css [ textAlign center ] ] [ text "You can now log in using your username and password" ]
        ]
    ]


activateForm : Model -> List (Html Msg)
activateForm model =
    [ form
        [ css
            [ displayFlex
            , flexDirection column
            , border2 (px 1) solid
            , borderRadius (px 10)
            , padding (px 20)
            ]
        , onSubmit SendActivationEmail
        ]
        [ div [ css [ marginBottom (em 1) ] ]
            [ input
                [ Attributes.disabled model.loading
                , css [ width (pct 100) ]
                , name "email"
                , type_ "text"
                , placeholder "email"
                , autofocus True
                , onInput SetEmail
                ]
                []
            ]
        , div [] <|
            case model.error of
                Just error ->
                    [ div [ css [ color (rgb 255 0 0) ] ] [ text error ] ]

                Nothing ->
                    []
        , div [] <|
            case model.message of
                Just message ->
                    [ div [ css [ color (rgb 12 112 173), textAlign center ] ] [ text message ] ]

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
    ]


view : Model -> Document Msg
view model =
    { title = "Activate your account"
    , body =
        [ toUnstyled <|
            mainPage <|
                case ( model.code, model.loading, model.error ) of
                    ( Just _, True, _ ) ->
                        [ div
                            [ css
                                [ displayFlex
                                , justifyContent center
                                ]
                            ]
                            [ loader 100
                            ]
                        ]

                    ( Just _, False, Nothing ) ->
                        activatedMessage

                    _ ->
                        activateForm model
        ]
    }


httpPost : String -> Msg -> (String -> Msg) -> Http.Body -> Cmd Msg
httpPost url onOk onFailure requestBody =
    Http.post
        { url = url
        , body = requestBody
        , expect =
            Http.expectStringResponse
                (\resp ->
                    case resp of
                        Ok _ ->
                            onOk

                        Err e ->
                            onFailure e
                )
                (\resp ->
                    case resp of
                        Http.GoodStatus_ _ _ ->
                            Ok ()

                        Http.BadStatus_ meta body ->
                            case meta.statusCode of
                                429 ->
                                    Err "too_often"

                                _ ->
                                    Err body

                        Http.Timeout_ ->
                            Err "timeout"

                        Http.NetworkError_ ->
                            Err "network error"

                        Http.BadUrl_ string ->
                            Err string
                )
        }


activate : String -> Cmd Msg
activate code =
    httpPost
        "/activate"
        ActivationSuccess
        ActivationFailure
    <|
        Http.jsonBody (Json.Encode.object [ ( "code", Json.Encode.string code ) ])


sendActivationEmail : String -> Cmd Msg
sendActivationEmail email =
    httpPost
        "/send-activation"
        SendActivationEmailSuccess
        SendActivationEmailFailure
    <|
        Http.jsonBody (Json.Encode.object [ ( "email", Json.Encode.string email ) ])


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Noop ->
            ( model, Cmd.none )

        Activate code ->
            ( { model | loading = True }
            , activate code
            )

        ActivationSuccess ->
            ( { model | loading = False, error = Nothing }
            , Cmd.none
            )

        ActivationFailure error ->
            ( { model | loading = False, error = Just error }
            , Cmd.none
            )

        SetEmail email ->
            ( { model | email = email }, Cmd.none )

        SendActivationEmail ->
            ( { model | loading = True, message = Nothing }
            , sendActivationEmail model.email
            )

        SendActivationEmailSuccess ->
            ( { model | loading = False, error = Nothing, message = Just "Email sent, please check your email" }, Cmd.none )

        SendActivationEmailFailure error ->
            ( { model | loading = False, error = Just error }, Cmd.none )


initFromUrl : Url -> ( Model, Cmd Msg )
initFromUrl url =
    let
        code =
            Url.parse (query <| Query.string "code") { url | path = "" } |> Maybe.andThen identity
    in
    ( { code = code
      , loading = False
      , error = Nothing
      , message = Nothing
      , email = ""
      }
    , case code of
        Just activation_code ->
            Task.perform Activate (Task.succeed activation_code)

        Nothing ->
            Cmd.none
    )


main : Program () Model Msg
main =
    Browser.application
        { init = \_ -> \url -> \_ -> initFromUrl url
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        , onUrlRequest = \_ -> Noop
        , onUrlChange = \_ -> Noop
        }
