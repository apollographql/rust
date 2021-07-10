import {
  Schema,
  ObjectType,
  Type,
  DirectiveDefinition,
  InterfaceType,
  EnumType,
  SchemaElement,
  UnionType
} from '../../dist/definitions';
import { defaultOptions, printSchema } from '../../dist/print';
import { buildSchema } from '../../dist/buildSchema';
import { federationBuiltIns } from '../../dist/federation';

function expectObjectType(type?: Type): asserts type is ObjectType {
  expect(type).toBeDefined();
  expect(type!.kind).toBe('ObjectType');
}

function expectInterfaceType(type?: Type): asserts type is InterfaceType {
  expect(type).toBeDefined();
  expect(type!.kind).toBe('InterfaceType');
}

function expectUnionType(type?: Type): asserts type is UnionType {
  expect(type).toBeDefined();
  expect(type!.kind).toBe('UnionType');
}

function expectEnumType(type?: Type): asserts type is EnumType {
  expect(type).toBeDefined();
  expect(type!.kind).toBe('EnumType');
}

declare global {
  namespace jest {
    interface Matchers<R> {
      toHaveField(name: string, type?: Type): R;
      toHaveDirective<TArgs extends {[key: string]: any}>(directive: DirectiveDefinition<TArgs>, args?: TArgs): R;
      toMatchString(actual: string): R;
    }
  }
}

function deIndent(str: string): string {
  // Strip leading \n
  str = str.slice(str.search(/[^\n]/));
  // Strip trailing \n or space
  while (str.charAt(str.length - 1) === '\n' || str.charAt(str.length - 1) === ' ') {
    str = str.slice(0, str.length - 1);
  }
  const indent = str.search(/[^ ]/);
  return str
    .split('\n')
    .map(line => line.slice(indent))
    .join('\n');
}

expect.extend({
  toHaveField(parentType: ObjectType | InterfaceType, name: string, type?: Type) {
    const field = parentType.field(name);
    if (!field) {
      return {
        message: () => `Cannot find field '${name}' in Object Type ${parentType} with fields [${[...parentType.fields.keys()]}]`,
        pass: false
      };
    }
    if (field.name != name) {
      return {
        message: () => `Type ${parentType} has a field linked to name ${name} but that field name is actually ${field.name}`,
        pass: false
      };
    }
    if (type && field.type != type) {
      return {
        message: () => `Expected field ${parentType}.${name} to have type ${type} but got type ${field.type}`,
        pass: false
      };
    }
    return {
      message: () => `Expected ${parentType} not to have field ${name} but it does (${field})`,
      pass: true
    }
  },

  toHaveDirective(element: SchemaElement<any>, definition: DirectiveDefinition, args?: Record<string, any>) {
    const directives = element.appliedDirectivesOf(definition);
    if (directives.length == 0) {
      return {
        message: () => `Cannot find directive @${definition} applied to element ${element} (whose applied directives are [${element.appliedDirectives.join(', ')}]`,
        pass: false
      };
    }
    if (!args) {
      return {
        message: () => `Expected directive @${definition} to not be applied to ${element} but it is`,
        pass: true
      };
    }

    for (const directive of directives) {
      if (directive.matchArguments(args)) {
        return {
          // Not 100% certain that message is correct but I don't think it's going to be used ...
          message: () => `Expected directive ${directive.name} applied to ${element} to have arguments ${JSON.stringify(args)} but got ${JSON.stringify(directive.arguments)}`,
          pass: true
        };
      }
    }
    return {
      message: () => `Element ${element} has application of directive @${definition} but not with the requested arguments. Got applications: [${directives.join(', ')}]`,
      pass: false
    }
  },

  toMatchString(expected: string, received: string) {
    received = deIndent(received);
    const pass = this.equals(expected, received);
    const message = pass
      ? () => this.utils.matcherHint('toMatchString', undefined, undefined)
          + '\n\n'
          + `Expected: not ${this.printExpected(expected)}`
      : () => {
        return (
          this.utils.matcherHint('toMatchString', undefined, undefined,)
          + '\n\n'
          + this.utils.printDiffOrStringify(expected, received, 'Expected', 'Received', true));
      };
    return {received, expected, message, name: 'toMatchString', pass};
  }
});

test('building a simple schema programatically', () => {
  const schema = new Schema(federationBuiltIns);
  const queryType = schema.schemaDefinition.setRoot('query', schema.addType(new ObjectType('Query'))).type;
  const typeA = schema.addType(new ObjectType('A'));
  const inaccessible = federationBuiltIns.inaccessibleDirective(schema);
  const key = federationBuiltIns.keyDirective(schema);

  queryType.addField('a', typeA);
  typeA.addField('q', queryType);
  typeA.applyDirective(inaccessible);
  typeA.applyDirective(key, { fields: 'a'});

  expect(queryType).toBe(schema.schemaDefinition.root('query')!.type);
  expect(queryType).toHaveField('a', typeA);
  expect(typeA).toHaveField('q', queryType);
  expect(typeA).toHaveDirective(inaccessible);
  expect(typeA).toHaveDirective(key, { fields: 'a'});
});


