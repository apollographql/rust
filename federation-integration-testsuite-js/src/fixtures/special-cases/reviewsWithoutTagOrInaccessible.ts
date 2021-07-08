import gql from 'graphql-tag';

export { name, url, resolvers } from '../reviews';
export const typeDefs = gql`
  directive @stream on FIELD
  directive @transform(from: String!) on FIELD

  extend type Query {
    topReviews(first: Int = 5): [Review]
  }

  type Review @key(fields: "id") {
    id: ID!
    body(format: Boolean = false): String
    author: User @provides(fields: "username")
    product: Product
    metadata: [MetadataOrError]
  }

  input UpdateReviewInput {
    id: ID!
    body: String
  }

  extend type UserMetadata {
    address: String @external
  }

  extend type User @key(fields: "id") {
    id: ID! @external
    username: String @external
    reviews: [Review]
    numberOfReviews: Int!
    metadata: [UserMetadata] @external
    goodAddress: Boolean @requires(fields: "metadata { address }")
  }

  extend interface Product {
    reviews: [Review]
  }

  extend type Furniture implements Product @key(fields: "upc") {
    upc: String! @external
    reviews: [Review]
  }

  extend type Book implements Product @key(fields: "isbn") {
    isbn: String! @external
    reviews: [Review]
    similarBooks: [Book]! @external
    relatedReviews: [Review!]! @requires(fields: "similarBooks { isbn }")
  }

  extend interface Vehicle {
    retailPrice: String
  }

  extend type Car implements Vehicle @key(fields: "id") {
    id: String! @external
    price: String @external
    retailPrice: String @requires(fields: "price")
  }

  extend type Van implements Vehicle @key(fields: "id") {
    id: String! @external
    price: String @external
    retailPrice: String @requires(fields: "price")
  }

  extend type Mutation {
    reviewProduct(upc: String!, body: String!): Product
    updateReview(review: UpdateReviewInput!): Review
    deleteReview(id: ID!): Boolean
  }

  # Value type
  type KeyValue {
    key: String!
    value: String!
  }

  # Value type
  type Error {
    code: Int
    message: String
  }

  # Value type
  union MetadataOrError = KeyValue | Error
`;
