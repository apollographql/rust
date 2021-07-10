import {
  ArgumentDefinition,
  Directive,
  DirectiveDefinition,
  EnumType,
  ExtendableElement,
  Extension,
  FieldDefinition,
  InputFieldDefinition,
  InputObjectType,
  InterfaceImplementation,
  InterfaceType,
  NamedType,
  ObjectType,
  ScalarType,
  Schema,
  SchemaDefinition,
  SchemaElement,
  UnionType
} from "./definitions";
import { valueToString } from "./values";

export type Options = {
  indentString: string;
  definitionsOrder: ('schema' | 'types' | 'directives')[],
  typeCompareFn?: (t1: NamedType, t2: NamedType) => number;
  directiveCompareFn?: (d1: DirectiveDefinition, d2: DirectiveDefinition) => number;
  mergeTypesAndExtensions: boolean
}

export const defaultOptions: Options = {
  indentString: "  ",
  definitionsOrder: ['schema', 'directives', 'types'],
  mergeTypesAndExtensions: false
}

function isDefinitionOrderValid(options: Options): boolean {
  return options.definitionsOrder.length === 3
    && options.definitionsOrder.indexOf('schema') >= 0
    && options.definitionsOrder.indexOf('types') >= 0
    && options.definitionsOrder.indexOf('directives') >= 0;
}

function validateOptions(options: Options) {
  if (!isDefinitionOrderValid(options)) {
    throw new Error(`'definitionsOrder' should be a 3-element array containing 'schema', 'types' and 'directives' in the desired order (got: [${options.definitionsOrder.join(', ')}])`);
  }
}

export function printSchema(schema: Schema, options: Options = defaultOptions): string {
  validateOptions(options);
  let directives = [...schema.directives()];
  if (options.directiveCompareFn) {
    directives = directives.sort(options.directiveCompareFn);
  }
  let types = [...schema.types()];
  if (options.typeCompareFn) {
    types = types.sort(options.typeCompareFn);
  }
  const definitions: string[][] = new Array(3);
  definitions[options.definitionsOrder.indexOf('schema')] = printSchemaDefinitionAndExtensions(schema.schemaDefinition, options);
  definitions[options.definitionsOrder.indexOf('directives')] = directives.map(directive => printDirectiveDefinition(directive));
  definitions[options.definitionsOrder.indexOf('types')] = types.flatMap(type => printTypeDefinitionAndExtensions(type, options));
  return definitions.flat().join('\n\n');
}

function definitionAndExtensions<T extends ExtendableElement>(element: {extensions(): ReadonlySet<Extension<T>>}, options: Options): (Extension<any> | null | undefined)[] {
  return options.mergeTypesAndExtensions ? [undefined] : [null, ...element.extensions()];
}

function printSchemaDefinitionAndExtensions(schemaDefinition: SchemaDefinition, options: Options): string[] {
  if (isSchemaOfCommonNames(schemaDefinition)) {
    return [];
  }
  return printDefinitionAndExtensions(schemaDefinition, options, printSchemaDefinitionOrExtension);
}

function printDefinitionAndExtensions<T extends {extensions(): ReadonlySet<Extension<any>>}>(
  t: T,
  options: Options,
  printer: (t: T, options: Options, extension?: Extension<any> | null) => string | undefined
): string[] {
  return definitionAndExtensions(t, options)
    .map(ext => printer(t, options, ext))
    .filter(v => v !== undefined) as string[];
}

function printIsExtension(extension?: Extension<any> | null): string {
  return extension ? 'extend ' : '';
}

function forExtension<T extends {ofExtension(): Extension<any> | undefined}>(ts: readonly T[], extension?: Extension<any> | null): readonly T[]  {
  if (extension === undefined) {
    return ts;
  }
  return ts.filter(r => (r.ofExtension() ?? null) === extension);
}

