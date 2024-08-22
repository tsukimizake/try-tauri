module Basics.Extra exposing (..)


andThen : (model -> ( model, Cmd msg )) -> ( model, Cmd msg ) -> ( model, Cmd msg )
andThen f ( model, cmd ) =
    let
        ( newModel, newCmd ) =
            f model
    in
    ( newModel, Cmd.batch [ cmd, newCmd ] )


noCmd : model -> ( model, Cmd msg )
noCmd m =
    ( m, Cmd.none )


withCmd : Cmd msg -> model -> ( model, Cmd msg )
withCmd cmd m =
    ( m, cmd )


mapCmd : (a -> b) -> ( m, Cmd a ) -> ( m, Cmd b )
mapCmd f ( m, cmd ) =
    ( m, Cmd.map f cmd )


mapModel : (a -> b) -> ( a, Cmd msg ) -> ( b, Cmd msg )
mapModel f ( m, cmd ) =
    ( f m, cmd )


putIn : (c -> b -> a) -> b -> c -> a
putIn f a b =
    f b a
