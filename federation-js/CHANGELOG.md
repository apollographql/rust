# CHANGELOG for `@apollo/federation`

## vNEXT

> The changes noted within this `vNEXT` section have not been released yet.  New PRs and commits which introduce changes should include an entry in this `vNEXT` section as part of their development.  When a release is being prepared, a new header will be (manually) created below and the appropriate changes within that release will be moved into the new section.

- Skip missing types while iterating over field directive usages. It's possible to capture directive usages on fields whose types don't actually exist in the schema (due to invalid composition). See PR for more details. [PR #868](https://github.com/apollographql/federation/pull/868)
- Prevent the usage of either @tag or @inaccessible on fields that are marked as @external. [PR #869](https://github.com/apollographql/federation/pull/869)

## v0.26.0

- Capture and propagate `@tag` and `@inaccessible` directives during composition from subgraph to supergraph SDL. This unblocks upcoming work for schema construction, schema filtering (API schemas), and future Studio features. [PR #756](https://github.com/apollographql/federation/pull/756)

## v0.25.2

- Sort composed schema using graphql-js's `lexicographicSortSchema` for schema ordering determinism independent of serviceList ordering. [PR #824](https://github.com/apollographql/federation/pull/824)

## v0.25.1

-  `ASTNodeWithDirectives` now includes all AST nodes with the `directives` field on it. [PR #755](https://github.com/apollographql/federation/pull/755)

## Update spec

- Add `repeatable` keyword to the @key directive in federation spec. [PR #758](https://github.com/apollographql/federation/pull/758)

## v0.25.0

- Composition errors now include `locations` corresponding to the line number & column in the subgraph SDL. [PR #686](https://github.com/apollographql/federation/pull/686)

## v0.24.0

- Expand the range of supported `node` versions in the package's `engines` specifier to include the now-tested Node.js `16`. [PR #713](https://github.com/apollographql/federation/pull/713)

## v0.23.2

- Remove lingering `core-js` polyfill imports, they're no longer needed (since `@apollo/gateway@0.15.0` dropped support for <= Node.js v10) and their presence is problematic since `core-js` isn't defined as a dependency within the package. Update `apollo-graphql` dependency which resolves a missing dependency (`sha.js`) within that package. [PR #699](https://github.com/apollographql/federation/pull/699)

## v0.23.1

- This change is mostly a set of follow-up changes for PR #622. Most of these changes are internal (renaming, etc.). Some noteworthy changes worth mentioning are: a switch to graphql-js's `stripIgnoredCharacters` during field set printing, an update to the `join__Enum` generation algorithm, and some additional assertions. [PR #656](https://github.com/apollographql/federation/pull/656)

## v0.23.0

- __BREAKING__ - Update CSDL to the new core schema format, implementing the currently-being-introduced core and join specs. `composeAndValidate` now returns `supergraphSdl` in the new format instead of `composedSdl` in the previous CSDL format. [PR #622](https://github.com/apollographql/federation/pull/622)

## v0.22.0

- No changes to the package itself, though there are some small changes to the way this package is compiled and the tests within this package due to the changes in [PR #453](https://github.com/apollographql/federation/pull/453)

## v0.21.2

- Fix an erroneous `break` to `continue`, follow-up fix for #478 [PR #481](https://github.com/apollographql/federation/pull/481)

## v0.21.1

- Ignore thrown errors from `extendSchema` during composition (these particular errors are already validated against and returned as composition errors) [PR #478](https://github.com/apollographql/federation/pull/478)

## v0.21.0

- __BREAKING__: Drop support for Node.js 8 and Node.js 10.  This package now only targets Node.js 12+ LTS (Long-Term Support) versions, the same as `@apollo/gateway`, which first received this treatment in https://github.com/apollographql/apollo-server/pull/4031.  Node.js 8 has already lapsed from the [Node.js Foundation's LTS schedule](https://github.com/nodejs/release) and Node.js 10 (in _Maintenance LTS_ right now) is targeted to be end-of-life'd (EOL) at the end of April 2021.  [PR #311](https://github.com/apollographql/federation/pull/311)
- Export `GraphQLSchemaModule` type. [PR #293](https://github.com/apollographql/federation/pull/293)
- __BREAKING__: Remove `ComposedGraphQLSchema` type as it's no longer needed. This is breaking because it was part of the public API, though we strongly believe nobody was or should have had any need for this type. Update `composeAndValidate` function signature for better typing, and align the `compose` function signature with that of `composeAndValidate` [PR #278](https://github.com/apollographql/federation/pull/278)
## v0.20.7

- Fix check for value types when having fields and arguments with the same name [PR #280](https://github.com/apollographql/federation/pull/280)

## v0.20.6

- No changes, but please note that `v0.20.5` was a botched release, with no update to the `@apollo/query-planner-wasm` package that was needed. If you're seeing an error similar to `This data graph is missing a valid configuration. unreachable`, please upgrade to at least this patch release.

## v0.20.5

- Apply `repeatable` keyword to CSDL schema directives `@key` and `@graph. [PR #285](https://github.com/apollographql/apollo-federation/pull/285)

## v0.20.4

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.20.3

- Fix warning for non-matching `@external` types when the declaration's type is non-null or a list [PR #4392](https://github.com/apollographql/apollo-server/pull/4392)

## v0.20.2

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.20.1

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.20.0

- __FIX__: CSDL complex `@key`s shouldn't result in an unparseable document [PR #4490](https://github.com/apollographql/apollo-server/pull/4490)
- __FIX__: Value type validations - restrict unions, scalars, enums [PR #4496](https://github.com/apollographql/apollo-server/pull/4496)
- __FIX__: Composition - aggregate interfaces for types and interfaces in composed schema [PR #4497](https://github.com/apollographql/apollo-server/pull/4497)
- __FIX__: Create new `@key` validations to prevent invalid compositions [PR #4498](https://github.com/apollographql/apollo-server/pull/4498)
- CSDL: make `fields` directive args parseable [PR #4489](https://github.com/apollographql/apollo-server/pull/4489)

## v0.19.1

- Include new directive definitions in CSDL [PR #4452](https://github.com/apollographql/apollo-server/pull/4452)

## v0.19.0

- New federation composition format. Capture federation metadata in SDL [PR #4405](https://github.com/apollographql/apollo-server/pull/4405)

## v0.18.1

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.18.0

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.17.0

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.16.11

- Reinstate typings for `make-fetch-happen` at the `apollo-gateway` project level (and now, additionally, `apollo-server-plugin-operation-registry`) [PR #4333](https://github.com/apollographql/apollo-server/pull/4333)

## 0.16.10

- The default branch of the repository has been changed to `main`.  As this changed a number of references in the repository's `package.json` and `README.md` files (e.g., for badges, links, etc.), this necessitates a release to publish those changes to npm. [PR #4302](https://github.com/apollographql/apollo-server/pull/4302)
- __BREAKING__: Move federation metadata from custom objects on schema nodes over to the `extensions` field on schema nodes which are intended for metadata. This is a breaking change because it narrows the `graphql` peer dependency from `^14.0.2` to `^14.5.0` which is when [`extensions` were introduced](https://github.com/graphql/graphql-js/pull/2097) for all Type System objects. [PR #4302](https://github.com/apollographql/apollo-server/pull/4313)

## 0.16.9

- Handle `@external` validation edge case for interface implementors [#4284](https://github.com/apollographql/apollo-server/pull/4284)

## 0.16.7

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.16.6

- In-house `Maybe` type which was previously imported from `graphql` and has been moved in `v15.1.0`. [#4230](https://github.com/apollographql/apollo-server/pull/4230)
- Remove remaining common primitives from SDL during composition. This is a follow up to [#4209](https://github.com/apollographql/apollo-server/pull/4209), and additionally removes directives which are included in a schema by default (`@skip`, `@include`, `@deprecated`, and `@specifiedBy`) [#4228](https://github.com/apollographql/apollo-server/pull/4209)

## v0.16.5

- Remove federation primitives from SDL during composition. This allows for services to report their *full* SDL from the `{ _service { sdl } }` query as opposed to the previously limited SDL without federation definitions. [#4209](https://github.com/apollographql/apollo-server/pull/4209)

## v0.16.4

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.16.3

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.16.2

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.16.1

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.16.0

- No changes. This package was major versioned to maintain lockstep versioning with @apollo/gateway.

## v0.15.1

- Export `defaultRootOperationNameLookup` and `normalizeTypeDefs`; needed by `@apollo/gateway` to normalize root operation types when reporting to Apollo Graph Manager. [#4071](https://github.com/apollographql/apollo-server/pull/4071)

## v0.15.0

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/e37384a49b2bf474eed0de3e9f4a1bebaeee64c7)

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.14.1

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/b898396e9fcd3b9092b168f9aac8466ca186fa6b)

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.14.0

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/71a3863f59f4ab2c9052c316479d94c6708c4309)

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.13.2

- Only changes in the similarly versioned `@apollo/gateway` package.

## v0.12.1

- Fix `v0.12.0` regression: Preserve the `@deprecated` type-system directive as a special case when removing type system directives during composition, resolving an unintentional breaking change introduced by [#3736](https://github.com/apollographql/apollo-server/pull/3736). [#3792](https://github.com/apollographql/apollo-server/pull/3792)

## v0.12.0

- Strip all Type System Directives during composition [#3736](https://github.com/apollographql/apollo-server/pull/3736)
- Prepare for changes in upcoming `graphql@15` release. [#3712](https://github.com/apollographql/apollo-server/pull/3712)

## v0.11.1

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/2a4c654986a158aaccf947ee56a4bfc48a3173c7)

- Ignore TypeSystemDirectiveLocations during composition [#3536](https://github.com/apollographql/apollo-server/pull/3536)

## v0.11.0

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/93002737d53dd9a50b473ab9cef14849b3e539aa)

- Begin supporting executable directives in federation [#3464](https://github.com/apollographql/apollo-server/pull/3464)

## v0.10.3

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/3cdde1b7a71ace6411fbacf82a1a61bf737444a6)

- Remove `apollo-env` dependency to eliminate circular dependency between the two packages. This circular dependency makes the tooling repo unpublishable when `apollo-env` requires a version bump. [#3463](https://github.com/apollographql/apollo-server/pull/3463)

## v0.10.1

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/aa200ce24b834320fc79d2605dac340b37d3e434)

- Use reference-equality when omitting validation rules during composition. [#3338](https://github.com/apollographql/apollo-server/pull/3338)

## v0.10.0

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/6100fb5e0797cd1f578ded7cb77b60fac47e58e3)

- Remove federation directives from composed schema [#3272](https://github.com/apollographql/apollo-server/pull/3272)
- Do not remove Query/Mutation/Subscription types when schema is included if schema references those types [#3260](https://github.com/apollographql/apollo-server/pull/3260)

## v0.9.1

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/029c8dca3af812ee70589cdb6de749df3d2843d8)

- Fix value type behavior within composition and execution [#3182](https://github.com/apollographql/apollo-server/pull/2922)

## v0.6.8

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/5974b2ce405a06bc331230400b9073f6381738d3)

- Support __typenames if defined by an incoming operation [#2922](https://github.com/apollographql/apollo-server/pull/2922)

## v0.6.7

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/2ea5887acc43461a5539071f4981a5f70e0d0652)

- Fix bug in externalUnused validation [#2919](https://github.com/apollographql/apollo-server/pull/2919)

## v0.6.6

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/183de5f112324def375a45c239955e1bf1608fae)

- Allow specified directives during validation (@deprecated) [#2823](https://github.com/apollographql/apollo-server/pull/2823)

## v0.6.1

> [See complete versioning details.](https://github.com/apollographql/apollo-server/commit/1209839c01b4cac1eb23f42c747296dd9507a8ac)

- Normalize SDL in a normalization step before validation [#2771](https://github.com/apollographql/apollo-server/pull/2771)
