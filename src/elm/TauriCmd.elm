port module TauriCmd exposing (decodeStl, fromTauri, toTauri)

import Bindings exposing (FromTauriCmdType, SerdeStlFace(..), SerdeStlFaces(..), ToTauriCmdType)
import Json.Decode
import StlDecoder exposing (Stl)


port fromTauriMsg : (Json.Decode.Value -> msg) -> Sub msg


port toTauriMsg : Json.Decode.Value -> Cmd msg


toTauri : ToTauriCmdType -> Cmd msg
toTauri cmd =
    toTauriMsg <| Bindings.toTauriCmdTypeEncoder cmd


fromTauri : (FromTauriCmdType -> msg) -> Sub msg
fromTauri msg =
    fromTauriMsg <|
        \value ->
            case
                Json.Decode.decodeValue Bindings.fromTauriCmdTypeDecoder value
            of
                Ok r ->
                    msg r

                Err e ->
                    Debug.todo <| "Failed to decode Tauri message: " ++ Debug.toString e



-- decode stl file for elm-3d-scene


decodeStl : SerdeStlFaces -> Stl
decodeStl (SerdeStlFaces triangles) =
    { header = ""
    , numTriangles = triangles |> List.length
    , triangles =
        triangles
            |> List.concatMap
                (\face ->
                    case face of
                        SerdeStlFace [ [ v00, v01, v02 ], [ v10, v11, v12 ], [ v20, v21, v22 ] ] ->
                            [ ( ( v00, v01, v02 ), ( v10, v11, v12 ), ( v20, v21, v22 ) ) ]

                        x ->
                            -- should not happen
                            let
                                _ =
                                    Debug.log "failed to decode" x
                            in
                            []
                )
    }
