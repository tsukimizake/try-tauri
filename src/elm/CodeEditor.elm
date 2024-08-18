module CodeEditor exposing (Model, Msg, init, update, view)

import Basics.Extra exposing (..)
import Bytes exposing (Endianness(..))
import Css exposing (fontFamily, height, monospace, pct)
import Css.Extra exposing (..)
import Html.Styled exposing (..)
import Html.Styled.Attributes exposing (contenteditable, css, spellcheck)
import Html.Styled.Events exposing (..)


type alias Model =
    { code : String
    }


getLines : String -> Int
getLines code =
    code
        |> String.split "\n"
        |> List.length


init : Model
init =
    { code = "(def (fib x) (if (< x 2) x (+ (fib (- x 1)) (fib (- x 2)))))\n(fib 10)"
    }


type Msg
    = UpdateCode String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg mPrev =
    case msg of
        UpdateCode code ->
            { mPrev | code = code } |> noCmd


view : (Msg -> msg) -> Model -> Html msg
view tagger m =
    Html.Styled.map tagger <|
        div
            [ css
                [ displayGrid
                , gridTemplateColumns "2em 1fr"
                , fontFamily monospace
                ]
            ]
            [ linums m
            , div []
                [ div
                    [ contenteditable True
                    , spellcheck False
                    , onInput UpdateCode
                    , css [ height (pct 100) ]
                    ]
                    [ text m.code ]
                ]
            ]


linums : Model -> Html msg
linums _ =
    div
        [ css
            [ displayGrid
            , gridTemplateColumns "2em"
            , gridAutoRows "1em"
            ]
        ]
        (List.range 1 100
            |> List.map (\n -> div [] [ text (String.fromInt n) ])
        )