function printSchemaDefinitionOrExtension(
  schemaDefinition: SchemaDefinition,
  options: Options,
  extension?: Extension<SchemaDefinition> | null
): string | undefined {
  const roots = forExtension([...schemaDefinition.roots()],  extension);
  const directives = forExtension(schemaDefinition.appliedDirectives, extension);
  if (!roots.length && !directives.length) {
    return undefined;
  }
  const rootEntries = roots.map((rootType) => `${options.indentString}${rootType.rootKind}: ${rootType.type}`);
  return printDescription(schemaDefinition)
    + printIsExtension(extension)
    + 'schema'
    + printAppliedDirectives(directives)
    + ' {\n' + rootEntries.join('\n') + '\n}';
}

/**
 * GraphQL schema define root types for each type of operation. These types are
 * the same as any other type and can be named in any manner, however there is
 * a common naming convention:
 *
 *   schema {
 *     query: Query
 *     mutation: Mutation
 *   }
 *
 * When using this naming convention, the schema description can be omitted.
 */
function isSchemaOfCommonNames(schema: SchemaDefinition): boolean {
  return schema.appliedDirectives.length === 0 && !schema.description && [...schema.roots()].every(r => r.isDefaultRootName());
}

export function printTypeDefinitionAndExtensions(type: NamedType, options: Options): string[] {
  switch (type.kind) {
    case 'ScalarType': return printDefinitionAndExtensions(type, options, printScalarDefinitionOrExtension);
    case 'ObjectType': return printDefinitionAndExtensions(type, options, (t, options, ext) => printFieldBasedTypeDefinitionOrExtension('type', t, options, ext));
    case 'InterfaceType': return printDefinitionAndExtensions(type, options, (t, options, ext) => printFieldBasedTypeDefinitionOrExtension('interface', t, options, ext));
    case 'UnionType': return printDefinitionAndExtensions(type, options, printUnionDefinitionOrExtension);
    case 'EnumType': return printDefinitionAndExtensions(type, options, printEnumDefinitionOrExtension);
    case 'InputObjectType': return printDefinitionAndExtensions(type, options, printInputDefinitionOrExtension);
  }
}

export function printDirectiveDefinition(directive: DirectiveDefinition): string {
  const locations = directive.locations.join(' | ');
  return `${printDescription(directive)}directive @${directive}${printArgs([...directive.arguments()])}${directive.repeatable ? ' repeatable' : ''} on ${locations}`;
}

function printAppliedDirectives(appliedDirectives: readonly Directive<any>[]): string {
  return appliedDirectives.length == 0 ? "" : " " + appliedDirectives.map(d => d.toString()).join(" ");
}

function printDescription(
  element: SchemaElement<any>,
  indentation: string = '',
  firstInBlock: boolean = true
): string {
  if (!element.description) {
    return '';
  }

  const preferMultipleLines = element.description.length > 70;
  const blockString = printBlockString(element.description, '', preferMultipleLines);
  const prefix =
    indentation && !firstInBlock ? '\n' + indentation : indentation;

  return prefix + blockString.replace(/\n/g, '\n' + indentation) + '\n';
}

function printScalarDefinitionOrExtension(type: ScalarType, _?: Options, extension?: Extension<any> | null): string | undefined {
  const directives = forExtension(type.appliedDirectives, extension);
  if (extension && !directives.length) {
    return undefined;
  }
  return `${printDescription(type)}${printIsExtension(extension)}scalar ${type.name}${printAppliedDirectives(directives)}`
}

function printImplementedInterfaces(implementations: readonly InterfaceImplementation<any>[]): string {
  return implementations.length
    ? ' implements ' + implementations.map(i => i.interface.name).join(' & ')
    : '';
}

function printFieldBasedTypeDefinitionOrExtension(kind: string, type: ObjectType | InterfaceType, options: Options, extension?: Extension<any> | null): string | undefined {
  const directives = forExtension(type.appliedDirectives, extension);
  const interfaces = forExtension([...type.interfaceImplementations()], extension);
  const fields = forExtension([...type.fields.values()], extension);
  if (!directives.length && !interfaces.length && !fields.length) {
    return undefined;
  }
  return printDescription(type)
    + printIsExtension(extension)
    + kind + ' ' + type
    + printImplementedInterfaces(interfaces)
    + printAppliedDirectives(directives)
    + printFields(fields, options);
}

