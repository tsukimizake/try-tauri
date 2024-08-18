module Css.Extra exposing (..)

import Css exposing (Style, property)


displayGrid : Style
displayGrid =
    property "display" "grid"


gridTemplateColumns : String -> Style
gridTemplateColumns value =
    property "grid-template-columns" value


gridTemplateRows : String -> Style
gridTemplateRows value =
    property "grid-template-rows" value


gridColumnGap : String -> Style
gridColumnGap value =
    property "grid-column-gap" value


gridAutoRows : String -> Style
gridAutoRows value =
    property "grid-auto-rows" value
