module Scene exposing (Model, Msg(..), preview, subscriptions, update)

import Angle exposing (Angle)
import Browser.Events
import Camera3d exposing (Camera3d)
import Color
import Direction3d
import Html.Styled exposing (..)
import Html.Styled.Events exposing (on)
import Html.Styled.Lazy exposing (lazy3)
import Json.Decode as Decode exposing (Decoder)
import Length exposing (Meters)
import Pixels exposing (Pixels, int)
import Point3d
import Quantity exposing (Quantity)
import Scene3d exposing (backgroundColor)
import SketchPlane3d
import StlDecoder exposing (Vec)
import Viewpoint3d


type alias Model =
    { rotatexy : Angle
    , elevation : Angle
    , distance : Float
    , viewPoint : Vec
    }


type Msg
    = MouseDown
    | MouseUp
    | MouseMove (Quantity Float Pixels) (Quantity Float Pixels)
    | MouseWheel Float


update : Msg -> Model -> ( Model, Bool )
update message model =
    case message of
        -- Start dragging when a mouse button is pressed
        MouseDown ->
            ( model, True )

        -- Stop dragging when a mouse button is released
        MouseUp ->
            ( model, False )

        -- Orbit camera on mouse move
        MouseMove dx dy ->
            let
                -- How fast we want to orbit the camera (orbiting the
                -- camera by 1 degree per pixel of drag is a decent default
                -- to start with)
                rotationRate =
                    Angle.degrees 1 |> Quantity.per Pixels.pixel

                -- Adjust azimuth based on horizontal mouse motion
                newRotatexy =
                    model.rotatexy
                        |> Quantity.minus (dx |> Quantity.at rotationRate)

                -- Adjust elevation based on vertical mouse motion
                -- and clamp to avoid camera flipping over
                newElevation =
                    model.elevation
                        |> Quantity.plus (dy |> Quantity.at rotationRate)
                        |> Quantity.clamp (Angle.degrees -90) (Angle.degrees 90)
            in
            -- Return updated model and keep isDragging as True
            ( { model | rotatexy = newRotatexy, elevation = newElevation }, True )

        -- Zoom with mouse wheel
        MouseWheel deltaY ->
            let
                -- Adjust zoom based on wheel movement
                -- Negative deltaY means wheel scrolled up (zoom in)
                -- Positive deltaY means wheel scrolled down (zoom out)
                zoomFactor =
                    0.005

                newDistance =
                    model.distance
                        * (1 + (deltaY * zoomFactor))
                        -- Don't let camera get too close
                        |> max 1.0
                        -- Don't let camera get too far
                        |> min 1000.0
            in
            ( { model | distance = newDistance }, False )



-- Decoder for mouse movement


decodeMouseMove : Decoder Msg
decodeMouseMove =
    Decode.map2 MouseMove
        (Decode.field "movementX" (Decode.map Pixels.float Decode.float))
        (Decode.field "movementY" (Decode.map Pixels.float Decode.float))



-- Decoder for mouse wheel
-- Custom event listener for wheel events


onWheel : (Float -> msg) -> Attribute msg
onWheel msg =
    on "wheel"
        (Decode.map msg (Decode.field "deltaY" Decode.float))


subscriptions : Bool -> Sub Msg
subscriptions isDragging =
    if isDragging then
        -- If we're currently dragging, listen for mouse moves and mouse button up events
        Sub.batch
            [ Browser.Events.onMouseMove decodeMouseMove
            , Browser.Events.onMouseUp (Decode.succeed MouseUp)
            ]

    else
        -- If we're not currently dragging, just listen for mouse down events
        Browser.Events.onMouseDown (Decode.succeed MouseDown)


preview : Model -> (c -> Scene3d.Entity coordinates) -> { d | triangles : List c } -> Html Msg
preview model entity stl =
    div
        [ onWheel MouseWheel
        , on "mousedown" (Decode.succeed MouseDown) -- Add mousedown handler directly to this div
        ]
        [ lazy3 renderScene model entity stl ]



-- Separate rendering function that can be lazily evaluated


renderScene : Model -> (c -> Scene3d.Entity coordinates) -> { d | triangles : List c } -> Html msg
renderScene model entity stl =
    Scene3d.sunny
        { upDirection = Direction3d.z
        , sunlightDirection = Direction3d.xy model.rotatexy
        , shadows = True
        , dimensions = ( int 400, int 400 )
        , camera = orbitingCamera model
        , clipDepth = Length.meters 1
        , background = backgroundColor Color.black
        , entities =
            List.map entity stl.triangles
        }
        |> Html.Styled.fromUnstyled


orbitingCamera : Model -> Camera3d Meters coordinates
orbitingCamera model =
    Camera3d.perspective
        { viewpoint =
            Viewpoint3d.orbit
                { focalPoint = Point3d.origin
                , groundPlane = SketchPlane3d.xy
                , azimuth = model.rotatexy
                , elevation = model.elevation
                , distance = Length.meters model.distance
                }
        , verticalFieldOfView = Angle.degrees 30
        }
