# 3. Target version for dependencies

Date: 2026-03-13

## Status

Accepted

## Context

We need to select a minimum stable version to target the dependencies and C++ runtimes (like: compilers and so on).

This can be relevant when deciding which features to support and which feature to block or provide mitigation.

Assuming that the platforms we targets hits the minimum required features that we use, an example would be Glaze that requires C++23 or greater.

## Decision

We have decided to adopt the latest version of the packages as supported by the current latest version of Debian stable.

For example, if the current last version of CMake in Debian stable is 3.0, the project shall target CMake 3.0.

## Consequences

As we target the minimum version for Debian stable, a distro known for it's commitment into stable versions, we might lose newer features and forces us to keep on older versions of libraries or compilers to guarantee support on all distributions.