test('parse schema and modify', () => {
  const sdl = `
    schema {
      query: MyQuery
    }

    type A {
      f1(x: Int @inaccessible): String
      f2: String @inaccessible
    }

    type MyQuery {
      a: A
      b: Int
    }`;
  const schema = buildSchema(sdl, federationBuiltIns);

  const queryType = schema.type('MyQuery')!;
  const typeA = schema.type('A')!;
  expectObjectType(queryType);
  expectObjectType(typeA);
  expect(schema.schemaDefinition.root('query')!.type).toBe(queryType);
  expect(queryType).toHaveField('a', typeA);
  const f2 = typeA.field('f2');
  expect(f2).toHaveDirective(federationBuiltIns.inaccessibleDirective(schema));
  expect(printSchema(schema)).toMatchString(sdl);

  expect(typeA).toHaveField('f1');
  typeA.field('f1')!.remove();
  expect(typeA).not.toHaveField('f1');
});

test('removal of all directives of a schema', () => {
  const schema = buildSchema(`
    schema @foo {
      query: Query
    }

    type Query {
      a(id: String @bar): A @inaccessible
    }

    type A {
      a1: String @foo
      a2: [Int]
    }

    type B @foo {
      b: String @bar
    }

    union U @foobar = A | B

    directive @foo on SCHEMA | FIELD_DEFINITION
    directive @foobar on UNION
    directive @bar on ARGUMENT_DEFINITION
  `, federationBuiltIns);

  for (const element of schema.allSchemaElement()) {
    element.appliedDirectives.forEach(d => d.remove());
  }

  expect(printSchema(schema)).toMatchString(`
    directive @foo on SCHEMA | FIELD_DEFINITION

    directive @foobar on UNION

    directive @bar on ARGUMENT_DEFINITION

    type Query {
      a(id: String): A
    }

    type A {
      a1: String
      a2: [Int]
    }

    type B {
      b: String
    }

    union U = A | B`);
});

test('removal of all inacessible elements of a schema', () => {
  const schema = buildSchema(`
    schema @foo {
      query: Query
    }

    type Query {
      a(id: String @bar, arg: Int @inaccessible): A
    }

    type A {
      a1: String @inaccessible
      a2: [Int]
    }

    type B @inaccessible {
      b: String @bar
    }

    union U @inaccessible = A | B

    directive @foo on SCHEMA | FIELD_DEFINITION
    directive @bar on ARGUMENT_DEFINITION
  `, federationBuiltIns);

  for (const element of schema.allNamedSchemaElement()) {
    if (element.appliedDirectivesOf(federationBuiltIns.inaccessibleDirective(schema)).length > 0) {
      element.remove();
    }
  }

  expect(printSchema(schema)).toMatchString(`
    schema @foo {
      query: Query
    }

    directive @foo on SCHEMA | FIELD_DEFINITION

    directive @bar on ARGUMENT_DEFINITION

    type Query {
      a(id: String @bar): A
    }

    type A {
      a2: [Int]
    }
  `);
});

test('handling of interfaces', () => {
  const schema = buildSchema(`
    type Query {
      bestIs: [I!]!
    }

    interface B {
      a: Int
    }

    interface I implements B {
      a: Int
      b: String
    }

    type T1 implements B & I {
      a: Int
      b: String
      c: Int
    }

    type T2 implements B & I {
      a: Int
      b: String
      c: String
    }
  `);

  const b = schema.type('B');
  const i = schema.type('I');
  const t1 = schema.type('T1');
  const t2 = schema.type('T2');
  expectInterfaceType(b);
  expectInterfaceType(i);
  expectObjectType(t1);
  expectObjectType(t2);

  for (const t of [b, i, t1, t2]) {
    expect(t).toHaveField('a', schema.intType());
  }
  for (const t of [i, t1, t2]) {
    expect(t).toHaveField('b', schema.stringType());
  }
  expect(t1).toHaveField('c', schema.intType());
  expect(t2).toHaveField('c', schema.stringType());

  expect(i.implementsInterface(b.name)).toBeTruthy();
  expect(t1.implementsInterface(b.name)).toBeTruthy();
  expect(t1.implementsInterface(i.name)).toBeTruthy();
  expect(t2.implementsInterface(b.name)).toBeTruthy();
  expect(t2.implementsInterface(i.name)).toBeTruthy();

  expect(b.allImplementations()).toEqual([i, t1, t2]);
  expect(i.allImplementations()).toEqual([t1, t2]);

  for (const itf of [b, i]) {
    expect(itf.possibleRuntimeTypes()).toEqual([t1, t2]);
  }

  b.remove();

  expect(printSchema(schema)).toMatchString(`
    type Query {
      bestIs: [I!]!
    }

    interface I {
      a: Int
      b: String
    }

    type T1 implements I {
      a: Int
      b: String
      c: Int
    }

    type T2 implements I {
      a: Int
      b: String
      c: String
    }`);
});

