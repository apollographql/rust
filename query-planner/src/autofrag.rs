use crate::builder::get_field_def_from_type;
use crate::consts::{MUTATION_TYPE_NAME, QUERY_TYPE_NAME};
use crate::context::QueryPlanningContext;
use graphql_parser::query::refs::{FragmentDefinitionRef, SelectionRef, SelectionSetRef};
use graphql_parser::query::{Operation, Selection};
use graphql_parser::schema::TypeDefinition;
use graphql_parser::Name;
use std::collections::HashMap;

pub(crate) fn auto_fragmentation<'q>(
    context: &'q QueryPlanningContext<'q>,
    selection_set: SelectionSetRef<'q>,
) -> (Vec<FragmentDefinitionRef<'q>>, SelectionSetRef<'q>) {
    let root_parent = if let Operation::Query = &context.operation.kind {
        context.names_to_types[QUERY_TYPE_NAME]
    } else {
        context.names_to_types[MUTATION_TYPE_NAME]
    };

    fn auto_frag_selection_set<'a, 'q>(
        context: &'q QueryPlanningContext<'q>,
        frags: &'a mut HashMap<&'q SelectionSetRef<'q>, FragmentDefinitionRef<'q>>,
        parent: &'q TypeDefinition<'q>,
        selection_set: SelectionSetRef<'q>,
    ) -> SelectionSetRef<'q> {
        unimplemented!()
    }

    fn auto_frag_selection<'a, 'q>(
        context: &'q QueryPlanningContext<'q>,
        frags: &'a mut HashMap<&'q SelectionSetRef<'q>, FragmentDefinitionRef<'q>>,
        parent: &'q TypeDefinition<'q>,
        selection: SelectionRef<'q>,
    ) -> SelectionRef<'q> {
        match selection {
            SelectionRef::Ref(sel) => match sel {
                Selection::Field(field) => {
                    let field_return_type = get_field_def_from_type(parent, field.name)
                        .field_type
                        .as_name();

                    if let Some(new_parent) = context.names_to_types.get(field_return_type) {
                        let new_field = field_ref!(
                            field,
                            auto_frag_selection_set(
                                context,
                                frags,
                                *new_parent,
                                SelectionSetRef::from(&field.selection_set)
                            )
                        );
                        SelectionRef::FieldRef(new_field)
                    } else {
                        SelectionRef::FieldRef(field_ref!(field))
                    }
                }
                Selection::InlineFragment(inline) => {
                    if let Some(tc) = inline.type_condition {
                        let new_parent = context.names_to_types[tc];
                        SelectionRef::InlineFragmentRef(inline_fragment_ref!(
                            inline,
                            auto_frag_selection_set(
                                context,
                                frags,
                                new_parent,
                                SelectionSetRef::from(&inline.selection_set)
                            )
                        ))
                    } else {
                        SelectionRef::InlineFragmentRef(inline_fragment_ref!(inline))
                    }
                }
                Selection::FragmentSpread(_) => {
                    unreachable!("Fragment spreads is only used at the end of query planning")
                }
            },
            SelectionRef::Field(field) => {
                let field_return_type = get_field_def_from_type(parent, field.name)
                    .field_type
                    .as_name();

                if let Some(new_parent) = context.names_to_types.get(field_return_type) {
                    let new_field = field_ref!(
                        field,
                        auto_frag_selection_set(
                            context,
                            frags,
                            *new_parent,
                            SelectionSetRef::from(&field.selection_set)
                        )
                    );
                    SelectionRef::FieldRef(new_field)
                } else {
                    SelectionRef::FieldRef(field_ref!(field))
                }
            }
            SelectionRef::FieldRef(field) => {
                let field_return_type = get_field_def_from_type(parent, field.name)
                    .field_type
                    .as_name();

                if let Some(new_parent) = context.names_to_types.get(field_return_type) {
                    let new_field = field_ref!(
                        field,
                        auto_frag_selection_set(context, frags, *new_parent, field.selection_set)
                    );
                    SelectionRef::FieldRef(new_field)
                } else {
                    SelectionRef::FieldRef(field)
                }
            }
            SelectionRef::InlineFragmentRef(inline) => {
                if let Some(tc) = inline.type_condition {
                    let new_parent = context.names_to_types[tc];
                    SelectionRef::InlineFragmentRef(inline_fragment_ref!(
                        inline,
                        auto_frag_selection_set(context, frags, new_parent, inline.selection_set)
                    ))
                } else {
                    SelectionRef::InlineFragmentRef(inline)
                }
            }
            SelectionRef::FragmentSpreadRef(_) => {
                unreachable!("Fragment spreads is only used at the end of query planning")
            }
        }
    }

    let mut frags: HashMap<&'q SelectionSetRef<'q>, FragmentDefinitionRef<'q>> = HashMap::new();
    let mut new_ss = SelectionSetRef {
        span: selection_set.span,
        items: vec![],
    };

    for sel in selection_set.items.into_iter() {
        let new_sel = auto_frag_selection(context, &mut frags, root_parent, sel);
        new_ss.items.push(new_sel)
    }

    (values!(frags), new_ss)
}

#[cfg(test)]
mod tests {
    use crate::autofrag::auto_fragmentation;
    use crate::context::QueryPlanningContext;
    use crate::federation::Federation;
    use crate::helpers::{build_possible_types, names_to_types, variable_name_to_def, Op};
    use graphql_parser::query::refs::SelectionSetRef;
    use graphql_parser::query::{Definition, Operation};
    use graphql_parser::{parse_query, parse_schema, DisplayMinified};
    use std::collections::HashMap;

    #[test]
    #[should_panic]
    fn test_auto_fragmentation() {
        let schema = "schema {
            query: Query
        }

        type Query {
            field: SomeField
        }

        interface IFace {
            x: Int
        }

        type IFaceImpl1 implements IFace { x: Int }
        type IFaceImpl2 implements IFace { x: Int }

        type SomeField {
          a: A
          b: B
          iface: IFace
        }

        type A {
          b: B
        }

        type B {
          f1: String
          f2: String
          f3: String
          f4: String
          f5: String
          f6: String
        }
        ";

        let query = "{
          field {
            a { b { f1 f2 f4 } }
            b { f1 f2 f4 }
            iface {
                ...on IFaceImpl1 { x }
                ...on IFaceImpl2 { x }
            }
          }
        }";

        let expected = parse_query(
            "
            fragment __QueryPlanFragment_1__ on B { f1 f2 f4 }
            {
                field {
                  a { b { ...__QueryPlanFragment_1__ } }
                  b { ...__QueryPlanFragment_1__ }
                }
            }
        ",
        )
        .unwrap()
        .minified();

        let schema = parse_schema(schema).unwrap();
        let query = parse_query(query).unwrap();
        let ss = letp!(Definition::SelectionSet(ref ss) = query.definitions[0] => ss);
        let operation = Op {
            selection_set: ss,
            kind: Operation::Query,
        };

        let types = names_to_types(&schema);
        let context = QueryPlanningContext {
            schema: &schema,
            operation,
            fragments: HashMap::new(),
            auto_fragmentization: true,
            possible_types: build_possible_types(&schema, &types),
            variable_name_to_def: variable_name_to_def(&query),
            federation: Federation::new(&schema),
            names_to_types: types,
        };
        let (frags, ssr) = auto_fragmentation(
            &context,
            SelectionSetRef::from(context.operation.selection_set),
        );
        assert_eq!(1, frags.len());
        let got = format!("{} {}", frags[0].minified(), ssr.minified());
        let new_query = parse_query(&got).unwrap().minified();
        assert_eq!(expected, new_query);
    }
}