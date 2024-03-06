module Score.ViewUtils exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import List
import String.Extra

import Score.DataTypes exposing (..)
import Compliance.Utils exposing (buildTooltipContent)

scoreLabel : ScoreValue -> String
scoreLabel score =
  case score of
    A -> "A"
    B -> "B"
    C -> "C"
    D -> "D"
    E -> "E"
    F -> "F"
    X -> "X"

buildTooltipBadge : String -> String -> List (Attribute msg)
buildTooltipBadge name msg =
  [ attribute "data-bs-toggle" "tooltip"
  , attribute "data-bs-placement" "top"
  , title (buildTooltipContent (String.Extra.humanize name) msg)
  ]

getScoreBadge : ScoreValue -> List (Attribute Msg) -> Bool -> Html Msg
getScoreBadge score tooltipAttributes smallSize =
  span
    ( List.append
      [ class ("badge-compliance-score " ++ (scoreLabel score) ++ (if smallSize then " sm" else ""))]
      tooltipAttributes
    )[]

scoreBreakdownList : List Score -> List (Html Msg)
scoreBreakdownList scoreDetails = scoreDetails
  |> List.map(\sD ->
    div[class "d-flex flex-column pe-5 align-items-center"]
    [ getScoreBadge sD.value (buildTooltipBadge sD.scoreId sD.message) True
    , label[class "text-center pt-2"][text (String.Extra.humanize sD.scoreId)]
    ]
  )