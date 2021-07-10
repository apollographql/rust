import deepEqual from 'deep-equal';
import { ArgumentDefinition, InputType, isInputObjectType, isListType, isNonNullType } from './definitions';
import { GraphQLError } from 'graphql';
import { didYouMean, suggestionList } from './suggestions';

export function valueToString(v: any): string {
  if (v === undefined || v === null) {
    return "null";
  }

  if (Array.isArray(v)) {
    return '[' + v.map(e => valueToString(e)).join(', ') + ']';
  }

  if (typeof v === 'object') {
    return '{' + Object.keys(v).map(k => `${k}: ${valueToString(v[k])}`).join(', ') + '}';
  }

  if (typeof v === 'string') {
    return JSON.stringify(v);
  }

  return String(v);
}

export function valueEquals(a: any, b: any): boolean {
  return deepEqual(a, b);
}

function buildError(message: string): Error {
  // Maybe not the right error for this?
  return new Error(message);
}

export function applyDefaultValues(value: any, type: InputType): any {
  if (value === null) {
    if (isNonNullType(type)) {
      throw new GraphQLError(`Invalid null value for non-null type ${type} while computing default values`);
    }
    return null;
  }

  if (isNonNullType(type)) {
    return applyDefaultValues(value, type.ofType);
  }

  if (isListType(type)) {
    if (Array.isArray(value)) {
      return value.map(v => applyDefaultValues(v, type.ofType));
    } else {
      return applyDefaultValues(value, type.ofType);
    }
  }

  if (isInputObjectType(type)) {
    if (typeof value !== 'object') {
      throw new GraphQLError(`Expected value for type ${type} to be an object, but is ${typeof value}.`);
    }

    const updated = Object.create(null);
    for (const field of type.fields.values()) {
      if (!field.type) {
        throw buildError(`Cannot compute default value for field ${field.name} of ${type} as the field type is undefined`);
      }
      const fieldValue = value[field.name];
      if (fieldValue === undefined) {
        if (field.defaultValue !== undefined) {
          updated[field.name] = applyDefaultValues(field.defaultValue, field.type);
        } else if (isNonNullType(field.type)) {
          throw new GraphQLError(`Field "${field.name}" of required type ${type} was not provided.`);
        }
      } else {
        updated[field.name] = applyDefaultValues(fieldValue, field.type);
      }
    }

    // Ensure every provided field is defined.
    for (const fieldName of Object.keys(value)) {
      if (!type.fields.has(fieldName)) {
        const suggestions = suggestionList(fieldName, [...type.fields.keys()]);
        throw new GraphQLError(`Field "${fieldName}" is not defined by type "${type}".` + didYouMean(suggestions));
      }
    }
    return updated;
  }
  return value;
}

export function withDefaultValues(value: any, argument: ArgumentDefinition<any>): any {
  if (!argument.type) {
    throw buildError(`Cannot compute default value for argument ${argument} as the type is undefined`);
  }
  if (value === undefined) {
    if (argument.defaultValue) {
      return applyDefaultValues(argument.defaultValue ?? null, argument.type);
    }
  }
  return applyDefaultValues(value, argument.type);
}
