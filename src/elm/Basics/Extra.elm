module Basics.Extra exposing (noCmd, withCmd)


noCmd : model -> ( model, Cmd msg )
noCmd m =
    ( m, Cmd.none )


withCmd : Cmd msg -> model -> ( model, Cmd msg )
withCmd cmd m =
    ( m, cmd )
