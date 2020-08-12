use crate::federation::get_federation_metadata;
use crate::helpers::Op;
use crate::model::ResponsePathElement;
use crate::visitors::VariableUsagesMap;
use graphql_parser::query::refs::{FieldRef, SelectionSetRef};
use graphql_parser::query::*;
use graphql_parser::schema::TypeDefinition;
use graphql_parser::{schema, Name};
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlanningContext<'q> {
    pub schema: &'q schema::Document<'q>,
    pub operation: Op<'q>,
    pub fragments: HashMap<&'q str, &'q FragmentDefinition<'q>>,
    pub possible_types: HashMap<&'q str, Vec<&'q schema::ObjectType<'q>>>,
    pub names_to_types: HashMap<&'q str, &'q TypeDefinition<'q>>,
    pub variable_name_to_def: HashMap<&'q str, &'q VariableDefinition<'q>>,
    pub auto_fragmentization: bool,
}

impl<'q> QueryPlanningContext<'q> {
    pub fn new_scope(
        &self,
        td: &'q TypeDefinition<'q>,
        enclosing_scope: Option<Rc<Scope<'q>>>,
    ) -> Rc<Scope<'q>> {
        let possible_types: Vec<&'q schema::ObjectType<'q>> = self
            .get_possible_types(td)
            .iter()
            .copied()
            .filter(|t| {
                enclosing_scope
                    .as_ref()
                    .map(|enclosing_scope| enclosing_scope.possible_types.contains(t))
                    .unwrap_or(true)
            })
            .collect();

        Rc::new(Scope {
            parent_type: td,
            possible_types,
            enclosing_scope,
        })
    }

    pub fn get_type(&self, type_name: &str) -> &TypeDefinition {
        self.names_to_types[type_name]
    }

    fn get_possible_types(&self, td: &'q TypeDefinition<'q>) -> &Vec<&'q schema::ObjectType<'q>> {
        &self.possible_types[td.name().unwrap()]
    }

    pub fn get_variable_usages(
        &self,
        selection_set: &SelectionSetRef,
        fragments: &[&'q FragmentDefinition<'q>],
    ) -> (Vec<String>, Vec<&VariableDefinition>) {
        let mut v = selection_set
            .map(VariableUsagesMap::new(&self.variable_name_to_def))
            .output
            .unwrap();

        v.extend(fragments.iter().flat_map(|fd| {
            fd.selection_set
                .map(VariableUsagesMap::new(&self.variable_name_to_def))
                .output
                .unwrap()
        }));

        v.into_iter().unzip()
    }

    pub fn type_def_for_object(&self, obj: &schema::ObjectType) -> &schema::TypeDefinition {
        self.names_to_types[obj.name]
    }

    pub fn get_provided_fields<'a>(
        &self,
        field_def: &'q schema::Field<'q>,
        service_name: &'a str,
    ) -> FieldSet
    where
        'q: 'a,
    {
        unimplemented!()
    }

    pub fn get_owning_service(
        &self,
        parent_type: &TypeDefinition,
        field_def: &schema::Field,
    ) -> String {
        // panic if we can't find one.
        unimplemented!()
    }

    // TODO(ran) FIXME: we may be able to change this return type to &str
    pub fn get_base_service(&self, parent_type: &TypeDefinition) -> String {
        get_federation_metadata(parent_type)
            .unwrap_or_else(|| {
                panic!(
                    "There is no federation metadata for {}",
                    parent_type.name().unwrap()
                )
            })
            .service_name
            .to_string()
    }

    pub fn get_key_fields(&self, parent_type: &TypeDefinition, service_name: &str) -> FieldSet {
        // panic if we can't find one.
        unimplemented!()
    }

    pub fn get_required_fields(
        &self,
        parent_type: &TypeDefinition,
        field_def: &schema::Field,
        owning_service: &str,
    ) -> FieldSet {
        // panic if we can't find one.
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope<'q> {
    pub parent_type: &'q TypeDefinition<'q>,
    pub possible_types: Vec<&'q schema::ObjectType<'q>>,
    pub enclosing_scope: Option<Rc<Scope<'q>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<'q> {
    pub scope: Rc<Scope<'q>>,
    pub field_node: FieldRef<'q>,
    pub field_def: &'q schema::Field<'q>,
}

pub type FieldSet<'q> = Vec<Field<'q>>;

#[derive(Debug, Clone)]
pub struct FetchGroup<'q> {
    pub service_name: String,
    pub fields: FieldSet<'q>,
    // This is only for auto_fragmentization -- which is currently unimplemented
    pub internal_fragments: LinkedHashMap<&'q str, &'q FragmentDefinition<'q>>,
    pub required_fields: FieldSet<'q>,
    pub provided_fields: FieldSet<'q>,
    pub dependent_groups_by_service: HashMap<String, FetchGroup<'q>>,
    pub other_dependent_groups: Vec<FetchGroup<'q>>,
    pub merge_at: Vec<ResponsePathElement>,
}

pub trait OwnedValues<'q> {
    fn owned_values(self) -> Vec<&'q FragmentDefinition<'q>>;
}

impl<'q> OwnedValues<'q> for LinkedHashMap<&'q str, &'q FragmentDefinition<'q>> {
    fn owned_values(self) -> Vec<&'q FragmentDefinition<'q>> {
        self.into_iter().map(|(_, v)| v).collect()
    }
}

impl<'q> FetchGroup<'q> {
    pub fn init(service_name: String) -> FetchGroup<'q> {
        FetchGroup::new(service_name, vec![], vec![])
    }

    pub fn new(
        service_name: String,
        merge_at: Vec<ResponsePathElement>,
        provided_fields: FieldSet<'q>,
    ) -> FetchGroup<'q> {
        FetchGroup {
            service_name,
            merge_at,
            provided_fields,

            fields: vec![],
            internal_fragments: LinkedHashMap::new(),
            required_fields: vec![],
            dependent_groups_by_service: HashMap::new(),
            other_dependent_groups: vec![],
        }
    }

    pub fn dependent_group_for_service<'a>(
        &'a mut self,
        service: String,
        required_fields: FieldSet<'q>,
    ) -> &'a mut FetchGroup<'q> {
        let group = self
            .dependent_groups_by_service
            .entry(service.clone())
            .or_insert_with(|| FetchGroup::init(service));

        if group.merge_at.is_empty() {
            group.merge_at = self.merge_at.clone();
        }

        if !required_fields.is_empty() {
            // TODO(ran) FIXME: this clones, ensure that's ok.
            group.required_fields.extend_from_slice(&required_fields);

            // TODO(ran) FIXME: consider using Rc for .fields and .required_fields
            self.fields.extend(required_fields.into_iter());
        }

        group
    }
}

// TODO(ran) FIXME: copy documentation comments from .ts
