import {
  isObjectType,
  FieldNode,
  isInterfaceType,
  isNonNullType,
  getNullableType,
  isUnionType,
  GraphQLError,
} from 'graphql';
import { logServiceAndType, errorWithCode, getFederationMetadata } from '../../utils';
import { PostCompositionValidator } from '.';

/**
 * - The fields argument can not have root fields that result in a list
 * - The fields argument can not have root fields that result in an interface
 * - The fields argument can not have root fields that result in a union type
 */
export const keyFieldsSelectInvalidType: PostCompositionValidator = ({
  schema,
}) => {
  const errors: GraphQLError[] = [];

  const types = schema.getTypeMap();
  for (const [typeName, namedType] of Object.entries(types)) {
    if (!isObjectType(namedType)) continue;

    const typeFederationMetadata = getFederationMetadata(namedType);
    if (typeFederationMetadata?.keys) {
      const allFieldsInType = namedType.getFields();
      for (const [serviceName, selectionSets = []] of Object.entries(
        typeFederationMetadata.keys,
      )) {
        for (const selectionSet of selectionSets) {
          for (const field of selectionSet as FieldNode[]) {
            const name = field.name.value;

            // find corresponding field for each selected field
            const matchingField = allFieldsInType[name];
            if (!matchingField) {
              errors.push(
                errorWithCode(
                  'KEY_FIELDS_SELECT_INVALID_TYPE',
                  logServiceAndType(serviceName, typeName) +
                    `A @key selects ${name}, but ${typeName}.${name} could not be found`,
                  namedType.astNode ?? undefined,
                ),
              );
            }

            if (matchingField) {
              if (
                isInterfaceType(matchingField.type) ||
                (isNonNullType(matchingField.type) &&
                  isInterfaceType(getNullableType(matchingField.type)))
              ) {
                errors.push(
                  errorWithCode(
                    'KEY_FIELDS_SELECT_INVALID_TYPE',
                    logServiceAndType(serviceName, typeName) +
                      `A @key selects ${typeName}.${name}, which is an interface type. Keys cannot select interfaces.`,
                    namedType.astNode ?? undefined,
                  ),
                );
              }

              if (
                isUnionType(matchingField.type) ||
                (isNonNullType(matchingField.type) &&
                  isUnionType(getNullableType(matchingField.type)))
              ) {
                errors.push(
                  errorWithCode(
                    'KEY_FIELDS_SELECT_INVALID_TYPE',
                    logServiceAndType(serviceName, typeName) +
                      `A @key selects ${typeName}.${name}, which is a union type. Keys cannot select union types.`,
                    namedType.astNode ?? undefined,
                  ),
                );
              }
            }
          }
        }
      }
    }
  }

  return errors;
};
