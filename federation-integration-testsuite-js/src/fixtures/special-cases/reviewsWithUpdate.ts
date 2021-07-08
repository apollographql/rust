import gql from 'graphql-tag';

import * as reviewsService from '../reviews';

export const name = reviewsService.name;
export const url = reviewsService.url;

// For simplicity the "new" resolver just already exists - we only need to
// update the typeDefs with the newly added `review` field.
export const resolvers = reviewsService.resolvers;

// Add a new `review` field to the `Query` type
export const typeDefs = gql`
  extend type Query {
    review(id: ID!): Review
  }
  ${reviewsService.typeDefs}
`;