test('handling of enums', () => {
  const schema = buildSchema(`
    type Query {
      a: A
    }

    enum E {
      V1
      V2
    }

    type A {
      a: Int
      e: E
    }
  `);

  const a = schema.type('A');
  const e = schema.type('E');
  expectObjectType(a);
  expectEnumType(e);

  expect(a).toHaveField('e', e);
  const v1 = e.value('V1');
  const v2 = e.value('V2');
  expect(v1).toBeDefined();
  expect(v2).toBeDefined();
  expect(e.values).toEqual([v1, v2]);
});

test('handling of descriptions', () => {
  const sdl = `
    """A super schema full of great queries"""
    schema {
      query: ASetOfQueries
    }

    """Marks field that are deemed more important than others"""
    directive @Important(
      """The reason for the importance of this field"""
      why: String = "because!"
    ) on FIELD_DEFINITION

    """The actual queries of the schema"""
    type ASetOfQueries {
      """Returns a set of products"""
      bestProducts: [Product!]!
    
      """Finds a product by ID"""
      product(
        """The ID identifying the product"""
        id: ID!
      ): Product
    }

    """A product that is also a book"""
    type Book implements Product {
      id: ID!
      description: String!
    
      """
      Number of pages in the book. Good so the customer knows its buying a 1000 page book for instance
      """
      pages: Int @Important
    }

    type DVD implements Product {
      id: ID!
      description: String!

      """The film author"""
      author: String @Important(why: "it's good to give credit!")
    }

    """Common interface to all our products"""
    interface Product {
      """Identifies the product"""
      id: ID!

      """
      Something that explains what the product is. This can just be the title of the product, but this can be more than that if we want to. But it should be useful you know, otherwise our customer won't buy it.
      """
      description: String!
    }`;
  const schema = buildSchema(sdl);

  // Checking we get back the schema through printing it is mostly good enough, but let's just
  // make sure long descriptions don't get annoying formatting newlines for instance when acessed on the
  // schema directly.
  const longComment = "Something that explains what the product is. This can just be the title of the product, but this can be more than that if we want to. But it should be useful you know, otherwise our customer won't buy it.";
  const product = schema.type('Product');
  expectInterfaceType(product);
  expect(product.field('description')!.description).toBe(longComment);

  expect(printSchema(schema)).toMatchString(sdl);
});

test('handling of extensions', () => {
  const sdl = `
    extend schema {
      query: AType
    }

    interface AInterface {
      i1: Int
    }

    extend interface AInterface @deprecated

    scalar AScalar

    extend scalar AScalar @deprecated

    extend type AType {
      t1: Int
      t2: String
    }

    type AType2 {
      t1: String
    }

    type AType3 {
      t2: Int
    }

    union AUnion = AType | AType2

    extend union AUnion = AType3
  `;

  const schema = buildSchema(sdl);
  expect(printSchema(schema)).toMatchString(sdl);

  const atype = schema.type('AType');
  expectObjectType(atype);
  expect(atype).toHaveField('t1', schema.intType());
  expect(atype).toHaveField('t2', schema.stringType());

  const aunion = schema.type('AUnion');
  expectUnionType(aunion);
  expect([...aunion.types()].map(t => t.name)).toEqual(['AType', 'AType2', 'AType3']);

  expect(printSchema(schema, { ...defaultOptions, mergeTypesAndExtensions: true })).toMatchString(`
    schema {
      query: AType
    }

    interface AInterface @deprecated {
      i1: Int
    }

    scalar AScalar @deprecated

    type AType {
      t1: Int
      t2: String
    }

    type AType2 {
      t1: String
    }

    type AType3 {
      t2: Int
    }

    union AUnion = AType | AType2 | AType3
  `);
});

test('default arguments for directives', () => {
  const sdl = `
    directive @Example(inputObject: ExampleInputObject! = {}) on FIELD_DEFINITION

    type Query {
      v1: Int @Example
      v2: Int @Example(inputObject: {})
      v3: Int @Example(inputObject: {number: 3})
    }

    input ExampleInputObject {
      number: Int! = 3
    }
  `;

  const schema = buildSchema(sdl);
  expect(printSchema(schema)).toMatchString(sdl);

  const query = schema.schemaDefinition.root('query')!.type;
  const exampleDirective = schema.directive('Example')!;
  expect(query).toHaveField('v1');
  expect(query).toHaveField('v2');
  expect(query).toHaveField('v3');
  const v1 = query.field('v1')!;
  const v2 = query.field('v2')!;
  const v3 = query.field('v3')!;

  const d1 = v1.appliedDirectivesOf(exampleDirective)[0];
  const d2 = v2.appliedDirectivesOf(exampleDirective)[0];
  const d3 = v3.appliedDirectivesOf(exampleDirective)[0];

  expect(d1.arguments()).toEqual({});
  expect(d2.arguments()).toEqual({ inputObject: {}});
  expect(d3.arguments()).toEqual({ inputObject: { number: 3 }});

  expect(d1.arguments(true)).toEqual({ inputObject: { number: 3 }});
  expect(d2.arguments(true)).toEqual({ inputObject: { number: 3 }});
  expect(d3.arguments(true)).toEqual({ inputObject: { number: 3 }});
});
