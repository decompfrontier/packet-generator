# 2. Enum type implementation in KDL

Date: 2026-01-15

## Status

Accepted

## Context

A JSON may contain one or more fields that are mutually exclusive.
We currently indicate such cases as the fields being `optional` and let the
generator handle it.

```kdl
json Foo {
    // ...

    field a type="str" {
        // ...
        optional
    }

    field b type="str" {
        // ...
        optional
    }
}
```

This poses a problem when the set of distinct fields has many sub-sets,
for example it may happen that the mutual exclusion has to apply to different
sets of fields in the same definition:

```kdl
json Foo {
    // ...

    field a type="str" {
        doc "Conflicts with field b"
        optional
    }

    field b type="str" {
        doc "Conflicts with field a"
        optional
    }

    // Later:

    field c type="str" {
        doc "Conflicts with field d"
        optional
    }

    field d type="str" {
        doc "Conflicts with field c"
        optional
    }
}
```

The parser does not make the distinction between the exclusivity of different
sets of fields.


## Decision

We are ignoring this requirement and continue to treat all mutually-exclusive
sets as all fields being optional.
The alternative would be to implement a `flat-enum` definition that automatically
flattens to the correct field-group definition, essentially reimplementing
sum-types.

In practice, Glaze does not handle `std::variant<...>`, so the generator would
have to generate `std::optional<...>` in any case; such addition would only
allow the generator to generate the glue code to parse the mutually-excluding
groups correctly.

## Consequences

If, in the future, a new JSON has many mutually-exclusive fields, a user-provided
glue code may be needed.