function printUnionDefinitionOrExtension(type: UnionType, _?: Options, extension?: Extension<any> | null): string | undefined {
  const directives = forExtension(type.appliedDirectives, extension);
  const members = forExtension([...type.members()], extension);
  if (!directives.length && !members.length) {
    return undefined;
  }
  const possibleTypes = members.length ? ' = ' + members.map(m => m.type).join(' | ') : '';
  return printDescription(type)
    + printIsExtension(extension)
    + 'union ' + type
    + printAppliedDirectives(directives)
    + possibleTypes;
}

function printEnumDefinitionOrExtension(type: EnumType, options: Options, extension?: Extension<any> | null): string | undefined {
  const directives = forExtension(type.appliedDirectives, extension);
  const values = forExtension(type.values, extension);
  if (!directives.length && !values.length) {
    return undefined;
  }
  const vals = values.map((v, i) => 
    printDescription(v, options.indentString, !i)
    + v
    + printAppliedDirectives(v.appliedDirectives));
  return printDescription(type)
    + printIsExtension(extension)
    + 'enum ' + type
    + printAppliedDirectives(directives)
    + printBlock(vals);
}

function printInputDefinitionOrExtension(type: InputObjectType, options: Options, extension?: Extension<any> | null): string | undefined {
  const directives = forExtension(type.appliedDirectives, extension);
  const fields = forExtension([...type.fields.values()], extension);
  if (!directives.length && !fields.length) {
    return undefined;
  }
  return printDescription(type)
    + printIsExtension(extension)
    + 'input ' + type
    + printAppliedDirectives(directives)
    + printFields(fields, options);
}

function printFields(fields: readonly (FieldDefinition<any> | InputFieldDefinition)[], options: Options): string {
  return printBlock(fields.map((f, i) =>
    printDescription(f, options.indentString, !i)
    + options.indentString 
    + printField(f, options) 
    + printAppliedDirectives(f.appliedDirectives)));
}

function printField(field: FieldDefinition<any> | InputFieldDefinition, options: Options): string {
  let args = field.kind == 'FieldDefinition' ? printArgs([...field.arguments.values()], options.indentString) : '';
  let defaultValue = field.kind == 'InputFieldDefinition' && field.defaultValue !== undefined
    ? ' = ' + valueToString(field.defaultValue)
    : '';
  return `${field.name}${args}: ${field.type}${defaultValue}`;
}

function printArgs(args: ArgumentDefinition<any>[], indentation = '') {
  if (args.length === 0) {
    return '';
  }

  // If every arg does not have a description, print them on one line.
  if (args.every(arg => !arg.description)) {
    return '(' + args.map(printArg).join(', ') + ')';
  }

  const formattedArgs = args
    .map((arg, i) => printDescription(arg, '  ' + indentation, !i) + '  ' + indentation + printArg(arg))
    .join('\n');
  return `(\n${formattedArgs}\n${indentation})`;
}

function printArg(arg: ArgumentDefinition<any>) {
  return `${arg}${printAppliedDirectives(arg.appliedDirectives)}`;
}

function printBlock(items: string[]): string {
  return items.length !== 0 ? ' {\n' + items.join('\n') + '\n}' : '';
}

/**
 * Print a block string in the indented block form by adding a leading and
 * trailing blank line. However, if a block string starts with whitespace and is
 * a single-line, adding a leading blank line would strip that whitespace.
 */
function printBlockString(
  value: string,
  indentation: string = '',
  preferMultipleLines: boolean = false,
): string {
  const isSingleLine = value.indexOf('\n') === -1;
  const hasLeadingSpace = value[0] === ' ' || value[0] === '\t';
  const hasTrailingQuote = value[value.length - 1] === '"';
  const hasTrailingSlash = value[value.length - 1] === '\\';
  const printAsMultipleLines =
    !isSingleLine ||
    hasTrailingQuote ||
    hasTrailingSlash ||
    preferMultipleLines;

  let result = '';
  // Format a multi-line block quote to account for leading space.
  if (printAsMultipleLines && !(isSingleLine && hasLeadingSpace)) {
    result += '\n' + indentation;
  }
  result += indentation ? value.replace(/\n/g, '\n' + indentation) : value;
  if (printAsMultipleLines) {
    result += '\n';
  }

  return '"""' + result.replace(/"""/g, '\\"""') + '"""';
}
