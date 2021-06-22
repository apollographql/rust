/** @typedef {{typeDefs: string, name: string, url?: string;}} ServiceDefinition */

/**
 * This `bridge` is defined as a global by the runtime we define in Rust.
 * We declare this as a `var` here only to allow the TSDoc type annotation to be
 * applied to it. Running `var` multiple times has no effect.
 * @type {{
 *   parseGraphqlDocument: import('graphql').parse,
 *   buildOperationContext: import('../../query-planner-js').buildOperationContext,
 *   buildQueryPlan: import('../../query-planner-js').buildQueryPlan
 *   buildComposedSchema: import('../../query-planner-js').buildComposedSchema
 * }} */
var bridge;

/**
 * @type {OperationalContext}
 */
var context;

/**
 * @type {QueryPlanOptions}
 */
var options;

if (!context) {
    throw new Error("Error in JS-Rust-land: context is missing.");
}
if (!options) {
    throw new Error("Error in JS-Rust-land: options is missing.");
}

try {
    let schema = bridge.parseGraphqlDocument(context.schema);
    let composedSchema = bridge.buildComposedSchema(schema);
    let query = bridge.parseGraphqlDocument(context.query);
    let operationalContext = bridge.buildOperationContext(composedSchema, query, context.operation);
    const plan = bridge.buildQueryPlan(operationalContext, options);

    done(
        plan.errors ? {Err: plan.errors} : {Ok: JSON.stringify(plan, null, 2)},
    );
} catch (err) {
    done({Err: [err]});
}

