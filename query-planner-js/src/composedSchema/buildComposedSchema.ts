import {
  buildASTSchema,
  DocumentNode,
  GraphQLDirective,
  GraphQLNamedType,
  GraphQLSchema,
  isEnumType,
  isIntrospectionType,
  isObjectType,
} from 'graphql';
import { assert } from '../utilities/assert';
import {
  getArgumentValuesForDirective,
  getArgumentValuesForRepeatableDirective,
  isASTKind,
  parseSelections,
} from '../utilities/graphql';
import { MultiMap } from '../utilities/MultiMap';
import {
  FederationFieldMetadata,
  FederationTypeMetadata,
  FieldSet,
  GraphMap,
} from './metadata';

export function buildComposedSchema(document: DocumentNode): GraphQLSchema {
  const schema = buildASTSchema(document);

  // TODO: We should follow the Bootstrap algorithm from the Core Schema spec
  // to handle renames of @core itself.
  const coreName = 'core';

  const coreDirective = schema.getDirective(coreName);
  assert(coreDirective, `Expected core schema, but can't find @core directive`);

  // TODO: We should follow the CollectFeatures algorithm from the Core Schema
  // spec here, and use th collected features to validate feature
  // versions and handle renames.

  const joinName = 'join';

  function getJoinDirective(name: string) {
    const fullyQualifiedName = `${joinName}__${name}`;

    const directive = schema.getDirective(fullyQualifiedName);
    assert(
      directive,
      `Composed schema should define @${fullyQualifiedName} directive`,
    );
    return directive;
  }

  const ownerDirective = getJoinDirective('owner');
  const typeDirective = getJoinDirective('type');
  const fieldDirective = getJoinDirective('field');
  const endpointDirective = getJoinDirective('endpoint');

  const graphEnumType = schema.getType(`${joinName}__Graph`);
  assert(isEnumType(graphEnumType));

  const graphMap: GraphMap = Object.create(null);

  schema.extensions = {
    ...schema.extensions,
    federation: {
      graphs: graphMap,
    },
  };

  for (const graphValue of graphEnumType.getValues()) {
    const name = graphValue.name;

    const endpointDirectiveArgs = getArgumentValuesForDirective(
      endpointDirective,
      graphValue.astNode!,
    );
    assert(
      endpointDirectiveArgs,
      `${graphEnumType.name} value ${name} in composed schema should have a @${endpointDirective.name} directive`,
    );

    const serviceName: string = endpointDirectiveArgs['serviceName'];
    const url: string = endpointDirectiveArgs['url'];

    graphMap[name] = {
      serviceName,
      url,
    };
  }

  for (const type of Object.values(schema.getTypeMap())) {
    if (isIntrospectionType(type)) continue;

    // We currently only allow join spec directives on object types.
    if (!isObjectType(type)) continue;

    assert(
      type.astNode,
      `GraphQL type "${type.name}" should contain AST nodes`,
    );

    const ownerDirectiveArgs = getArgumentValuesForDirective(
      ownerDirective,
      type.astNode,
    );

    const typeMetadata: FederationTypeMetadata = ownerDirectiveArgs
      ? {
          serviceName: graphMap[ownerDirectiveArgs?.['graph']].serviceName,
          keys: new MultiMap(),
          isValueType: false,
        }
      : {
          isValueType: true,
        };

    type.extensions = {
      ...type.extensions,
      federation: typeMetadata,
    };

    const typeDirectivesArgs = getArgumentValuesForRepeatableDirective(
      typeDirective,
      type.astNode,
    );

    assert(
      !(typeMetadata.isValueType && typeDirectivesArgs.length >= 1),
      `GraphQL type "${type.name}" cannot have a @${typeDirective.name} \
directive without an $${ownerDirective.name} directive`,
    );

    for (const typeDirectiveArgs of typeDirectivesArgs) {
      const serviceName = graphMap[typeDirectiveArgs['graph']].serviceName;

      const keyFields = parseFieldSet(typeDirectiveArgs['key']);

      typeMetadata.keys?.add(serviceName, keyFields);
    }

    for (const fieldDef of Object.values(type.getFields())) {
      assert(
        fieldDef.astNode,
        `Field "${type.name}.${fieldDef.name}" should contain AST nodes`,
      );

      const fieldDirectiveArgs = getArgumentValuesForDirective(
        fieldDirective,
        fieldDef.astNode,
      );

      if (!fieldDirectiveArgs) continue;

      const fieldMetadata: FederationFieldMetadata = {
        serviceName: graphMap[fieldDirectiveArgs?.['graph']]?.serviceName,
      };

      fieldDef.extensions = {
        ...fieldDef.extensions,
        federation: fieldMetadata,
      };

      if (fieldDirectiveArgs) {
        const { requires, provides } = fieldDirectiveArgs;

        if (requires) {
          fieldMetadata.requires = parseFieldSet(requires);
        }

        if (provides) {
          fieldMetadata.provides = parseFieldSet(provides);
        }
      }
    }
  }

  // We filter out schema elements that should not be exported to get to the
  // API schema.

  const schemaConfig = schema.toConfig();

  return new GraphQLSchema({
    ...schemaConfig,
    types: schemaConfig.types.filter(isExported),
    directives: schemaConfig.directives.filter(isExported),
  });

  // TODO: Implement the IsExported algorithm from the Core Schema spec.
  function isExported(element: NamedSchemaElement) {
    return !(isAssociatedWithFeature(element, coreName) || isAssociatedWithFeature(element, joinName))
  }

  function isAssociatedWithFeature(
    element: NamedSchemaElement,
    featureName: string,
  ) {
    return (
      element.name === `${featureName}` ||
      element.name.startsWith(`${featureName}__`)
    );
  }
}

type NamedSchemaElement = GraphQLDirective | GraphQLNamedType;

function parseFieldSet(source: string): FieldSet {
  const selections = parseSelections(source);

  assert(
    selections.every(isASTKind('Field', 'InlineFragment')),
    `Field sets may not contain fragments spreads, but found: "${source}"`,
  );

  assert(selections.length > 0, `Field sets may not be empty`);

  return selections;
}