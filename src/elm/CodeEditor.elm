module CodeEditor exposing (Model, Msg, init, update, view)

-- カーソルが移動する、elementが空になるとエラーが出るなどの問題があるため一旦後回し
-- https://github.com/jxxcarlson/elm-text-editor などを使う？

import Basics.Extra exposing (..)
import Bytes exposing (Endianness(..))
import Css exposing (fontFamily, height, monospace, pct)
import Css.Extra exposing (..)
import Html.Styled exposing (..)
import Html.Styled.Attributes exposing (contenteditable, css, id, spellcheck)
import Html.Styled.Events exposing (..)
import Json.Decode as Json
import RecordSetter exposing (..)


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
            let
                _ =
                    Debug.log "UpdateCode" code
            in
            if code == "" then
                mPrev |> noCmd

            else
                mPrev |> s_code code |> noCmd


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
                    , onContentEditableInput UpdateCode
                    , css
                        [ height (pct 100)
                        ]
                    , id "codeEditor"
                    ]
                    [ text <| m.code ]
                ]
            ]


onContentEditableInput : (String -> msg) -> Attribute msg
onContentEditableInput tagger =
    Html.Styled.Events.stopPropagationOn "input"
        (innerText |> Json.map (\str -> ( tagger str, False )))


innerText : Json.Decoder String
innerText =
    Json.at [ "target", "innerText" ] Json.string


linums : Model -> Html msg
linums m =
    div
        [ css
            [ displayGrid
            , gridTemplateColumns "2em"
            , gridAutoRows "1em"
            ]
        ]
        (List.range 1 (getLines m.code)
            |> List.map (\n -> div [] [ text (String.fromInt n) ])
        )
