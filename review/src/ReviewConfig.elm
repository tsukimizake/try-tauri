module ReviewConfig exposing (config)

{-| Do not rename the ReviewConfig module or the config function, because
`elm-review` will look for these.

To add packages that contain rules, add them to this review project using

    `elm install author/packagename`

when inside the directory containing this file.

-}

import Docs.ReviewAtDocs
import NoConfusingPrefixOperator
import NoDebug.Log
import NoDebug.TodoOrToString
import NoExposingEverything
import NoImportingEverything
import NoMissingTypeAnnotation
import NoMissingTypeAnnotationInLetIn
import NoMissingTypeExpose
import NoPrematureLetComputation
import NoSimpleLetBody
import NoUnused.CustomTypeConstructorArgs
import NoUnused.CustomTypeConstructors
import NoUnused.Dependencies
import NoUnused.Exports
import NoUnused.Parameters
import NoUnused.Patterns
import NoUnused.Variables
import Simplify
import Review.Rule exposing (Rule, ignoreErrorsForDirectories)
import NoUnoptimizedRecursion exposing (optOutWithComment)


config : List Rule
config =
    [ Docs.ReviewAtDocs.rule
    , NoConfusingPrefixOperator.rule
    , NoMissingTypeAnnotation.rule
    , NoMissingTypeAnnotationInLetIn.rule
    , NoMissingTypeExpose.rule
    , NoSimpleLetBody.rule
    , NoPrematureLetComputation.rule
    , NoUnused.Dependencies.rule
    , NoUnused.Exports.rule
    , NoUnused.Parameters.rule
    , NoUnused.Patterns.rule
    , NoUnused.Variables.rule
    , Simplify.rule Simplify.defaults
    , NoUnoptimizedRecursion.rule (optOutWithComment "IGNORE TCO")
    ]
    |> List.map (ignoreErrorsForDirectories [ "src/generated/" ])
