port module Main exposing (main)

import Basics.Extra exposing (..)
import Bindings
import Browser
import Bytes exposing (Endianness(..))
import CodeEditor
import Color
import Css exposing (height, pct)
import Css.Extra exposing (..)
import Html.Styled exposing (..)
import Html.Styled.Attributes exposing (css)
import Html.Styled.Events exposing (..)
import Json.Decode
import Point3d
import RecordSetter exposing (..)
import Scene
import Scene3d
import Scene3d.Material as Material
import StlDecoder exposing (Stl, Vec)
import Triangle3d


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
        , view = Html.Styled.toUnstyled << view
        }



-- MODEL


type alias Model =
    { stl : Maybe Stl
    , viewPoint : Vec
    , codeEditor : CodeEditor.Model
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { stl = Nothing
      , viewPoint = ( 50, 20, 30 )
      , codeEditor = CodeEditor.init
      }
        |> Debug.log "init"
    , readStlFile ()
    )



-- UPDATE


type Msg
    = ReadStlFile
    | TauriMsg Json.Decode.Value
    | CodeEditorMsg CodeEditor.Msg


update : Msg -> Model -> ( Model, Cmd Msg )
update msg mPrev =
    case msg of
        ReadStlFile ->
            ( mPrev
            , readStlFile ()
            )

        TauriMsg value ->
            -- TODO switch with some label
            ( { mPrev
                | stl =
                    Json.Decode.decodeValue Bindings.stlBytesDecoder value
                        |> Result.toMaybe
                        |> Maybe.andThen StlDecoder.run
              }
            , Cmd.none
            )

        CodeEditorMsg codeEditorMsg ->
            CodeEditor.update codeEditorMsg mPrev.codeEditor
                |> mapModel (putIn s_codeEditor mPrev)
                |> mapCmd CodeEditorMsg



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ tauriMsg TauriMsg
        ]



-- VIEW


view : Model -> Html Msg
view model =
    let
        point ( x, y, z ) =
            Point3d.meters x y z

        entity : ( Vec, Vec, Vec ) -> Scene3d.Entity coordinates
        entity ( a, b, c ) =
            let
                tri ( p, q, r ) =
                    Triangle3d.from (point p) (point q) (point r)
            in
            Scene3d.facet (Material.color Color.blue) (tri ( a, b, c ))
    in
    div [ css [ displayGrid, gridTemplateColumns "repeat(2, 1fr)", gridColumnGap "10px", height (pct 100) ] ]
        [ div [ css [ height (pct 100) ] ]
            [ model.stl
                |> Maybe.map (Scene.unlit model entity)
                |> Maybe.withDefault (text "")
            , div [] [ text <| "stl file len: " ++ (String.fromInt <| Maybe.withDefault 0 <| Maybe.map (\stl -> List.length stl.triangles) <| model.stl) ]
            , div [] [ text model.codeEditor.code ]
            ]
        , CodeEditor.view CodeEditorMsg model.codeEditor
        ]
