module Input exposing (textInput)

import Html.Styled exposing (Html)
import Input.Text


textInput : String -> (String -> msg) -> Html msg
textInput value onInput =
    Input.Text.input (Input.Text.defaultOptions onInput) [] value
        |> Html.Styled.fromUnstyled
