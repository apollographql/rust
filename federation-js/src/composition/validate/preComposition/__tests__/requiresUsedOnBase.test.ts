import gql from 'graphql-tag';
import { requiresUsedOnBase as validateRequiresUsedOnBase } from '../';
import { graphqlErrorSerializer } from 'apollo-federation-integration-testsuite';
import { parse } from 'graphql';

expect.addSnapshotSerializer(graphqlErrorSerializer);

describe('requiresUsedOnBase', () => {
  it('does not warn when no requires directives are defined', () => {
    const serviceA = {
      typeDefs: gql`
        type Product @key(fields: "color { id value }") {
          sku: String!
          upc: String!
          color: Color!
        }

        type Color {
          id: ID!
          value: String!
        }
      `,
      name: 'serviceA',
    };

    const warnings = validateRequiresUsedOnBase(serviceA);
    expect(warnings).toEqual([]);
  });

  it('warns when there is a @requires field on a base type', () => {
    const serviceA = {
      typeDefs: parse(`
        type Product @key(fields: "sku") {
          sku: String!
          upc: String! @requires(fields: "sku")
          id: ID!
        }
      `),
      name: 'serviceA',
    };

    const warnings = validateRequiresUsedOnBase(serviceA);
    expect(warnings).toMatchInlineSnapshot(`
      Array [
        Object {
          "code": "REQUIRES_USED_ON_BASE",
          "locations": Array [
            Object {
              "column": 11,
              "line": 4,
            },
          ],
          "message": "[serviceA] Product.upc -> Found extraneous @requires directive. @requires cannot be used on base types.",
        },
      ]
    `);
  });
});
