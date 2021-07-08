import { URL } from "apollo-server-env";

export interface ParsedFeatureURL {
  identity: string;
  name: string;
  version: string;
}

export function parseFeatureURL(feature: string): ParsedFeatureURL {
  const url = new URL(feature);

  const path = url.pathname.split('/');

  // Remove trailing slashes
  while (path[path.length - 1] === '') {
    path.pop();
  }

  const version = path.pop()!;

  const name = path[path.length - 1];

  // This was copied from core-schema-js, not sure if this normalization is
  // part of the spec?
  url.hash = '';
  url.search = '';
  url.password = '';
  url.username = '';
  url.pathname = path.join('/');

  return { identity: url.toString(), name, version };
}
