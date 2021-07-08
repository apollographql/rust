import gql from 'graphql-tag';

export { name, url, resolvers } from '../accounts';
export const typeDefs = gql`
  directive @stream on FIELD
  directive @transform(from: String!) on FIELD

  schema {
    query: RootQuery
    mutation: Mutation
  }

  extend type RootQuery {
    user(id: ID!): User
    me: User
  }

  type PasswordAccount @key(fields: "email") {
    email: String!
  }

  type SMSAccount @key(fields: "number") {
    number: String
  }

  union AccountType = PasswordAccount | SMSAccount

  type UserMetadata {
    name: String
    address: String
    description: String
  }

  type User @key(fields: "id") @key(fields: "username name { first last }") {
    id: ID!
    name: Name
    username: String
    birthDate(locale: String): String
    account: AccountType
    metadata: [UserMetadata]
    ssn: String
  }

  type Name {
    first: String
    last: String
  }

  type Mutation {
    login(username: String!, password: String!): User
  }

  extend type Library @key(fields: "id") {
    id: ID! @external
    name: String @external
    userAccount(id: ID! = "1"): User @requires(fields: "name")
  }
`;
