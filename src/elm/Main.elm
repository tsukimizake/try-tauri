module Main exposing (main)

import Angle
import Basics.Extra exposing (..)
import Bindings exposing (FromTauriCmdType(..), ToTauriCmdType(..))
import Browser
import Color
import Css exposing (absolute, backgroundColor, border, borderColor, borderRadius, borderStyle, borderWidth, bottom, color, cursor, fontFamily, height, hover, monospace, padding, padding2, pct, pointer, position, preWrap, px, relative, rgb, right, solid, whiteSpace, zero)
import Css.Extra exposing (..)
import Html.Styled exposing (..)
import Html.Styled.Attributes exposing (css)
import Html.Styled.Events exposing (..)
import Input exposing (textInput)
import Length exposing (Meters)
import Point3d exposing (Point3d)
import RecordSetter exposing (..)
import Scene
import Scene3d
import Scene3d.Material as Material
import StlDecoder exposing (Stl, Vec)
import Task
import TauriCmd
import Triangle3d exposing (Triangle3d)



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
    { sceneModel : Scene.Model
    , sourceFilePath : String
    , sourceCode : String
    , console : List String
    , previews : List PreviewConfig
    }


type alias PreviewConfig =
    { stlId : Int
    , stl : Stl
    }


init : () -> ( Model, Cmd Msg )
init _ =
    let
        viewPoint =
            ( 100, 100, 100 )

        ( x, y, z ) =
            viewPoint

        distance =
            sqrt (x * x + y * y + z * z)

        azimuth =
            Angle.radians (atan2 y x)

        elevation =
            Angle.radians (asin (z / distance))
    in
    { sceneModel =
        { rotatexy = azimuth
        , elevation = elevation
        , distance = distance
        , isDragging = False
        , viewPoint = viewPoint
        }
    , sourceFilePath = "../hoge.lisp"
    , sourceCode = ""
    , console = []
    , previews = []
    }
        |> withCmd (emit <| ToTauri (RequestCode "../hoge.lisp"))


emit : Msg -> Cmd Msg
emit msg =
    Task.perform identity (Task.succeed msg)



-- UPDATE


type Msg
    = FromTauri Bindings.FromTauriCmdType
    | ToTauri Bindings.ToTauriCmdType
    | SetSourceFilePath String
    | SceneMsg Scene.Msg
    | ShowSaveDialog Int


update : Msg -> Model -> ( Model, Cmd Msg )
update msg mPrev =
    case msg of
        FromTauri cmd ->
            case cmd of
                Code code ->
                    mPrev
                        |> s_sourceCode code
                        |> noCmd

                EvalOk res ->
                    mPrev
                        |> s_previews
                            (res.previews
                                |> List.concatMap
                                    (\id ->
                                        case
                                            res.polys
                                                |> List.filter (\( stlId, _ ) -> stlId == id)
                                                |> List.head
                                                |> Maybe.map Tuple.second
                                                |> Maybe.map TauriCmd.decodeStl
                                        of
                                            Just stl ->
                                                [ { stlId = id, stl = stl } ]

                                            Nothing ->
                                                []
                                    )
                            )
                        |> noCmd

                EvalError err ->
                    mPrev
                        |> s_console (err :: mPrev.console)
                        |> noCmd

                SaveStlFileOk message ->
                    mPrev
                        |> s_console (message :: mPrev.console)
                        |> noCmd

                SaveStlFileError error ->
                    mPrev
                        |> s_console (error :: mPrev.console)
                        |> noCmd

        ToTauri cmd ->
            mPrev
                |> withCmd (TauriCmd.toTauri cmd)

        SetSourceFilePath path ->
            mPrev
                |> s_sourceFilePath path
                |> noCmd

        SceneMsg sceneMsg ->
            let
                updatedSceneModel =
                    Scene.update sceneMsg mPrev.sceneModel
            in
            mPrev
                |> s_sceneModel updatedSceneModel
                |> noCmd

        ShowSaveDialog stlId ->
            -- For simplicity, just use a hardcoded filename
            -- In a real implementation, you would use a file dialog here
            mPrev
                |> withCmd (TauriCmd.toTauri (Bindings.SaveStlFile stlId "output.stl"))



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ TauriCmd.fromTauri FromTauri
        , Sub.map SceneMsg (Scene.subscriptions model.sceneModel)
        ]



-- VIEW


view : Model -> Html Msg
view model =
    let
        point : Vec -> Point3d Meters Vec
        point ( x, y, z ) =
            Point3d.meters x y z

        entity : ( Vec, Vec, Vec ) -> Scene3d.Entity Vec
        entity ( a, b, c ) =
            let
                tri : ( Vec, Vec, Vec ) -> Triangle3d Meters Vec
                tri ( p, q, r ) =
                    Triangle3d.from (point p) (point q) (point r)
            in
            Scene3d.facet (Material.matte Color.lightBlue) (tri ( a, b, c ))

        -- Create a preview with a save button for each STL model
        viewPreview : PreviewConfig -> Html Msg
        viewPreview { stlId, stl } =
            div [ css [ position relative ] ]
                [ Html.Styled.map SceneMsg (Scene.preview model.sceneModel entity stl)
                , div
                    [ css
                        [ position absolute
                        , bottom (px 10)
                        , right (px 10)
                        ]
                    ]
                    [ button
                        [ onClick (ShowSaveDialog stlId)
                        , css
                            [ backgroundColor (rgb 70 130 180)
                            , color (rgb 255 255 255)
                            , padding2 (px 8) (px 12)
                            , borderRadius (px 4)
                            , border zero
                            , cursor pointer
                            , hover [ backgroundColor (rgb 50 110 160) ]
                            ]
                        ]
                        [ text "Save as STL" ]
                    ]
                ]
    in
    div [ css [ displayGrid, gridTemplateColumns "repeat(2, 1fr)", gridColumnGap "10px", height (pct 100) ] ]
        [ div [ css [ height (pct 100) ] ]
            (model.previews |> List.map viewPreview)
        , div []
            [ text "file path"
            , textInput model.sourceFilePath SetSourceFilePath
            , button [ onClick (ToTauri (RequestCode model.sourceFilePath)) ] [ text "read file" ]
            , button [ onClick (ToTauri RequestEval) ] [ text "eval" ]
            , p
                [ css
                    [ fontFamily monospace
                    , whiteSpace preWrap
                    , borderStyle solid
                    , borderWidth (px 1)
                    ]
                ]
                [ text model.sourceCode ]

            -- console
            , div
                [ css
                    [ fontFamily monospace
                    , borderStyle solid
                    , borderColor black
                    , borderWidth (px 2)
                    ]
                ]
                (model.console
                    |> List.map (\line -> Html.Styled.div [ css [ padding (px 5) ] ] [ text line ])
                )
            ]
        ]


black : Css.Color
black =
    rgb 0 0 0
