import {
  assertValidSchema,
  GraphQLDirective,
  GraphQLNamedType,
  GraphQLSchema,
  isDirective,
} from 'graphql';
import { supportedFeatures } from './buildComposedSchema';
import { removeInaccessibleElements } from './removeInaccessibleElements';

declare module 'graphql' {
  interface GraphQLSchema {
    __apiSchema: GraphQLSchema | undefined;
  }
}

export function toAPISchema(schema: GraphQLSchema): GraphQLSchema {
  // Similar to `validateSchema()` caching its result in `__validationErrors`,
  // we cache the API schema to avoid recomputing it unnecessarily.

  if (schema.__apiSchema) {
    return schema.__apiSchema;
  }

  assertValidSchema(schema);

  schema = removeInaccessibleElements(schema);

  // TODO: We should get a list of feature names from the schema itself, rather
  // than relying on a static list of supported features.
  const featureNames = supportedFeatures.map(feature => feature.name);

  // We filter out schema elements that should not be exported to get to the
  // API schema.

  const schemaConfig = schema.toConfig();

  const apiSchema = new GraphQLSchema({
    ...schemaConfig,
    types: schemaConfig.types.filter(isExported),
    directives: schemaConfig.directives.filter(isExported),
  });

  assertValidSchema(apiSchema);

  schema.__apiSchema = apiSchema;

  return apiSchema;

  // TODO: Implement the IsExported algorithm from the Core Schema spec.
  function isExported(element: NamedSchemaElement) {
    for (const featureName of featureNames) {
      // For now, we skip any element that is associated with a feature.
      if (isAssociatedWithFeature(element, featureName)) {
        return false;
      }
    }
    return true;
  }
}

type NamedSchemaElement = GraphQLDirective | GraphQLNamedType;

function isAssociatedWithFeature(
  element: NamedSchemaElement,
  featureName: string,
) {
  return (
    // Only directives can use the unprefixed feature name.
    (isDirective(element) && element.name === featureName) ||
    element.name.startsWith(`${featureName}__`)
  );
}
