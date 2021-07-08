import {
  visit,
  visitWithTypeInfo,
  TypeInfo,
  parse,
  GraphQLSchema,
  GraphQLError,
  specifiedDirectives,
} from 'graphql';
import { buildSchemaFromSDL } from 'apollo-graphql';
import apolloTypeSystemDirectives from '../../../directives';
import { ServiceDefinition } from '../../types';
import {
  findDirectivesOnNode,
  isStringValueNode,
  logServiceAndType,
  errorWithCode,
} from '../../utils';
import { isNotNullOrUndefined } from '../../../utilities';

/**
 *  For every @key directive, it must reference a field marked as @external
 */
export const keyFieldsMissingExternal = ({
  name: serviceName,
  typeDefs,
}: ServiceDefinition) => {
  const errors: GraphQLError[] = [];

  // Build an array that accounts for all key directives on type extensions.
  let keyDirectiveInfoOnTypeExtensions: {
    typeName: string;
    keyArgument: string;
  }[] = [];
  visit(typeDefs, {
    ObjectTypeExtension(node) {
      const keyDirectivesOnTypeExtension = findDirectivesOnNode(
        node,
        'key',
      );

      const keyDirectivesInfo = keyDirectivesOnTypeExtension
        .map(keyDirective =>
          keyDirective.arguments &&
          isStringValueNode(keyDirective.arguments[0].value)
            ? {
                typeName: node.name.value,
                keyArgument: keyDirective.arguments[0].value.value,
              }
            : null,
        )
        .filter(isNotNullOrUndefined);

      keyDirectiveInfoOnTypeExtensions.push(...keyDirectivesInfo);
    },
  });

  // this allows us to build a partial schema
  let schema = new GraphQLSchema({
    query: undefined,
    directives: [...specifiedDirectives, ...apolloTypeSystemDirectives],
  });
  try {
    schema = buildSchemaFromSDL(typeDefs, schema);
  } catch (e) {
    errors.push(e);
    return errors;
  }

  const typeInfo = new TypeInfo(schema);

  for (const { typeName, keyArgument } of keyDirectiveInfoOnTypeExtensions) {
    const keyDirectiveSelectionSet = parse(
      `fragment __generated on ${typeName} { ${keyArgument} }`,
    );
    visit(
      keyDirectiveSelectionSet,
      visitWithTypeInfo(typeInfo, {
        Field(node) {
          const fieldDef = typeInfo.getFieldDef();
          const parentType = typeInfo.getParentType();
          if (parentType) {
            if (!fieldDef) {
              // TODO: find all fields that have @external and suggest them / heursitic match
              errors.push(
                errorWithCode(
                  'KEY_FIELDS_MISSING_EXTERNAL',
                  logServiceAndType(serviceName, parentType.name) +
                    `A @key directive specifies a field which is not found in this service. Add a field to this type with @external.`,
                  node,
                ),
              );
              return;
            }
            const externalDirectivesOnField = findDirectivesOnNode(
              fieldDef.astNode,
              'external',
            );

            if (externalDirectivesOnField.length === 0) {
              errors.push(
                errorWithCode(
                  'KEY_FIELDS_MISSING_EXTERNAL',
                  logServiceAndType(serviceName, parentType.name) +
                    `A @key directive specifies the \`${fieldDef.name}\` field which has no matching @external field.`,
                  fieldDef.astNode ?? undefined,
                ),
              );
            }
          }
        },
      }),
    );
  }

  return errors;
};
