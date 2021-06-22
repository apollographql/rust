// This file itself gets its types automatically in VSCode.
//
// However, if you change the exports from this file, be sure to provide the
// types in the do_compose.js which is alongside (and injected in the runtime!)
export { composeAndValidate } from "@apollo/federation";
export { buildComposedSchema, buildQueryPlan, buildOperationContext } from "@apollo/query-planner";
export { parse as parseGraphqlDocument } from "graphql";
