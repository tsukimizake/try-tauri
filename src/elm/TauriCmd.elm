port module TauriCmd exposing (decodeStl, fromTauri, toTauri)

import Bindings exposing (FromTauriCmdType, SerdeIndexedTriangle(..), SerdeTriangle(..), SerdeVector(..), StlObjSerde, ToTauriCmdType)
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


decodeStl : StlObjSerde -> Stl
decodeStl { mesh } =
    let
        triangles =
            mesh.faces
    in
    { header = ""
    , numTriangles = triangles |> List.length
    , triangles =
        triangles
            |> List.concatMap
                (\face ->
                    case face of
                        SerdeIndexedTriangle _ [ v1, v2, v3 ] ->
                            [ ( v1, v2, v3 ) ]

                        _ ->
                            -- should not happen
                            []
                )
    }
