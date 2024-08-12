port module Main exposing (main)

import Angle
import Bindings
import Browser
import Bytes exposing (Endianness(..))
import Camera3d exposing (Camera3d)
import Color
import Direction3d
import Html exposing (..)
import Html.Attributes exposing (value)
import Html.Events exposing (..)
import Json.Decode
import Length exposing (Meters)
import Pixels exposing (int)
import Point3d
import Scene3d as Scene exposing (backgroundColor)
import Scene3d.Material as Material
import StlDecoder exposing (Stl, Vec, decodeStl, encodeToBytes)
import Triangle3d
import Viewpoint3d


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
    { stl : Maybe Stl
    , viewPoint : Vec
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { stl = Nothing, viewPoint = ( 50, 20, 30 ) }
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
                | stl =
                    Json.Decode.decodeValue Bindings.stlBytesDecoder value
                        |> Result.toMaybe
                        |> Maybe.map encodeToBytes
                        |> Maybe.andThen decodeStl
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
    let
        point ( x, y, z ) =
            Point3d.meters x y z

        entity : ( Vec, Vec, Vec ) -> Scene.Entity coordinates
        entity ( a, b, c ) =
            let
                tri ( p, q, r ) =
                    Triangle3d.from (point p) (point q) (point r)
            in
            Scene.facet (Material.color Color.blue) (tri ( a, b, c ))
    in
    div []
        [ h1 [] [ text "Read stl file" ]
        , button [ onClick ReadStlFile ] [ text "Read stl file" ]
        , model.stl
            |> Maybe.map .triangles
            |> Maybe.map
                (\triangles ->
                    Scene.unlit
                        { dimensions = ( int 400, int 400 )
                        , camera = camera model.viewPoint
                        , clipDepth = Length.meters 1
                        , background = backgroundColor Color.black
                        , entities =
                            List.map entity triangles
                        }
                )
            |> Maybe.withDefault (text "")
        , div [] [ text <| "len: " ++ (String.fromInt <| Maybe.withDefault 0 <| Maybe.map (\stl -> List.length stl.triangles) <| model.stl) ]
        , div [] [ text <| "decoded: " ++ Debug.toString model.stl ]
        ]



------------
-- SCENE
------------


camera : Vec -> Camera3d Meters coordinates
camera ( x, y, z ) =
    Camera3d.perspective
        { viewpoint =
            Viewpoint3d.lookAt
                { eyePoint = Point3d.meters x y z
                , focalPoint = Point3d.origin
                , upDirection = Direction3d.positiveZ
                }
        , verticalFieldOfView = Angle.degrees 30
        }
