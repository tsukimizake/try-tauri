module Decoder exposing (decodeStl, triangleDecoder)

import Bytes exposing (Bytes)
import Bytes.Decode as BD exposing (Decoder, andThen, bytes, decode, float32, loop, map, string, succeed, unsignedInt32)
import Scene3d exposing (Stl, Vec)


decodeStl : Bytes -> Maybe Stl
decodeStl bytes =
    let
        stlDecoder : Decoder Stl
        stlDecoder =
            BD.map2 (\a ( b, c ) -> Stl a b c)
                (BD.string 80)
                (BD.unsignedInt32 BD.LE
                    |> BD.andThen
                        (\len ->
                            BD.loop ( len, [] )
                                (\( l, xs ) ->
                                    if l == 0 then
                                        BD.succeed <| BD.Done ( len, xs )

                                    else
                                        BD.map (\x -> BD.Loop ( l - 1, x :: xs )) triangleDecoder
                                )
                        )
                )
    in
    BD.decode stlDecoder bytes


triangleDecoder : Decoder ( Vec, Vec, Vec )
triangleDecoder =
    let
        vecDecoder =
            BD.map3 (\a b c -> ( a, b, c )) (BD.float32 BD.LE) (BD.float32 BD.LE) (BD.float32 BD.LE)
    in
    BD.map4 (\_ b c d -> ( b, c, d )) vecDecoder vecDecoder vecDecoder vecDecoder
        |> BD.andThen (\r -> BD.bytes 2 |> BD.map (\_ -> r))
