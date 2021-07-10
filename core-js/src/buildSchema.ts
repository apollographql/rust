import {
  DefinitionNode,
  DirectiveDefinitionNode,
  DirectiveLocationEnum,
  DirectiveNode,
  DocumentNode,
  FieldDefinitionNode,
  GraphQLError,
  InputValueDefinitionNode,
  parse,
  SchemaDefinitionNode,
  Source,
  TypeNode,
  valueFromASTUntyped,
  ValueNode,
  NamedTypeNode,
  ArgumentNode,
  StringValueNode,
  ASTNode,
  SchemaExtensionNode
} from "graphql";
import { Maybe } from "graphql/jsutils/Maybe";
import {
  BuiltIns,
  Schema,
  graphQLBuiltIns,
  newNamedType,
  NamedTypeKind,
  NamedType,
  SchemaDefinition,
  SchemaElement,
  ObjectType,
  InterfaceType,
  FieldDefinition,
  Type,
  ListType,
  OutputType,
  isOutputType,
  isInputType,
  InputType,
  NonNullType,
  ArgumentDefinition,
  InputFieldDefinition,
  DirectiveDefinition,
  UnionType,
  InputObjectType,
  EnumType,
  Extension
} from "./definitions";

function buildValue(value?: ValueNode): any {
  // TODO: Should we rewrite a version of valueFromAST instead of using valueFromASTUntyped? Afaict, what we're missing out on is
  // 1) coercions, which concretely, means:
  //   - for enums, we get strings
  //   - for int, we don't get the validation that it should be a 32bit value.
  //   - for ID, which accepts strings and int, we don't get int converted to string.
  //   - for floats, we get either int or float, we don't get int converted to float.
  //   - we don't get any custom coercion (but neither is buildSchema in graphQL-js anyway).
  // 2) type validation. 
  return value ? valueFromASTUntyped(value) : undefined;
}

export function buildSchema(source: string | Source, builtIns: BuiltIns = graphQLBuiltIns): Schema {
  return buildSchemaFromAST(parse(source), builtIns);
}

export function buildSchemaFromAST(documentNode: DocumentNode, builtIns: BuiltIns = graphQLBuiltIns): Schema {
  const schema = new Schema(builtIns);
  // We do a first pass to add all empty types and directives definition. This ensure any reference on one of
  // those can be resolved in the 2nd pass, regardless of the order of the definitions in the AST.
  buildNamedTypeAndDirectivesShallow(documentNode, schema);
  for (const definitionNode of documentNode.definitions) {
    switch (definitionNode.kind) {
      case 'OperationDefinition':
      case 'FragmentDefinition':
        throw new GraphQLError("Invalid executable definition found while building schema", definitionNode);
      case 'SchemaDefinition':
        buildSchemaDefinitionInner(definitionNode, schema.schemaDefinition);
        break;
      case 'SchemaExtension':
        buildSchemaDefinitionInner(
          definitionNode,
          schema.schemaDefinition, 
          schema.schemaDefinition.newExtension());
        break;
      case 'ScalarTypeDefinition':
      case 'ObjectTypeDefinition':
      case 'InterfaceTypeDefinition':
      case 'UnionTypeDefinition':
      case 'EnumTypeDefinition':
      case 'InputObjectTypeDefinition':
        buildNamedTypeInner(definitionNode, schema.type(definitionNode.name.value)!);
        break;
      case 'ScalarTypeExtension':
      case 'ObjectTypeExtension':
      case 'InterfaceTypeExtension':
      case 'UnionTypeExtension':
      case 'EnumTypeExtension':
      case 'InputObjectTypeExtension':
        const toExtend = schema.type(definitionNode.name.value)!;
        const extension = toExtend.newExtension();
        extension.sourceAST = definitionNode;
        buildNamedTypeInner(definitionNode, toExtend, extension);
        break;
      case 'DirectiveDefinition':
        buildDirectiveDefinitionInner(definitionNode, schema.directive(definitionNode.name.value)!);
        break;
    }
  }
  return schema;
}

