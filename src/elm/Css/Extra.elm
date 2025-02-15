module Css.Extra exposing (displayGrid, gridColumnGap, gridTemplateColumns)

import Css exposing (Style, property)


displayGrid : Style
displayGrid =
    property "display" "grid"


gridTemplateColumns : String -> Style
gridTemplateColumns value =
    property "grid-template-columns" value


gridColumnGap : String -> Style
gridColumnGap value =
    property "grid-column-gap" value
