import { buildSchema } from 'graphql';
import { removeInaccessibleElements } from '../removeInaccessibleElements';

describe('removeInaccessibleElements', () => {
  it(`removes @inaccessible fields`, () => {
    let schema = buildSchema(`
      directive @core(feature: String!) repeatable on SCHEMA

      directive @inaccessible on FIELD_DEFINITION | OBJECT | INTERFACE | UNION

      schema
        @core(feature: "https://specs.apollo.dev/core/v0.1")
        @core(feature: "https://specs.apollo.dev/inaccessible/v0.1")
      {
        query: Query
      }

      type Query {
        someField: String
        privateField: String @inaccessible
      }
    `);

    schema = removeInaccessibleElements(schema);

    const queryType = schema.getQueryType()!;

    expect(queryType.getFields()['someField']).toBeDefined();
    expect(queryType.getFields()['privateField']).toBeUndefined();
  });

  it(`removes @inaccessible object types`, () => {
    let schema = buildSchema(`
      directive @core(feature: String!) repeatable on SCHEMA

      directive @inaccessible on FIELD_DEFINITION | OBJECT | INTERFACE | UNION

      schema
        @core(feature: "https://specs.apollo.dev/core/v0.1")
        @core(feature: "https://specs.apollo.dev/inaccessible/v0.1")
      {
        query: Query
      }

      type Query {
        fooField: Foo @inaccessible
      }

      type Foo @inaccessible {
        someField: String
      }

      union Bar = Foo
    `);

    schema = removeInaccessibleElements(schema);

    expect(schema.getType('Foo')).toBeUndefined();
  });

  it(`removes @inaccessible interface types`, () => {
    let schema = buildSchema(`
      directive @core(feature: String!) repeatable on SCHEMA

      directive @inaccessible on FIELD_DEFINITION | OBJECT | INTERFACE | UNION

      schema
        @core(feature: "https://specs.apollo.dev/core/v0.1")
        @core(feature: "https://specs.apollo.dev/inaccessible/v0.1")
      {
        query: Query
      }

      type Query {
        fooField: Foo @inaccessible
      }

      interface Foo @inaccessible {
        someField: String
      }

      type Bar implements Foo {
        someField: String
      }
    `);

    schema = removeInaccessibleElements(schema);

    expect(schema.getType('Foo')).toBeUndefined();
  });

  it(`removes @inaccessible union types`, () => {
    let schema = buildSchema(`
      directive @core(feature: String!) repeatable on SCHEMA

      directive @inaccessible on FIELD_DEFINITION | OBJECT | INTERFACE | UNION

      schema
        @core(feature: "https://specs.apollo.dev/core/v0.1")
        @core(feature: "https://specs.apollo.dev/inaccessible/v0.1")
      {
        query: Query
      }

      type Query {
        fooField: Foo @inaccessible
      }

      union Foo @inaccessible = Bar | Baz

      type Bar {
        someField: String
      }

      type Baz {
        anotherField: String
      }
    `);

    schema = removeInaccessibleElements(schema);

    expect(schema.getType('Foo')).toBeUndefined();
    expect(schema.getType('Bar')).toBeDefined();
    expect(schema.getType('Baz')).toBeDefined();
  });

  it(`throws when a field returning an @inaccessible type isn't marked @inaccessible itself`, () => {
    let schema = buildSchema(`
      directive @core(feature: String!) repeatable on SCHEMA

      directive @inaccessible on FIELD_DEFINITION | OBJECT | INTERFACE | UNION

      schema
        @core(feature: "https://specs.apollo.dev/core/v0.1")
        @core(feature: "https://specs.apollo.dev/inaccessible/v0.1")
      {
        query: Query
      }

      type Query {
        fooField: Foo
      }

      type Foo @inaccessible {
        someField: String
      }

      union Bar = Foo
    `);

    expect(() => {
      removeInaccessibleElements(schema);
    }).toThrow(
      `Field Query.fooField returns an @inaccessible type without being marked @inaccessible itself`,
    );
  });
});