function buildNamedTypeAndDirectivesShallow(documentNode: DocumentNode, schema: Schema) {
  for (const definitionNode of documentNode.definitions) {
    switch (definitionNode.kind) {
      case 'ScalarTypeDefinition':
      case 'ObjectTypeDefinition':
      case 'InterfaceTypeDefinition':
      case 'UnionTypeDefinition':
      case 'EnumTypeDefinition':
      case 'InputObjectTypeDefinition':
      case 'ScalarTypeExtension':
      case 'ObjectTypeExtension':
      case 'InterfaceTypeExtension':
      case 'UnionTypeExtension':
      case 'EnumTypeExtension':
      case 'InputObjectTypeExtension':
        // Note that because of extensions, this may be called multiple times for the same type.
        if (!schema.type(definitionNode.name.value)) {
          schema.addType(newNamedType(withoutTrailingDefinition(definitionNode.kind), definitionNode.name.value));
        }
        break;
      case 'DirectiveDefinition':
        schema.addDirectiveDefinition(definitionNode.name.value);
        break;
    }
  }
}

type NodeWithDirectives = {directives?: ReadonlyArray<DirectiveNode>};
type NodeWithDescription = {description?: Maybe<StringValueNode>};
type NodeWithArguments = {arguments?: ReadonlyArray<ArgumentNode>};

function withoutTrailingDefinition(str: string): NamedTypeKind {
  const endString = str.endsWith('Definition') ? 'Definition' : 'Extension';
  return str.slice(0, str.length - endString.length) as NamedTypeKind;
}

function getReferencedType(node: NamedTypeNode, schema: Schema): NamedType {
  const type = schema.type(node.name.value);
  if (!type) {
    throw new GraphQLError(`Unknown type ${node.name.value}`, node);
  }
  return type;
}

function withNodeAttachedToError(operation: () => void, node: ASTNode) {
  try {
    operation();
  } catch (e) {
    if (e instanceof GraphQLError) {
      const allNodes: ASTNode | ASTNode[] = e.nodes ? [node, ...e.nodes] : node;
      throw new GraphQLError(
        e.message,
        allNodes,
        e.source,
        e.positions,
        e.path,
        e,
        e.extensions
      );
    } else {
      throw e;
    }
  }
}

function buildSchemaDefinitionInner(
  schemaNode: SchemaDefinitionNode | SchemaExtensionNode,
  schemaDefinition: SchemaDefinition,
  extension?: Extension<SchemaDefinition>
) {
  for (const opTypeNode of schemaNode.operationTypes ?? []) {
    withNodeAttachedToError(
      () => schemaDefinition.setRoot(opTypeNode.operation, opTypeNode.type.name.value).setOfExtension(extension),
      opTypeNode);
  }
  schemaDefinition.sourceAST = schemaNode;
  schemaDefinition.description = 'description' in schemaNode ? schemaNode.description?.value : undefined;
  buildAppliedDirectives(schemaNode, schemaDefinition, extension);
}

function buildAppliedDirectives(
  elementNode: NodeWithDirectives,
  element: SchemaElement<any>,
  extension?: Extension<any>
) {
  for (const directive of elementNode.directives ?? []) {
    withNodeAttachedToError(() => {
      const d = element.applyDirective(directive.name.value, buildArgs(directive));
      d.setOfExtension(extension);
      d.sourceAST = directive;
    }, directive);
  }
}

function buildArgs(argumentsNode: NodeWithArguments): Record<string, any> {
  const args = Object.create(null);
  for (const argNode of argumentsNode.arguments ?? []) {
    args[argNode.name.value] = buildValue(argNode.value);
  }
  return args;
}

