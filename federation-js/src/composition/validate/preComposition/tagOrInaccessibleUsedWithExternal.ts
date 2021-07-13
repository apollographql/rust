import {
  visit,
  GraphQLError,
  ObjectTypeExtensionNode,
  InterfaceTypeExtensionNode,
} from 'graphql';
import { ServiceDefinition } from '../../types';

import {
  logServiceAndType,
  errorWithCode,
  findDirectivesOnNode,
} from '../../utils';

/**
 * Ensure there are no fields with @external and _either_ @tag or @inaccessible.
 * We're only concerned with the xTypeExtension nodes because @external is
 * already disallowed on base types.
 */
export const tagOrInaccessibleUsedWithExternal = ({
  name: serviceName,
  typeDefs,
}: ServiceDefinition) => {
  const errors: GraphQLError[] = [];

  visit(typeDefs, {
    ObjectTypeExtension: fieldsVisitor,
    InterfaceTypeExtension: fieldsVisitor,
  });

  function fieldsVisitor(
    typeDefinition: ObjectTypeExtensionNode | InterfaceTypeExtensionNode,
  ) {
    if (!typeDefinition.fields) return;
    for (const fieldDefinition of typeDefinition.fields) {
      const tagDirectives = findDirectivesOnNode(fieldDefinition, 'tag');
      const hasTagDirective = tagDirectives.length > 0;
      const inaccessibleDirectives = findDirectivesOnNode(
        fieldDefinition,
        'inaccessible',
      );
      const hasInaccessibleDirective = inaccessibleDirectives.length > 0;
      const hasExternalDirective =
        findDirectivesOnNode(fieldDefinition, 'external').length > 0;
      if (hasTagDirective && hasExternalDirective) {
        errors.push(
          errorWithCode(
            'TAG_USED_WITH_EXTERNAL',
            logServiceAndType(
              serviceName,
              typeDefinition.name.value,
              fieldDefinition.name.value,
            ) +
              `Found illegal use of @tag directive. @tag directives cannot currently be used in tandem with an @external directive.`,
            tagDirectives,
          ),
        );
      }

      if (hasInaccessibleDirective && hasExternalDirective) {
        errors.push(
          errorWithCode(
            'INACCESSIBLE_USED_WITH_EXTERNAL',
            logServiceAndType(
              serviceName,
              typeDefinition.name.value,
              fieldDefinition.name.value,
            ) +
              `Found illegal use of @inaccessible directive. @inaccessible directives cannot currently be used in tandem with an @external directive.`,
            inaccessibleDirectives,
          ),
        );
      }
    }
  }

  return errors;
};
