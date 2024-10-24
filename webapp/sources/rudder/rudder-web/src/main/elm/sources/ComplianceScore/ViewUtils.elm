module ComplianceScore.ViewUtils exposing (..)

import Html exposing (..)
import Html.Attributes exposing (..)
import List
import List.Extra
import String.Extra
import Markdown

import ComplianceScore.DataTypes exposing (..)
import NodeCompliance.DataTypes exposing (NodeId)
import Compliance.Utils exposing (buildComplianceBar, defaultComplianceFilter, buildTooltipContent)

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

scoreBreakdownList : List GlobalScoreDetails -> List (Html Msg)
scoreBreakdownList scoreDetails = scoreDetails
  |> List.map(\sD ->
    div[class "d-flex flex-column pe-5 align-items-center"]
    [ getScoreBadge sD.value (buildTooltipBadge sD.name sD.message) True
    , label[class "text-center pt-2"][text (String.Extra.humanize sD.name)]
    ]
  )

buildComplianceScoreDetails : ComplianceScoreDetails -> Html msg
buildComplianceScoreDetails complianceScoreDetails =
  div[class "d-flex mb-3 align-items-center"]
  [ label[class "text-end"][text "Compliance"]
  , buildComplianceBar defaultComplianceFilter complianceScoreDetails.details
  ]

buildSystemUpdatesScoreDetails : Maybe SystemUpdatesScoreDetails -> Html msg
buildSystemUpdatesScoreDetails systemUpdatesScoreDetails =
  let
    toBadge : String -> String -> Maybe Int -> Html msg
    toBadge id iconClass value =
      case value of
        Just v  ->
          let
            valueTxt = String.fromInt v
            titleTxt = (String.Extra.humanize id) ++ ": " ++ valueTxt
          in
            span[class ("badge badge-systemupdates " ++ id), title titleTxt][i[class ("fa fa-" ++ iconClass)][], text valueTxt]
        Nothing -> text ""
  in
    case systemUpdatesScoreDetails of
      Just systemUpdatesScore ->
        div[class "d-flex mb-3 align-items-center"]
        [ label[class "text-end"][text "System updates"]
        , div[]
          [ toBadge "security"    "warning" systemUpdatesScore.details.security
          , toBadge "bugfix"      "bug"     systemUpdatesScore.details.bugfix
          , toBadge "enhancement" "plus"    systemUpdatesScore.details.enhancement
          , toBadge "update"      "box"     systemUpdatesScore.details.update
          ]
        ]
      Nothing -> text ""