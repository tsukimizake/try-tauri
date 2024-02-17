port module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Events exposing (..)


port readStlFile : () -> Cmd msg


port readStlFileResult : (String -> msg) -> Sub msg



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
    { file : String
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Model ""
    , Cmd.none
    )



-- UPDATE


type Msg
    = ReadStlFile
    | ReadStlFileResult String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ReadStlFile ->
            ( model
            , readStlFile ()
            )

        ReadStlFileResult file ->
            let
                _ =
                    Debug.log "ReadStlFile" file
            in
            ( { model | file = file }, Cmd.none )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ readStlFileResult ReadStlFileResult
        ]



-- VIEW


view : Model -> Html Msg
view model =
    div []
        [ h1 [] [ text "Read stl file" ]
        , button [ onClick ReadStlFile ] [ text "Read stl file" ]
        , div [] [ text model.file ]
        ]
