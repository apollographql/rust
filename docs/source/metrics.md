---
title: Federated trace data
description: How federated tracing works
---

One of the many benefits of using GraphQL as an API layer is that it enables fine-grained [tracing](https://www.apollographql.com/docs/platform/performance/#traces) of every API call. The Apollo platform supports consuming and aggregating these traces to provide detailed insights into your GraphQL layer's performance and usage.

Apollo Federation supports sending **federated traces** from the Apollo gateway, which are constructed from timing and error information provided by your subgraphs. These federated traces capture the subgraph-level details in the shape of the query plan, which is sent to Apollo's [metrics ingress](https://www.apollographql.com/docs/studio/setup-analytics/#sending-metrics-to-the-reporting-endpoint) by default, and aggregated into query-level stats and field-level stats. The overall flow of a federated trace is as follows:

1. The gateway receives an operation from a client.
2. The gateway constructs a query plan for the operation, delegating sub-queries to subgraphs.
3. For each [fetch](https://www.apollographql.com/docs/federation/federation-spec/#fetch-service-capabilities) to a subgraph, a response is received.
4. The [`extensions`](https://www.apollographql.com/docs/resources/graphql-glossary/#extensions) of each response includes a trace from the sub-query.
5. The gateway collects the set of sub-query traces from subgraphs and arranges them in the shape of the query plan.
6. The federated trace is sent to the Apollo [metrics ingress](https://www.apollographql.com/docs/studio/setup-analytics/#sending-metrics-to-the-reporting-endpoint) for processing.

In summary, subgraphs report timing and error information to the gateway, and the gateway is responsible for reporting those metrics to Apollo.

## Turning it on

Ensure that all dependencies on `apollo-server` are at version `2.7.0` or higher. Provide an API key to your gateway via the `APOLLO_KEY` environment variable for the gateway to report metrics to the default ingress. To ensure that subgraphs do not report metrics as well, either do not provide them with an `APOLLO_KEY` or install the [`ApolloServerPluginUsageReportingDisabled` plugin](https://www.apollographql.com/docs/apollo-server/api/plugin/usage-reporting/) in your `ApolloServer`.

These options will cause the Apollo gateway to collect tracing information from the underlying subgraphs and pass them on, along with the query plan, to the Apollo metrics ingress. Currently, only Apollo Server supports detailed metrics insights as a subgraph, but we would love to work with you to implement the protocol in other languages!

> NOTE: By default, metrics will be reported to the `current` variant. To change the variant for reporting, set the `APOLLO_GRAPH_VARIANT` environment variable.

## How tracing data is exposed from a subgraph

> NOTE: this section details how the gateway communicates with subgraphs around encoded tracing information and is not necessary to understand in order to enable federated tracing.

The Apollo gateway looks to the `extensions` field of all subgraph responses for the presence of an `ftv1` field. This field contains a representation of the tracing information for the sub-query that was executed against the subgraph, sent as the Base64 encoding of the [protobuf representation](https://github.com/apollographql/apollo-server/blob/main/packages/apollo-reporting-protobuf/src/reports.proto) of the trace. In order to request this information of subgraph, the gateway will send the header pair `'apollo-federation-include-trace': 'ftv1'` on fetches if configured to collect metrics, as per above. By default, a federated Apollo Server subgraph will recognize this header pair and attach tracing information in extensions of the response.

## How traces are constructed and aggregated

The Apollo gateway constructs traces in the shape of the [query plan](https://www.apollographql.com/docs/federation/implementing/#inspecting-query-plans), embedding an individual `Trace` for each fetch that is performed in the query plan. This indicates the sub-query traces as well as which order they were fetched from the underlying subgraphs. The field-level statistics that the Apollo Platform aggregates from these traces are collected over the fields over which the operation was executed **in the federated subgraphs**. In other words, field stats are collected based on the operations the query planner makes rather than the operations that the clients make. On the other hand, operation-level statistics are aggregated over the operations executed **by the client**, which means that even if query-planning changes, statistics will still be corresponded to the same client-delivered operation.

## How errors work

The Apollo Platform provides functionality to modify error details for the client, via the [`formatError`](https://www.apollographql.com/docs/apollo-server/features/errors/#for-the-client-response) option. Additionally, there is functionality to support modifying error details for the metrics ingress, via the [`rewriteError`](https://www.apollographql.com/docs/apollo-server/data/errors/#for-apollo-studio-reporting) option to the [inline trace plugin](https://www.apollographql.com/docs/apollo-server/api/plugin/inline-trace/).

When modifying errors for the client, you might want to use this option to hide implementation details, like database errors, from your users. When modifying errors for reporting, you might want to obfuscate or redact personal information, like user IDs or emails.

Since federated metrics collection works by collecting latency and error information from a set of distributed subgraphs, **these options are respected from those subgraphs** as well as from the gateway level. Subgraphs will embed errors in their `ftv1` extension after the `rewriteError` method (passed to the inline trace plugin in the subgraph, not the usage reporting plugin in the gateway!) is applied, and the gateway will only report the errors that are sent via that extension, ignoring the format that downstream errors are reported to end users. This functionality enables subgraph implementers to determine how error information should be displayed to both users and in metrics without needing the gateway to contain any logic that might be subgraph-specific.
