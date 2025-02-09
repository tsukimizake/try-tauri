port module TauriCmd exposing (fromTauri, toTauri)

import Bindings exposing (FromTauriCmdType, ToTauriCmdType)
import Json.Decode


port fromTauriMsg : (Json.Decode.Value -> msg) -> Sub msg


port toTauriMsg : Json.Decode.Value -> Cmd msg


toTauri : ToTauriCmdType -> Cmd msg
toTauri cmd =
    toTauriMsg <| Bindings.toTauriCmdTypeEncoder cmd


fromTauri : (FromTauriCmdType -> msg) -> Sub msg
fromTauri msg =
    let
        _ =
            Debug.log "before" ""
    in
    fromTauriMsg <|
        \value ->
            case
                Json.Decode.decodeValue Bindings.fromTauriCmdTypeDecoder value
            of
                Ok r ->
                    let
                        _ =
                            Debug.log "r" r
                    in
                    msg r

                Err e ->
                    Debug.todo <| "Failed to decode Tauri message: " ++ Debug.toString e
