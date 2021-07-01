export type Maybe<T> = T | null;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  /** ISO 8601, extended format with nanoseconds, Zulu (or '[+-]seconds' for times relative to now) */
  Timestamp: any;
};

export type ApiMonitoringReport = {
  error: Error;
  request: Request;
  response?: Maybe<Response>;
  startedAt: Scalars['Timestamp'];
  endedAt: Scalars['Timestamp'];
  /** Tags can include things like version and package name */
  tags?: Maybe<Array<Scalars['String']>>;
};

export type Error = {
  code: ErrorCode;
  message?: Maybe<Scalars['String']>;
};

export enum ErrorCode {
  InvalidBody = 'INVALID_BODY',
  UnexpectedResponse = 'UNEXPECTED_RESPONSE',
  ConnectionFailed = 'CONNECTION_FAILED',
  Timeout = 'TIMEOUT',
  Other = 'OTHER'
}

export type FetchError = {
  __typename?: 'FetchError';
  code: FetchErrorCode;
  message: Scalars['String'];
};

export enum FetchErrorCode {
  /** This token provided is not a valid graph token. Do not retry */
  AuthenticationFailed = 'AUTHENTICATION_FAILED',
  /** This token does not have access to fetch the schema for this ref. Do not retry. */
  AccessDenied = 'ACCESS_DENIED',
  /** The graphRef passed is not a valid ref or no configuration for that ref is found. Do not retry */
  UnknownRef = 'UNKNOWN_REF',
  /** An internal server error occurred. Please retry with some backoff */
  RetryLater = 'RETRY_LATER'
}

export type HttpHeader = {
  name: Scalars['String'];
  value?: Maybe<Scalars['String']>;
};

export type Message = {
  __typename?: 'Message';
  level: MessageLevel;
  body: Scalars['String'];
};

export enum MessageLevel {
  Error = 'ERROR',
  Warn = 'WARN',
  Info = 'INFO'
}

export type Mutation = {
  __typename?: 'Mutation';
  reportError: Scalars['Boolean'];
};


export type MutationReportErrorArgs = {
  report?: Maybe<ApiMonitoringReport>;
};

export type Query = {
  __typename?: 'Query';
  _empty?: Maybe<Scalars['String']>;
  /** Fetch the configuration for a router. If a valid configuration is available, it will be readable as cSDL. */
  routerConfig: RouterConfigResponse;
};


export type QueryRouterConfigArgs = {
  ref: Scalars['String'];
  apiKey: Scalars['String'];
  supportedSpecURLs?: Array<Scalars['String']>;
};

export type Request = {
  url: Scalars['String'];
  headers?: Maybe<Array<HttpHeader>>;
  body?: Maybe<Scalars['String']>;
};

export type Response = {
  httpStatusCode: Scalars['Int'];
  headers?: Maybe<Array<HttpHeader>>;
  body?: Maybe<Scalars['String']>;
};

export type RouterConfigResponse = RouterConfigResult | FetchError;

export type RouterConfigResult = {
  __typename?: 'RouterConfigResult';
  id: Scalars['ID'];
  /** The configuration as core schema */
  supergraphSDL: Scalars['String'];
  /** Messages that should be reported back to the operators of this router, eg through logs and/or monitoring. */
  messages: Array<Message>;
};


export type SupergraphSdlQueryVariables = Exact<{
  apiKey: Scalars['String'];
  ref: Scalars['String'];
}>;


export type SupergraphSdlQuery = (
  { __typename?: 'Query' }
  & { routerConfig: (
    { __typename: 'RouterConfigResult' }
    & Pick<RouterConfigResult, 'id'>
    & { supergraphSdl: RouterConfigResult['supergraphSDL'] }
  ) | (
    { __typename: 'FetchError' }
    & Pick<FetchError, 'code' | 'message'>
  ) }
);

export type OobReportMutationVariables = Exact<{
  input?: Maybe<ApiMonitoringReport>;
}>;


export type OobReportMutation = (
  { __typename?: 'Mutation' }
  & Pick<Mutation, 'reportError'>
);
