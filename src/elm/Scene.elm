module Scene exposing (preview)

import Angle
import Camera3d exposing (Camera3d)
import Color
import Direction3d
import Html.Styled exposing (..)
import Length exposing (Meters)
import Pixels exposing (int)
import Point3d
import Scene3d exposing (backgroundColor)
import StlDecoder exposing (Vec)
import Viewpoint3d


preview : { a | viewPoint : Vec } -> (c -> Scene3d.Entity coordinates) -> { d | triangles : List c } -> Html msg
preview model entity stl =
    Scene3d.sunny
        { upDirection = Direction3d.z
        , sunlightDirection = Direction3d.z
        , shadows = True
        , dimensions = ( int 400, int 400 )
        , camera = camera model.viewPoint
        , clipDepth = Length.meters 1
        , background = backgroundColor Color.black
        , entities =
            List.map entity stl.triangles
        }
        |> Html.Styled.fromUnstyled


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
