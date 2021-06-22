export { queryPlanSerializer, astSerializer } from './snapshotSerializers';
export { prettyFormatQueryPlan } from './prettyFormatQueryPlan';

export * from './QueryPlan';
import { QueryPlan } from './QueryPlan';

export * from './composedSchema';
import {
  buildOperationContext,
  buildQueryPlan,
  BuildQueryPlanOptions,
  OperationContext,
} from './buildQueryPlan';
import {buildComposedSchema} from "./composedSchema";
import { GraphQLSchema } from 'graphql';
export { BuildQueryPlanOptions, OperationContext, buildComposedSchema, buildOperationContext, buildQueryPlan };

// There isn't much in this class yet, and I didn't want to make too many
// changes at once, but since we were already storing a pointer to a
// Rust query planner instance in the gateway, I think it makes sense to retain
// that structure. I suspect having a class instead of a stand-alone function
// will come in handy to encapsulate schema-derived data that is used during
// planning but isn't operation specific. The next step is likely to be to
// convert `buildQueryPlan` into a method.
export class QueryPlanner {
  constructor(public readonly schema: GraphQLSchema) {}

  // TODO(#632): We should change the API to avoid confusion, because
  // taking an operationContext with a schema on it isn't consistent
  // with a QueryPlanner instance being bound to a single schema.
  buildQueryPlan(
    operationContext: OperationContext,
    options?: BuildQueryPlanOptions,
  ): QueryPlan {
    return buildQueryPlan(
      { ...operationContext, schema: this.schema },
      options,
    );
  }
}
