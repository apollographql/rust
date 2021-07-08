import fs from 'fs';
import {
  GraphQLSchema,
  parse,
} from 'graphql';
import path from 'path';
import { buildComposedSchema, toAPISchema } from '..';

describe('toAPISchema', () => {
  let schema: GraphQLSchema;

  beforeAll(() => {
    const schemaPath = path.join(
      __dirname,
      '../../__tests__/features/basic/',
      'supergraphSdl.graphql',
    );
    const supergraphSdl = fs.readFileSync(schemaPath, 'utf8');

    schema = toAPISchema(buildComposedSchema(parse(supergraphSdl)));
  });

  it(`doesn't include core directives`, () => {
    const directiveNames = schema
      .getDirectives()
      .map((directive) => directive.name);

    expect(directiveNames).toEqual(expect.not.arrayContaining(['core']));
  });

  it(`doesn't include join directives`, () => {
    const directiveNames = schema
      .getDirectives()
      .map((directive) => directive.name);

    expect(directiveNames).toEqual(
      expect.not.arrayContaining([
        'join__graph',
        'join__type',
        'join__owner',
        'join__field',
      ]),
    );
  });

  it(`doesn't include join types`, () => {
    const typeNames = Object.keys(schema.getTypeMap());

    expect(typeNames).toEqual(
      expect.not.arrayContaining(['join__FieldSet', 'join__Graph']),
    );
  });

  it(`does pass through other custom directives`, () => {
    expect(schema.getDirectives()).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ name: 'transform' }),
        expect.objectContaining({ name: 'stream' }),
      ]),
    );
  });
});
