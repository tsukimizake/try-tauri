module Main exposing (main)

import Basics.Extra exposing (..)
import Bindings exposing (FromTauriCmdType(..), ToTauriCmdType(..))
import Browser
import Bytes exposing (Endianness(..))
import Color
import Css exposing (borderColor, borderStyle, borderWidth, fontFamily, height, monospace, padding, pct, preWrap, px, rgb, solid, whiteSpace)
import Css.Extra exposing (..)
import Html.Styled exposing (..)
import Html.Styled.Attributes exposing (css)
import Html.Styled.Events exposing (..)
import Input exposing (textInput)
import Point3d
import RecordSetter exposing (..)
import Scene
import Scene3d
import Scene3d.Material as Material
import StlDecoder exposing (Stl, Vec)
import Task
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
    { viewPoint : Vec
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
    { viewPoint = ( 100, 100, 100 )
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
                        |> s_console (Debug.toString res.value :: mPrev.console)
                        |> s_previews
                            (res.polys
                                |> List.map (\data -> { stlId = Tuple.first data, stl = TauriCmd.decodeStl <| Tuple.second data })
                            )
                        |> noCmd

                EvalError err ->
                    mPrev
                        |> s_console (err :: mPrev.console)
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
            (model.previews
                |> List.map (\{ stl } -> Scene.unlit model entity stl)
            )
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
                    -- |> List.intersperse (text "")
                    |> List.map (\line -> Html.Styled.div [ css [ padding (px 5) ] ] [ text line ])
                )

            -- , CodeEditor.view CodeEditorMsg model.codeEditor
            ]
        ]


black : Css.Color
black =
    rgb 0 0 0
