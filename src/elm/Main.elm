module Main exposing (main)

import Basics.Extra exposing (..)
import Bindings exposing (FromTauriCmdType(..), ToTauriCmdType(..))
import Browser
import Bytes exposing (Endianness(..))
import Color
import Css exposing (fontFamily, height, monospace, pct)
import Css.Extra exposing (..)
import Html.Styled exposing (..)
import Html.Styled.Attributes exposing (css, type_)
import Html.Styled.Events exposing (..)
import Point3d
import RecordSetter exposing (..)
import Scene
import Scene3d
import Scene3d.Material as Material
import StlDecoder exposing (Stl, Vec)
import TauriCmd
import Triangle3d



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
    , sourceFilePath : String
    , sourceCode : String
    }


init : () -> ( Model, Cmd Msg )
init _ =
    { stl = Nothing
    , viewPoint = ( 50, 20, 30 )
    , sourceFilePath = ""
    , sourceCode = ""
    }
        |> noCmd



-- UPDATE


type Msg
    = FromTauri Bindings.FromTauriCmdType
    | ToTauri Bindings.ToTauriCmdType
    | SetSourceFilePath String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg mPrev =
    case msg of
        FromTauri cmd ->
            case cmd of
                StlBytes stlBytes ->
                    mPrev
                        |> s_stl (StlDecoder.run stlBytes)
                        |> noCmd

                Code code ->
                    mPrev
                        |> s_sourceCode code
                        |> noCmd

        ToTauri cmd ->
            mPrev
                |> withCmd (TauriCmd.toTauri cmd)

        SetSourceFilePath path ->
            mPrev
                |> s_sourceFilePath path
                |> noCmd



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    TauriCmd.fromTauri FromTauri



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
            ]
        , div []
            [ text "ファイル名"
            , input [ type_ "text", css [ fontFamily monospace ], onInput SetSourceFilePath ] []
            , button [ onClick (ToTauri (RequestCode model.sourceFilePath)) ] [ text "ファイルを開く" ]
            , div [] [ text "code" ]
            , div [ css [ fontFamily monospace ] ] [ text model.sourceCode ]
            ]

        -- , CodeEditor.view CodeEditorMsg model.codeEditor
        ]
