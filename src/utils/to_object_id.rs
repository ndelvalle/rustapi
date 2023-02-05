use bson::oid::ObjectId;

use crate::errors::Error;

pub fn to_object_id<S: AsRef<str>>(id: S) -> Result<ObjectId, Error> {
  ObjectId::parse_str(id.as_ref()).map_err(|_| Error::ParseObjectID(id.as_ref().to_string()))
}
