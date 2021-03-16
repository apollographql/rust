import { loadCsdlFromStorage } from '../loadCsdlFromStorage';
import { getDefaultFetcher } from '../..';
import {
  mockCsdlRequestSuccess,
  graphId,
  graphVariant,
  apiKey,
  mockCloudConfigUrl,
  mockCsdlRequest,
} from './integration/nockMocks';

describe('loadCsdlFromStorage', () => {
  it('fetches CSDL as expected', async () => {
    mockCsdlRequestSuccess();
    const fetcher = getDefaultFetcher();
    const result = await loadCsdlFromStorage({
      graphId,
      graphVariant,
      apiKey,
      endpoint: mockCloudConfigUrl,
      fetcher,
    });

    expect(result).toMatchInlineSnapshot(`
      Object {
        "csdl": "schema
        @graph(name: \\"accounts\\", url: \\"https://accounts.api.com\\")
        @graph(name: \\"books\\", url: \\"https://books.api.com\\")
        @graph(name: \\"documents\\", url: \\"https://documents.api.com\\")
        @graph(name: \\"inventory\\", url: \\"https://inventory.api.com\\")
        @graph(name: \\"product\\", url: \\"https://product.api.com\\")
        @graph(name: \\"reviews\\", url: \\"https://reviews.api.com\\")
        @composedGraph(version: 1)
      {
        query: Query
        mutation: Mutation
      }

      directive @composedGraph(version: Int!) on SCHEMA

      directive @graph(name: String!, url: String!) repeatable on SCHEMA

      directive @owner(graph: String!) on OBJECT

      directive @key(fields: String!, graph: String!) repeatable on OBJECT

      directive @resolve(graph: String!) on FIELD_DEFINITION

      directive @provides(fields: String!) on FIELD_DEFINITION

      directive @requires(fields: String!) on FIELD_DEFINITION

      directive @stream on FIELD

      directive @transform(from: String!) on FIELD

      union AccountType = PasswordAccount | SMSAccount

      type Amazon {
        referrer: String
      }

      union Body = Image | Text

      type Book implements Product
        @owner(graph: \\"books\\")
        @key(fields: \\"{ isbn }\\", graph: \\"books\\")
        @key(fields: \\"{ isbn }\\", graph: \\"inventory\\")
        @key(fields: \\"{ isbn }\\", graph: \\"product\\")
        @key(fields: \\"{ isbn }\\", graph: \\"reviews\\")
      {
        isbn: String!
        title: String
        year: Int
        similarBooks: [Book]!
        metadata: [MetadataOrError]
        inStock: Boolean @resolve(graph: \\"inventory\\")
        isCheckedOut: Boolean @resolve(graph: \\"inventory\\")
        upc: String! @resolve(graph: \\"product\\")
        sku: String! @resolve(graph: \\"product\\")
        name(delimeter: String = \\" \\"): String @resolve(graph: \\"product\\") @requires(fields: \\"{ title year }\\")
        price: String @resolve(graph: \\"product\\")
        details: ProductDetailsBook @resolve(graph: \\"product\\")
        reviews: [Review] @resolve(graph: \\"reviews\\")
        relatedReviews: [Review!]! @resolve(graph: \\"reviews\\") @requires(fields: \\"{ similarBooks { isbn } }\\")
      }

      union Brand = Ikea | Amazon

      type Car implements Vehicle
        @owner(graph: \\"product\\")
        @key(fields: \\"{ id }\\", graph: \\"product\\")
        @key(fields: \\"{ id }\\", graph: \\"reviews\\")
      {
        id: String!
        description: String
        price: String
        retailPrice: String @resolve(graph: \\"reviews\\") @requires(fields: \\"{ price }\\")
      }

      type Error {
        code: Int
        message: String
      }

      type Furniture implements Product
        @owner(graph: \\"product\\")
        @key(fields: \\"{ upc }\\", graph: \\"product\\")
        @key(fields: \\"{ sku }\\", graph: \\"product\\")
        @key(fields: \\"{ sku }\\", graph: \\"inventory\\")
        @key(fields: \\"{ upc }\\", graph: \\"reviews\\")
      {
        upc: String!
        sku: String!
        name: String
        price: String
        brand: Brand
        metadata: [MetadataOrError]
        details: ProductDetailsFurniture
        inStock: Boolean @resolve(graph: \\"inventory\\")
        isHeavy: Boolean @resolve(graph: \\"inventory\\")
        reviews: [Review] @resolve(graph: \\"reviews\\")
      }

      type Ikea {
        asile: Int
      }

      type Image {
        name: String!
        attributes: ImageAttributes!
      }

      type ImageAttributes {
        url: String!
      }

      type KeyValue {
        key: String!
        value: String!
      }

      type Library
        @owner(graph: \\"books\\")
        @key(fields: \\"{ id }\\", graph: \\"books\\")
        @key(fields: \\"{ id }\\", graph: \\"accounts\\")
      {
        id: ID!
        name: String
        userAccount(id: ID! = 1): User @resolve(graph: \\"accounts\\") @requires(fields: \\"{ name }\\")
      }

      union MetadataOrError = KeyValue | Error

      type Mutation {
        login(username: String!, password: String!): User @resolve(graph: \\"accounts\\")
        reviewProduct(upc: String!, body: String!): Product @resolve(graph: \\"reviews\\")
        updateReview(review: UpdateReviewInput!): Review @resolve(graph: \\"reviews\\")
        deleteReview(id: ID!): Boolean @resolve(graph: \\"reviews\\")
      }

      type Name {
        first: String
        last: String
      }

      type PasswordAccount
        @owner(graph: \\"accounts\\")
        @key(fields: \\"{ email }\\", graph: \\"accounts\\")
      {
        email: String!
      }

      interface Product {
        upc: String!
        sku: String!
        name: String
        price: String
        details: ProductDetails
        inStock: Boolean
        reviews: [Review]
      }

      interface ProductDetails {
        country: String
      }

      type ProductDetailsBook implements ProductDetails {
        country: String
        pages: Int
      }

      type ProductDetailsFurniture implements ProductDetails {
        country: String
        color: String
      }

      type Query {
        user(id: ID!): User @resolve(graph: \\"accounts\\")
        me: User @resolve(graph: \\"accounts\\")
        book(isbn: String!): Book @resolve(graph: \\"books\\")
        books: [Book] @resolve(graph: \\"books\\")
        library(id: ID!): Library @resolve(graph: \\"books\\")
        body: Body! @resolve(graph: \\"documents\\")
        product(upc: String!): Product @resolve(graph: \\"product\\")
        vehicle(id: String!): Vehicle @resolve(graph: \\"product\\")
        topProducts(first: Int = 5): [Product] @resolve(graph: \\"product\\")
        topCars(first: Int = 5): [Car] @resolve(graph: \\"product\\")
        topReviews(first: Int = 5): [Review] @resolve(graph: \\"reviews\\")
      }

      type Review
        @owner(graph: \\"reviews\\")
        @key(fields: \\"{ id }\\", graph: \\"reviews\\")
      {
        id: ID!
        body(format: Boolean = false): String
        author: User @provides(fields: \\"{ username }\\")
        product: Product
        metadata: [MetadataOrError]
      }

      type SMSAccount
        @owner(graph: \\"accounts\\")
        @key(fields: \\"{ number }\\", graph: \\"accounts\\")
      {
        number: String
      }

      type Text {
        name: String!
        attributes: TextAttributes!
      }

      type TextAttributes {
        bold: Boolean
        text: String
      }

      union Thing = Car | Ikea

      input UpdateReviewInput {
        id: ID!
        body: String
      }

      type User
        @owner(graph: \\"accounts\\")
        @key(fields: \\"{ id }\\", graph: \\"accounts\\")
        @key(fields: \\"{ username name { first last } }\\", graph: \\"accounts\\")
        @key(fields: \\"{ id }\\", graph: \\"inventory\\")
        @key(fields: \\"{ id }\\", graph: \\"product\\")
        @key(fields: \\"{ id }\\", graph: \\"reviews\\")
      {
        id: ID!
        name: Name
        username: String
        birthDate(locale: String): String
        account: AccountType
        metadata: [UserMetadata]
        goodDescription: Boolean @resolve(graph: \\"inventory\\") @requires(fields: \\"{ metadata { description } }\\")
        vehicle: Vehicle @resolve(graph: \\"product\\")
        thing: Thing @resolve(graph: \\"product\\")
        reviews: [Review] @resolve(graph: \\"reviews\\")
        numberOfReviews: Int! @resolve(graph: \\"reviews\\")
        goodAddress: Boolean @resolve(graph: \\"reviews\\") @requires(fields: \\"{ metadata { address } }\\")
      }

      type UserMetadata {
        name: String
        address: String
        description: String
      }

      type Van implements Vehicle
        @owner(graph: \\"product\\")
        @key(fields: \\"{ id }\\", graph: \\"product\\")
        @key(fields: \\"{ id }\\", graph: \\"reviews\\")
      {
        id: String!
        description: String
        price: String
        retailPrice: String @resolve(graph: \\"reviews\\") @requires(fields: \\"{ price }\\")
      }

      interface Vehicle {
        id: String!
        description: String
        price: String
        retailPrice: String
      }
      ",
        "id": "originalId-1234",
      }
    `);
  });

  describe('errors', () => {
    it('throws on a malformed response', async () => {
      mockCsdlRequest().reply(200, 'Invalid JSON');

      const fetcher = getDefaultFetcher();
      await expect(
        loadCsdlFromStorage({
          graphId,
          graphVariant,
          apiKey,
          endpoint: mockCloudConfigUrl,
          fetcher,
        }),
      ).rejects.toThrowErrorMatchingInlineSnapshot(
        `"An error occurred while fetching your schema from Apollo: 200 invalid json response body at https://example.cloud-config-url.com/cloudconfig/ reason: Unexpected token I in JSON at position 0"`,
      );
    });

    it('throws errors from JSON on 400', async () => {
      const message = 'Query syntax error';
      mockCsdlRequest().reply(
        400,
        JSON.stringify({
          errors: [{ message }],
        }),
      );

      const fetcher = getDefaultFetcher();
      await expect(
        loadCsdlFromStorage({
          graphId,
          graphVariant,
          apiKey,
          endpoint: mockCloudConfigUrl,
          fetcher,
        }),
      ).rejects.toThrowError(message);
    });

    it("throws on non-OK status codes when `errors` isn't present in a JSON response", async () => {
      mockCsdlRequest().reply(500);

      const fetcher = getDefaultFetcher();
      await expect(
        loadCsdlFromStorage({
          graphId,
          graphVariant,
          apiKey,
          endpoint: mockCloudConfigUrl,
          fetcher,
        }),
      ).rejects.toThrowErrorMatchingInlineSnapshot(
        `"An error occurred while fetching your schema from Apollo: 500 Internal Server Error"`,
      );
    });
  });
});
