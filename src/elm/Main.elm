module Main exposing (main)

import Angle
import Basics.Extra exposing (noCmd, withCmd)
import Bindings exposing (FromTauriCmdType(..), ToTauriCmdType(..))
import Browser
import Color
import Css exposing (absolute, backgroundColor, border, borderColor, borderRadius, borderStyle, borderWidth, bottom, color, cursor, fontFamily, height, hover, left, monospace, padding, padding2, pct, pointer, position, preWrap, px, relative, rgb, right, solid, top, whiteSpace, zero)
import Css.Extra exposing (..)
import Html.Styled exposing (..)
import Html.Styled.Attributes exposing (css)
import Html.Styled.Events exposing (..)
import Html.Styled.Lazy exposing (lazy)
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
    { sourceFilePath : String
    , sourceCode : String
    , console : List String
    , previews : List PreviewConfig
    }


type alias PreviewConfig =
    { stlId : Int
    , stl : Stl
    , isDragging : Bool
    , sceneModel : Scene.Model
    }


init : () -> ( Model, Cmd Msg )
init _ =
    { sourceFilePath = "../hoge.lisp"
    , sourceCode = ""
    , console = []
    , previews = []
    }
        |> withCmd (emit <| ToTauri (RequestCode "../hoge.lisp"))


createPreviewConfig : Int -> Stl -> PreviewConfig
createPreviewConfig id stl =
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
    { stlId = id
    , stl = stl
    , isDragging = False
    , sceneModel =
        { rotatexy = azimuth
        , elevation = elevation
        , distance = distance
        , viewPoint = viewPoint
        }
    }


emit : Msg -> Cmd Msg
emit msg =
    Task.perform identity (Task.succeed msg)



-- UPDATE


type Msg
    = FromTauri Bindings.FromTauriCmdType
    | ToTauri Bindings.ToTauriCmdType
    | SetSourceFilePath String
    | SceneMsg Int Scene.Msg
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
                                                [ createPreviewConfig id stl ]

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

        SceneMsg previewId sceneMsg ->
            let
                -- Update the isDragging field and sceneModel of the specified preview
                updatedPreviews =
                    List.map
                        (\preview ->
                            if preview.stlId == previewId then
                                let
                                    ( updatedSceneModel, isDragging ) =
                                        Scene.update sceneMsg preview.sceneModel

                                    -- Remove debug log since everything is working now
                                in
                                { preview
                                    | isDragging = isDragging
                                    , sceneModel = updatedSceneModel
                                }

                            else
                                preview
                        )
                        mPrev.previews
            in
            mPrev
                |> s_previews updatedPreviews
                |> noCmd

        ShowSaveDialog stlId ->
            -- For simplicity, just use a hardcoded filename
            -- In a real implementation, you would use a file dialog here
            mPrev
                |> withCmd (TauriCmd.toTauri (Bindings.SaveStlFile stlId "output.stl"))



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    let
        -- For any preview that is currently being dragged, we need mouse move and mouse up events
        draggingSubs =
            model.previews
                |> List.filter .isDragging
                |> List.map
                    (\preview ->
                        Sub.map (SceneMsg preview.stlId) (Scene.subscriptions True)
                    )

        -- For all other previews, we need mouse down events to start dragging
        nonDraggingSubs =
            if List.any .isDragging model.previews then
                -- If any preview is being dragged, don't listen for mouseDown on others
                []

            else
                model.previews
                    |> List.map
                        (\preview ->
                            Sub.map (SceneMsg preview.stlId) (Scene.subscriptions False)
                        )
    in
    Sub.batch
        (TauriCmd.fromTauri FromTauri :: (draggingSubs ++ nonDraggingSubs))



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
        viewPreview preview =
            let
                { stlId, stl, sceneModel } =
                    preview

                previewLabel =
                    "Model Id" ++ String.fromInt stlId
            in
            div [ css [ position relative ] ]
                [ div [ css [ position absolute, top (px 10), left (px 10), color (rgb 255 255 255) ] ]
                    [ text previewLabel ]
                , Html.Styled.map (SceneMsg stlId) (Scene.preview sceneModel entity stl)
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
            (model.previews |> List.map (\preview -> lazy viewPreview preview))
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
