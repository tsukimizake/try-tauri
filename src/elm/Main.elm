port module Main exposing (main)

import Bindings exposing (Stl)
import Browser
import Html exposing (..)
import Html.Events exposing (..)
import Json.Decode


port readStlFile : () -> Cmd msg


port readStlFileResult : (String -> msg) -> Sub msg


port tauriMsg : (Json.Decode.Value -> msg) -> Sub msg



-- MAIN


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }



-- MODEL


type alias Model =
    { file : Stl
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Model (Stl [])
    , readStlFile ()
    )



-- UPDATE


type Msg
    = ReadStlFile
    | TauriMsg Json.Decode.Value


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ReadStlFile ->
            ( model
            , readStlFile ()
            )

        TauriMsg value ->
            -- TODO switch with some label
            ( { model
                | file =
                    Json.Decode.decodeValue Bindings.stlDecoder value
                        |> Result.withDefault (Stl [])
              }
            , Cmd.none
            )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ tauriMsg TauriMsg
        ]



-- VIEW


view : Model -> Html Msg
view model =
    div []
        [ h1 [] [ text "Read stl file" ]
        , button [ onClick ReadStlFile ] [ text "Read stl file" ]
        , div [] [ text <| "length: " ++ (String.fromInt <| List.length <| .bytes <| model.file) ]
        ]
