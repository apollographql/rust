# Distributed data schema for the atlas

"""
A public key identity.
"""
scalar Identity

"""
A cryptographic hash of a message.
"""
scalar Hash

"""
Initialize an atlas. The id of the new atlas is the hash of this signed
message.

Assertions: None. Init blocks are axioms; all are valid.
"""
input Atlas @signed(field: "by") {  
  by: Identity
  description: String
}

"""
Grant another identity the ability to announce on the atlas
until a certain date.

Assertions: `grantor` must be the creator of the atlas or must have
been granted privileges for `name`.
"""
input Grant @signed(field: "by") {
  atlas: Atlas
  by: Identity
  for: Identity
  name: atlas.Name
  expires: Date
}

input Revoke @signed(field: "by") {
  atlas: Atlas
  by: Identity
  for: Identity
  name: atlas.Name
  expires: Date
}


"""
A schema endpoint
"""
scalar Endpoint

input Announce @signed(field: "by") {
  atlas: Atlas
  by: Identity
  name: atlas.Name
  schema: Hash
  endpoint: Endpoint
  expires: Date
}