function buildNamedTypeInner(
  definitionNode: DefinitionNode & NodeWithDirectives & NodeWithDescription,
  type: NamedType,
  extension?: Extension<any>
) {
  switch (definitionNode.kind) {
    case 'ObjectTypeDefinition':
    case 'ObjectTypeExtension':
    case 'InterfaceTypeDefinition':
    case 'InterfaceTypeExtension':
      const fieldBasedType = type as ObjectType | InterfaceType;
      for (const fieldNode of definitionNode.fields ?? []) {
        const field = fieldBasedType.addField(fieldNode.name.value);
        field.setOfExtension(extension);
        buildFieldDefinitionInner(fieldNode, field);
      }
      for (const itfNode of definitionNode.interfaces ?? []) {
        withNodeAttachedToError(
          () => fieldBasedType.addImplementedInterface(itfNode.name.value).setOfExtension(extension),
          itfNode);
      }
      break;
    case 'UnionTypeDefinition':
    case 'UnionTypeExtension':
      const unionType = type as UnionType;
      for (const namedType of definitionNode.types ?? []) {
        withNodeAttachedToError(
          () => unionType.addType(namedType.name.value).setOfExtension(extension),
          namedType);
      }
      break;
    case 'EnumTypeDefinition':
    case 'EnumTypeExtension':
      const enumType = type as EnumType;
      for (const enumVal of definitionNode.values ?? []) {
        enumType.addValue(enumVal.name.value).setOfExtension(extension);
      }
      break;
    case 'InputObjectTypeDefinition':
    case 'InputObjectTypeExtension':
      const inputObjectType = type as InputObjectType;
      for (const fieldNode of definitionNode.fields ?? []) {
        const field = inputObjectType.addField(fieldNode.name.value);
        field.setOfExtension(extension);
        buildInputFieldDefinitionInner(fieldNode, field);
      }
      break;
  }
  buildAppliedDirectives(definitionNode, type, extension);
  type.description = definitionNode.description?.value;
  type.sourceAST = definitionNode;
}

function buildFieldDefinitionInner(fieldNode: FieldDefinitionNode, field: FieldDefinition<any>) {
  const type = buildWrapperTypeOrTypeRef(fieldNode.type, field.schema()!);
  field.type = ensureOutputType(type, fieldNode.type);
  for (const inputValueDef of fieldNode.arguments ?? []) {
    buildArgumentDefinitionInner(inputValueDef, field.addArgument(inputValueDef.name.value));
  }
  buildAppliedDirectives(fieldNode, field);
  field.description = fieldNode.description?.value;
  field.sourceAST = fieldNode;
}

function ensureOutputType(type: Type, node: TypeNode): OutputType {
  if (isOutputType(type)) {
    return type;
  } else {
    throw new GraphQLError(`Expected ${type} to be an output type`, node);
  }
}

function ensureInputType(type: Type, node: TypeNode): InputType {
  if (isInputType(type)) {
    return type;
  } else {
    throw new GraphQLError(`Expected ${type} to be an input type`, node);
  }
}

function buildWrapperTypeOrTypeRef(typeNode: TypeNode, schema: Schema): Type {
  switch (typeNode.kind) {
    case 'ListType':
      return new ListType(buildWrapperTypeOrTypeRef(typeNode.type, schema));
    case 'NonNullType':
      const wrapped = buildWrapperTypeOrTypeRef(typeNode.type, schema);
      if (wrapped.kind == 'NonNullType') {
        throw new GraphQLError(`Cannot apply the non-null operator (!) twice to the same type`, typeNode);
      }
      return new NonNullType(wrapped);
    default:
      return getReferencedType(typeNode, schema);
  }
}

function buildArgumentDefinitionInner(inputNode: InputValueDefinitionNode, arg: ArgumentDefinition<any>) {
  const type = buildWrapperTypeOrTypeRef(inputNode.type, arg.schema()!);
  arg.type = ensureInputType(type, inputNode.type);
  arg.defaultValue = buildValue(inputNode.defaultValue);
  buildAppliedDirectives(inputNode, arg);
  arg.description = inputNode.description?.value;
  arg.sourceAST = inputNode;
}

function buildInputFieldDefinitionInner(fieldNode: InputValueDefinitionNode, field: InputFieldDefinition) {
  const type = buildWrapperTypeOrTypeRef(fieldNode.type, field.schema()!);
  field.type = ensureInputType(type, fieldNode.type);
  field.defaultValue = buildValue(fieldNode.defaultValue);
  buildAppliedDirectives(fieldNode, field);
  field.description = fieldNode.description?.value;
  field.sourceAST = fieldNode;
}

function buildDirectiveDefinitionInner(directiveNode: DirectiveDefinitionNode, directive: DirectiveDefinition) {
  for (const inputValueDef of directiveNode.arguments ?? []) {
    buildArgumentDefinitionInner(inputValueDef, directive.addArgument(inputValueDef.name.value));
  }
  directive.repeatable = directiveNode.repeatable;
  const locations = directiveNode.locations.map(({ value }) => value as DirectiveLocationEnum);
  directive.addLocations(...locations);
  directive.description = directiveNode.description?.value;
  directive.sourceAST = directiveNode;
}
