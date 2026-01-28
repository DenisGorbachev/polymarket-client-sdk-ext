# Concepts

## Concept

A structure with the following fields:

* Name (required [name](#name))
* Definition (required [definition](#definition))
* Synonyms (optional list of strings)
* Constructors (optional list of [constructors](#constructor) that contains all possible constructors of a concept)
* Examples (optional [listmap](#listmap) of [examples](#example))
* Requirements (optional [listmap](#listmap) of [requirements](#requirement))
* Preferences (optional [listmap](#listmap) of [preferences](#preference))
* Propositions (optional [listmap](#listmap) of [propositions](#proposition))
* Methods (optional map from method names to [method specifications](#method-specification))
* Notes (optional [listmap](#listmap) of [notes](#note))

Examples:

* A definition of a Rust package.
* A definition of a Rust function.

Requirements:

* Synonyms must contain only exact semantic synonyms, not just similar words.

Preferences:

* Should be specific.
* Should contain periods at the ends of the sentences.
  * Reasons:
    * Periods are more common in technical texts than in conversational texts (e.g. reference docs vs forum messages).
    * Periods provide clear boundary cues.

Notes:

* A list of constructors is exhaustive, while a list of examples is not exhaustive.

## Definition

A string value that defines a concept.

Preferences:

* Should end with a period.

Notes:

* May start with an article ("A", "An", "The").
* May start with the name of the concept being defined.
* May be a multiline string.

## Constructor

A string that represents a constructor of a type.

## Example

A [stringtree](#stringtree) that [represents](#representation) an instance of a parent object.

## Requirement

A [stringtree](#stringtree) that [represents](#representation) a boolean test of an instance of a parent object.

Notes:

* If an input doesn't pass the requirement test, then it is not an instance of a parent object.

## Preference

A [stringtree](#stringtree) that [represents](#representation) a less-than-or-equal relation on a pair of instances of a parent object.

Notes:

* Preferences must be sorted by importance (most important first).
* Preferences should be used to make a choice between two inputs that pass the [requirements](#requirement).

## Proposition

A [stringtree](#stringtree) that [represents](#representation) a proposition about an instance of a parent object.

## Method specification

A structure with the following fields:

* Requirements (optional [listmap](#listmap) of [requirements](#requirement))
* Preferences (optional [listmap](#listmap) of [preferences](#preference))
* Propositions (optional [listmap](#listmap) of [propositions](#proposition))
* Notes (optional [listmap](#listmap) of [notes](#note))

Methods:

* Render as Markdown:
  * Requirements:
    * Must output a list where the top-level items correspond to the structure fields.

## Note

A [stringtree](#stringtree) that [represents](#representation) additional information about a parent object.

## Concepts document

A Markdown document that renders a list of [concepts](#concept) with the following elements:

* Heading level 1: document name that starts with "Concepts"
* For each concept:
  * Name: heading level 2.
  * Definition: paragraph after heading.
  * Other fields:
    * Paragraph that is exactly equal to the field name with ":" in the end.
    * Field value:
      * If the field type has a "Render as Markdown" method, then call it, otherwise render the most direct representation (e.g. render strings directly).

Requirements:

* The order of other fields in the document must match the order of other fields in the definition of [concept](#concept).
* A field may not appear twice.
* If the field is empty, it must not be rendered.

## Name

A string that is unique within a document.

## Listmap

A listmap of type A is one of:

* A list of values of type A.
* A map from [names](#name) to values of type A.

Methods:

* Render as Markdown:
  * Requirements:
    * If the value is a list: must output a Markdown list.
    * If the value is a map: must output a Markdown list where each item is rendered as `{key}: {value}`.

Notes:

* If a listmap is a map, its iteration order is the insertion order of keys as they appear in the source text.

## Stringtree

A stringtree is a structure with the following fields:

* Text (string)
* Children (a list of stringtrees)

Methods:

* Render as Markdown:
  * Requirements:
    * Must output a multi-level Markdown list.
      * The "Text" field must be rendered as the top-level list item.
      * The "Children" field must be rendered as child list items.

Notes:

* In Markdown, a stringtree is just a nested list.
* In this document, every "Notes", "Requirements", "Preferences" contain examples of stringtrees as nested lists.

## Representation

A string that passes an [association test](#association-test) evaluated by a specific list of [agents](#agent).

Notes:

* The list of agents is an external input to a program that evaluates the concept document.

## Association test

A function from two strings to a boolean.

Synonyms: is-test.

Examples:

* `is("cow", "animal") == true`
* `is("function", "relation") == true`
* `is("war", "peace") == false`
* `is("slavery", "freedom") == false`
* `is("love", "good") == true`

Notes:

* May represent a term-type relation ("cow" has type "animal").
* May represent a subtype-supertype relation ("function" is a subtype of "relation").
* May represent a term-predicate relation ("leaf" is "green").
* System prompt for association testing by LLMs: "Evaluate whether the statement in the user message is true or false in general".

## Agent

An entity whose goal is to prevent its own termination.

Notes:

* An agent can be artificial or natural (LLM or human).
